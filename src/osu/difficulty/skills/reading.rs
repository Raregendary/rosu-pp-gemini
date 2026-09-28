use crate::{
    model::mods::GameMods,
    osu::difficulty::{evaluators::ReadingEvaluator, object::OsuDifficultyObject},
    util::{difficulty::logistic, float_ext::FloatExt},
};

#[derive(Clone, Debug)]
pub struct Reading {
    has_hidden_mod: bool,
    current_strain: f64,
    object_difficulties: Vec<f64>,
    start_times: Vec<f64>,
    is_touch_device: bool,
    is_relax: bool,
    is_autopilot: bool,
}

impl Reading {
    const HARMONIC_SCALE: f64 = 1.0;
    const DECAY_EXPONENT: f64 = 0.9;

    pub fn new(mods: &GameMods) -> Self {
        Self {
            has_hidden_mod: mods.hd(),
            current_strain: 0.0,
            object_difficulties: Vec::new(),
            start_times: Vec::new(),
            is_touch_device: mods.td(),
            is_relax: mods.rx(),
            is_autopilot: mods.ap(),
        }
    }

    pub fn with_capacity(mods: &GameMods, total_objects: usize) -> Self {
        let mut this = Self::new(mods);

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
        let mut num = ReadingEvaluator::evaluate_diff_of(curr, diff_objects, self.has_hidden_mod);

        if self.is_touch_device {
            num = num.powf(0.89);
        }

        if self.is_relax {
            num *= 0.4;
        }

        if self.is_autopilot {
            num *= 0.1;
        }

        num * (0.825 + curr.overall_difficulty().max(0.0).powf(2.2) / 1125.0)
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

        let mut diffs: Vec<f64> = self
            .object_difficulties
            .iter()
            .copied()
            .filter(|&v| v > 0.0)
            .collect();

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

        diffs.sort_by(|a, b| b.total_cmp(a));

        let mut num = 0.0;
        let mut num2 = 0;
        let mut object_weight_sum = 0.0;

        for item in diffs {
            let scale_term = Self::HARMONIC_SCALE / (1 + num2) as f64;
            let num3 = (1.0 + scale_term)
                / ((num2 as f64).powf(Self::DECAY_EXPONENT) + 1.0 + scale_term);

            object_weight_sum += num3;
            num += item * num3;
            num2 += 1;
        }

        (num, object_weight_sum)
    }

    fn calculate_reduced_note_count(&self) -> usize {
        let Some(&first_start_time) = self.start_times.first() else {
            return 0;
        };

        let cutoff = first_start_time + 60000.0;
        self.start_times.iter().take_while(|&&t| t <= cutoff).count()
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
