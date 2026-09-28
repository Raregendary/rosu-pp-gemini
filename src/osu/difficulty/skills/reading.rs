use crate::{
    model::mods::GameMods,
    osu::difficulty::{
        context::OsuDifficultyContext, evaluators::ReadingEvaluator, object::OsuDifficultyObject,
    },
    util::{difficulty::logistic, float_ext::FloatExt},
};

use super::{scratch, weights};

#[derive(Clone, Debug)]
pub struct Reading {
    has_hidden_mod: bool,
    current_strain: f64,
    object_difficulties: Vec<f64>,
    start_times: Vec<f64>,
    is_touch_device: bool,
    is_relax: bool,
    is_autopilot: bool,
    ctx: OsuDifficultyContext,
}

impl Reading {
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

    const HARMONIC_SCALE: f64 = 1.0;
    const DECAY_EXPONENT: f64 = 0.9;

    pub fn new(mods: &GameMods, ctx: OsuDifficultyContext) -> Self {
        Self {
            has_hidden_mod: mods.hd(),
            current_strain: 0.0,
            object_difficulties: Vec::new(),
            start_times: Vec::new(),
            is_touch_device: mods.td(),
            is_relax: mods.rx(),
            is_autopilot: mods.ap(),
            ctx,
        }
    }

    pub fn with_capacity(mods: &GameMods, total_objects: usize, ctx: OsuDifficultyContext) -> Self {
        let mut this = Self::new(mods, ctx);

        // Both grow by at most one entry per processed object; reserving up front
        // avoids repeated reallocation on every calculation.
        this.object_difficulties = Vec::with_capacity(total_objects);
        this.start_times = Vec::with_capacity(total_objects);

        this
    }

    fn strain_decay(ms: f64) -> f64 {
        0.8_f64.powf(ms / 1000.0)
    }

    pub fn process(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        self.start_times.push(curr.start_time);
        let diff = self.object_difficulty_of(curr, diff_objects);
        self.object_difficulties.push(diff);
    }

    fn object_difficulty_of(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let num = Self::strain_decay(curr.delta_time);
        self.current_strain *= num;
        self.current_strain +=
            self.calculate_adjusted_difficulty(curr, diff_objects) * (1.0 - num) * 2.5;

        self.current_strain
    }

    fn calculate_adjusted_difficulty(
        &self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let mut num =
            ReadingEvaluator::evaluate_diff_of(curr, diff_objects, self.has_hidden_mod, &self.ctx);

        if self.is_touch_device {
            num = num.powf(0.89);
        }

        if self.is_relax {
            num *= 0.4;
        }

        if self.is_autopilot {
            num *= 0.1;
        }

        num * self.ctx.reading_od_factor
    }

    #[allow(dead_code)]
    pub fn cloned_difficulty_value(&self) -> f64 {
        self.difficulty_value().0
    }

    /// Returns the difficulty value together with the sum of the weights that
    /// produced it, which [`Self::count_top_weighted_object_difficulties`] needs.
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

            let num_reduced = self.calculate_reduced_note_count();
            let limit = diffs.len().min(num_reduced);

            for num2 in 0..limit {
                let clamped = (num2 as f64 / num_reduced as f64).clamp(0.0, 1.0);
                let num3 = (1.0 + (10.0 - 1.0) * clamped).log10();
                diffs[num2] *= num3;
            }

            // * An unstable sort is exact here (unlike in `Aim`, see the note in
            // * `Aim::difficulty_value`): the elements are plain `f64`s that are
            // * interchangeable when equal, and the weight applied to a position
            // * depends only on that position, so two equal values swapped still
            // * produce the identical sequence of products.
            diffs.sort_unstable_by(|a, b| b.total_cmp(a));

            weights::weighted_sum(
                weights::reading_weights(),
                Self::HARMONIC_SCALE,
                Self::DECAY_EXPONENT,
                diffs,
            )
        })
    }

    fn calculate_reduced_note_count(&self) -> usize {
        let Some(&first_start_time) = self.start_times.first() else {
            return 0;
        };

        let cutoff = first_start_time + 60000.0;
        self.start_times
            .iter()
            .take_while(|&&t| t <= cutoff)
            .count()
    }

    pub fn count_top_weighted_object_difficulties(
        &self,
        difficulty_value: f64,
        object_weight_sum: f64,
    ) -> f64 {
        if self.object_difficulties.is_empty() || FloatExt::eq(object_weight_sum, 0.0) {
            return 0.0;
        }

        let consistent_top_note = difficulty_value / object_weight_sum;
        if FloatExt::eq(consistent_top_note, 0.0) {
            return 0.0;
        }

        self.object_difficulties
            .iter()
            .map(|&d| logistic(d / consistent_top_note, 1.15, 5.0, Some(1.1)))
            .sum()
    }

    #[allow(dead_code)]
    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        4.0 * difficulty.powf(3.0)
    }
}
