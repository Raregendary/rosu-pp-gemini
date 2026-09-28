use crate::osu::difficulty::{context::OsuDifficultyContext, object::OsuDifficultyObject};

pub struct AgilityEvaluator;

impl AgilityEvaluator {
    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        ctx: &OsuDifficultyContext,
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let prev_lazy_travel_dist = diff_objects
            .get(curr.idx.wrapping_sub(1))
            .map_or(0.0, |prev| prev.lazy_travel_dist);

        (prev_lazy_travel_dist + curr.lazy_jump_dist).min(120.0) / 120.0 * 1000.0
            / curr.adjusted_delta_time
            * ctx.small_circle_bonus_pow_15
            * Self::high_bpm_bonus(curr.adjusted_delta_time)
    }

    fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - 0.2_f64.powf(ms / 1000.0))
    }
}
