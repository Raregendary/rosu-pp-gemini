use crate::{
    any::difficulty::object::IDifficultyObject,
    model::mods::GameMods,
    osu::difficulty::{
        evaluators::{RhythmEvaluator, SpeedEvaluator},
        object::OsuDifficultyObject,
    },
    util::{difficulty::logistic, float_ext::FloatExt},
};

#[derive(Clone, Debug)]
pub struct Speed {
    current_strain: f64,
    object_difficulties: Vec<f64>,
    slider_strains: Vec<f64>,
    timeline_peaks: Vec<f64>,
    timeline_section_end: f64,
    timeline_section_peak: f64,
    pub object_weight_sum: f64,
    is_relax: bool,
    is_autopilot: bool,
}

impl Speed {
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
            object_weight_sum: 0.0,
            is_relax: mods.rx(),
            is_autopilot: mods.ap(),
        }
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
        self.current_strain += self.calculate_adjusted_difficulty(curr, diff_objects) * (1.0 - num) * 1.16;

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
        let mut clone = self.clone();
        clone.difficulty_value()
    }

    pub fn into_current_strain_peaks(mut self) -> Vec<f64> {
        self.timeline_peaks.push(self.timeline_section_peak);
        self.timeline_peaks
    }

    pub fn difficulty_value(&mut self) -> f64 {
        if self.object_difficulties.is_empty() {
            return 0.0;
        }

        let mut diffs: Vec<f64> = self
            .object_difficulties
            .iter()
            .copied()
            .filter(|&v| v > 0.0)
            .collect();

        if diffs.is_empty() {
            return 0.0;
        }

        diffs.sort_by(|a, b| b.total_cmp(a));

        let mut num = 0.0;
        let mut num2 = 0;
        self.object_weight_sum = 0.0;

        for item in diffs {
            let scale_term = Self::HARMONIC_SCALE / (1 + num2) as f64;
            let num3 = (1.0 + scale_term)
                / ((num2 as f64).powf(Self::DECAY_EXPONENT) + 1.0 + scale_term);

            self.object_weight_sum += num3;
            num += item * num3;
            num2 += 1;
        }

        num
    }

    pub fn count_top_weighted_object_difficulties(&self, difficulty_value: f64) -> f64 {
        if self.object_difficulties.is_empty() || FloatExt::eq(self.object_weight_sum, 0.0) {
            return 0.0;
        }

        let consistent_top_object = difficulty_value / self.object_weight_sum;
        if FloatExt::eq(consistent_top_object, 0.0) {
            return 0.0;
        }

        self.object_difficulties
            .iter()
            .map(|&d| logistic(d / consistent_top_object, 0.88, 10.0, Some(1.1)))
            .sum()
    }

    pub fn count_top_weighted_sliders(&self, difficulty_value: f64) -> f64 {
        if self.slider_strains.is_empty() || FloatExt::eq(self.object_weight_sum, 0.0) {
            return 0.0;
        }

        let consistent_top_object = difficulty_value / self.object_weight_sum;
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
