use crate::{
    GameMods,
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills::strain_decay,
    },
    osu::difficulty::{evaluators::FlashlightEvaluator, object::OsuDifficultyObject},
    util::traits::IEnumerable,
};

define_skill! {
    pub struct Flashlight: StrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        current_strain: f64,
        has_hidden_mod: bool,
        is_touch_device: bool,
        is_relax: bool,
        is_autopilot: bool,
        total_objects: usize,
        evaluator: FlashlightEvaluator,
    }

    pub fn new(mods: &GameMods, radius: f64, total_objects: usize) -> Self {
        let scaling_factor = 52.0 / radius;

        Self {
            current_strain: 0.0,
            has_hidden_mod: mods.hd(),
            is_touch_device: mods.td(),
            is_relax: mods.rx(),
            is_autopilot: mods.ap(),
            total_objects: total_objects,
            evaluator: FlashlightEvaluator::new(scaling_factor),
        }
    }
}

impl Flashlight {
    const SKILL_MULTIPLIER: f64 = 0.058;
    const STRAIN_DECAY_BASE: f64 = 0.15;

    fn calculate_initial_strain(
        &mut self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, HasStartTime::start_time);

        self.current_strain * strain_decay(time - prev_start_time, Self::STRAIN_DECAY_BASE)
    }

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.current_strain *= strain_decay(curr.delta_time, Self::STRAIN_DECAY_BASE);
        self.current_strain += self.calculate_adjusted_difficulty(curr, objects)
            * Self::SKILL_MULTIPLIER;

        self.current_strain
    }

    fn calculate_adjusted_difficulty(
        &self,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let mut num = self.evaluator.evaluate_diff_of(curr, objects, self.has_hidden_mod);

        if self.is_touch_device {
            num = num.powf(0.9);
        }

        if self.is_relax {
            num *= 0.7;
        }

        if self.is_autopilot {
            num *= 0.4;
        }

        num * (0.985 + curr.overall_difficulty().max(0.0).powf(2.0) / 4000.0)
    }

    pub fn difficulty_value_with_total_objects(&self) -> f64 {
        let mut peaks = self.strain_skill_strain_peaks.clone();
        if self.strain_skill_current_section_peak > 0.0 {
            peaks.push(self.strain_skill_current_section_peak);
        }
        let peaks_sum: f64 = peaks.cs_sum();
        let total = self.total_objects as f64;
        let bonus = 0.7
            + 0.1 * (total / 200.0).min(1.0)
            + if self.total_objects > 200 {
                0.2 * ((total - 200.0) / 200.0).min(1.0)
            } else {
                0.0
            };

        peaks_sum * bonus
    }

    pub fn cloned_difficulty_value(&self) -> f64 {
        self.difficulty_value_with_total_objects()
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "function definition needs to stay in-sync with `StrainSkill::difficulty_value`"
    )]
    fn difficulty_value(current_strain_peaks: Vec<f64>) -> f64 {
        current_strain_peaks.cs_sum()
    }

    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        25.0 * f64::powf(difficulty, 2.0)
    }
}
