use crate::{
    any::difficulty::object::IDifficultyObject,
    osu::difficulty::object::OsuDifficultyObject,
    util::difficulty::{bpm_to_milliseconds, milliseconds_to_bpm},
};

pub struct SpeedEvaluator;

impl SpeedEvaluator {
    const MIN_SPEED_BONUS: f64 = 200.0; // 200 BPM 1/4th
    const SPEED_BALANCING_FACTOR: f64 = 40.0;

    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let mut adjusted_delta_time = curr.adjusted_delta_time;
        let num = 1.0 - curr.calculate_double_tap_feasibility(curr.next(0, diff_objects));

        adjusted_delta_time /=
            ((adjusted_delta_time / curr.hit_window_great) / 0.93).clamp(0.92, 1.0);

        let mut num2 = 0.0;
        if milliseconds_to_bpm(adjusted_delta_time, None) > Self::MIN_SPEED_BONUS {
            num2 = 0.75
                * ((bpm_to_milliseconds(Self::MIN_SPEED_BONUS, None) - adjusted_delta_time)
                    / Self::SPEED_BALANCING_FACTOR)
                    .powf(2.0);
        }

        (1.0 + num2) * 1000.0 / adjusted_delta_time
            * Self::high_bpm_bonus(curr.adjusted_delta_time)
            * num
    }

    fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - 0.3_f64.powf(ms / 1000.0))
    }
}
