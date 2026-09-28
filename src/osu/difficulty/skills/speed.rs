use crate::{
    any::difficulty::object::IDifficultyObject,
    model::mods::GameMods,
    osu::difficulty::{
        evaluators::{RhythmEvaluator, SpeedEvaluator},
        object::OsuDifficultyObject,
    },
    util::{difficulty::logistic, float_ext::FloatExt},
};

use super::{scratch, weights};

#[derive(Clone, Debug)]
pub struct Speed {
    current_strain: f64,
    object_difficulties: Vec<f64>,
    slider_strains: Vec<f64>,
    timeline_peaks: Vec<f64>,
    timeline_section_end: f64,
    timeline_section_peak: f64,
    is_relax: bool,
    is_autopilot: bool,
}

impl Speed {
    /// Runs [`Self::process`] over the first `take` objects.
    ///
    /// Exists so that [`OsuSkills::process_all`](super::OsuSkills::process_all)
    /// can hand whole skills to a thread pool: each skill is sequential in the
    /// objects, so spreading *skills* is the only parallelism available.
    ///
    /// `objects` stays the full list rather than being truncated to `take`,
    /// because evaluators look at neighbouring objects and a shortened slice
    /// would change the last processed object's difficulty.
    pub fn process_all(&mut self, objects: &[OsuDifficultyObject<'_>], take: usize) {
        for curr in objects.iter().take(take) {
            self.process(curr, objects);
        }
    }

    const HARMONIC_SCALE: f64 = 20.0;
    const DECAY_EXPONENT: f64 = 0.9;

    pub fn new(mods: &GameMods) -> Self {
        Self {
            current_strain: 0.0,
            object_difficulties: Vec::new(),
            slider_strains: Vec::new(),
            timeline_peaks: Vec::new(),
            timeline_section_end: 0.0,
            timeline_section_peak: 0.0,
            is_relax: mods.rx(),
            is_autopilot: mods.ap(),
        }
    }

    pub fn with_capacity(mods: &GameMods, total_objects: usize) -> Self {
        let mut this = Self::new(mods);

        // These all grow by at most one entry per processed object; reserving up
        // front avoids repeated reallocation on every calculation.
        this.object_difficulties = Vec::with_capacity(total_objects);
        this.slider_strains = Vec::with_capacity(total_objects);
        this.timeline_peaks = Vec::with_capacity(total_objects / 4 + 1);

        this
    }

    fn strain_decay(ms: f64) -> f64 {
        0.3_f64.powf(ms / 1000.0)
    }

    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let prev_start_time = curr.previous(0, diff_objects).map_or(0.0, |p| p.start_time);
        self.current_strain * Self::strain_decay(time - prev_start_time)
    }

    pub fn process(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        if curr.idx == 0 {
            self.timeline_section_end = (curr.start_time / 400.0).ceil() * 400.0;
        }

        while curr.start_time > self.timeline_section_end {
            self.timeline_peaks.push(self.timeline_section_peak);
            self.timeline_section_peak =
                self.calculate_initial_strain(self.timeline_section_end, curr, diff_objects);
            self.timeline_section_end += 400.0;
        }

        let diff = self.object_difficulty_of(curr, diff_objects);
        self.object_difficulties.push(diff);
        self.timeline_section_peak = f64::max(self.timeline_section_peak, diff);
    }

    fn object_difficulty_of(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        if self.is_relax {
            return 0.0;
        }

        let num = Self::strain_decay(curr.adjusted_delta_time);
        self.current_strain *= num;
        self.current_strain +=
            self.calculate_adjusted_difficulty(curr, diff_objects) * (1.0 - num) * 1.16;

        let num2 = RhythmEvaluator::evaluate_diff_of(curr, diff_objects);
        let num3 = self.current_strain * num2;

        if curr.base.is_slider() {
            self.slider_strains.push(num3);
        }

        num3
    }

