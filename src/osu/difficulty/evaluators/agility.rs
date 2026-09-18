use crate::{
    any::difficulty::object::IDifficultyObject,
    osu::difficulty::object::OsuDifficultyObject,
};

pub struct AgilityEvaluator;

impl AgilityEvaluator {
    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let prev_lazy_travel_dist = curr
            .previous(0, diff_objects)
            .map_or(0.0, |prev| prev.lazy_travel_dist);

        (prev_lazy_travel_dist + curr.lazy_jump_dist).min(120.0) / 120.0
            * 1000.0 / curr.adjusted_delta_time
            * curr.small_circle_bonus.powf(1.5)
            * Self::high_bpm_bonus(curr.adjusted_delta_time)
    }

    fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - 0.2_f64.powf(ms / 1000.0))
    }
}