    fn calculate_adjusted_difficulty(
        &self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let mut num = SpeedEvaluator::evaluate_diff_of(curr, diff_objects);
        if self.is_autopilot {
            num *= 0.5;
        }
        num
    }

    pub fn relevant_object_count(&self) -> f64 {
        if self.object_difficulties.is_empty() {
            return 0.0;
        }

        let max_strain = self.object_difficulties.iter().copied().fold(0.0, f64::max);
        if FloatExt::eq(max_strain, 0.0) {
            return 0.0;
        }

        self.object_difficulties
            .iter()
            .copied()
            .map(|strain| 1.0 / (1.0 + (-(strain / max_strain * 12.0 - 6.0)).exp()))
            .sum()
    }

    #[allow(dead_code)]
    pub fn cloned_difficulty_value(&self) -> f64 {
        self.difficulty_value().0
    }

    pub fn into_current_strain_peaks(mut self) -> Vec<f64> {
        self.timeline_peaks.push(self.timeline_section_peak);
        self.timeline_peaks
    }

    /// Returns the difficulty value together with the sum of the weights that
    /// produced it, which [`Self::count_top_weighted_object_difficulties`] and
    /// [`Self::count_top_weighted_sliders`] need.
    ///
    /// This takes `&self` rather than `&mut self` on purpose: `eval` runs once
    /// per hit object while calculating gradual difficulty, so keeping the
    /// weight sum in a field would force every caller to clone the whole skill.
    pub fn difficulty_value(&self) -> (f64, f64) {
        if self.object_difficulties.is_empty() {
            return (0.0, 0.0);
        }

        // The reduction runs once per hit object while calculating gradual
        // difficulty, so the working list comes out of a reusable buffer rather
        // than a fresh allocation each time.
        scratch::with_object_difficulties(|diffs| {
            diffs.clear();
            diffs.extend(
                self.object_difficulties
                    .iter()
                    .copied()
                    .filter(|&v| v > 0.0),
            );

            if diffs.is_empty() {
                return (0.0, 0.0);
            }

            // * An unstable sort is exact here (unlike in `Aim`, see the note in
            // * `Aim::difficulty_value`): the elements are plain `f64`s that are
            // * interchangeable when equal, and the weight applied to a position
            // * depends only on that position, so two equal values swapped still
            // * produce the identical sequence of products.
            diffs.sort_unstable_by(|a, b| b.total_cmp(a));

            weights::weighted_sum(
                weights::speed_weights(),
                Self::HARMONIC_SCALE,
                Self::DECAY_EXPONENT,
                diffs,
            )
        })
    }

    pub fn count_top_weighted_object_difficulties(
        &self,
        difficulty_value: f64,
        object_weight_sum: f64,
    ) -> f64 {
        if self.object_difficulties.is_empty() || FloatExt::eq(object_weight_sum, 0.0) {
            return 0.0;
        }

        let consistent_top_object = difficulty_value / object_weight_sum;
        if FloatExt::eq(consistent_top_object, 0.0) {
            return 0.0;
        }

        self.object_difficulties
            .iter()
            .map(|&d| logistic(d / consistent_top_object, 0.88, 10.0, Some(1.1)))
            .sum()
    }

    pub fn count_top_weighted_sliders(&self, difficulty_value: f64, object_weight_sum: f64) -> f64 {
        if self.slider_strains.is_empty() || FloatExt::eq(object_weight_sum, 0.0) {
            return 0.0;
        }

        let consistent_top_object = difficulty_value / object_weight_sum;
        if FloatExt::eq(consistent_top_object, 0.0) {
            return 0.0;
        }

        self.slider_strains
            .iter()
            .map(|&s| logistic(s / consistent_top_object, 0.88, 10.0, Some(1.1)))
            .sum()
    }

    #[allow(dead_code)]
    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        4.0 * difficulty.powf(3.0)
    }
}
