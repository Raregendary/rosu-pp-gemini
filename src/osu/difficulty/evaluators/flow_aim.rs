use crate::{
    any::difficulty::object::IDifficultyObject,
    osu::difficulty::object::OsuDifficultyObject,
    util::difficulty::{smootherstep, smoothstep},
};

use super::snap_aim::SnapAimEvaluator;

pub struct FlowAimEvaluator;

impl FlowAimEvaluator {
    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        with_slider_travel_distance: bool,
    ) -> f64 {
        if curr.base.is_spinner() || curr.idx <= 1 {
            return 0.0;
        }

        let Some(prev_diff_obj) = curr.previous(0, diff_objects) else {
            return 0.0;
        };

        if prev_diff_obj.base.is_spinner() {
            return 0.0;
        }

        let second = curr.previous(1, diff_objects);

        let num = if with_slider_travel_distance {
            curr.lazy_jump_dist
        } else {
            curr.jump_dist
        };

        let num2 = if with_slider_travel_distance {
            prev_diff_obj.lazy_jump_dist
        } else {
            prev_diff_obj.jump_dist
        };

        let mut num3 = num / curr.adjusted_delta_time;

        if prev_diff_obj.base.is_slider() && with_slider_travel_distance {
            let num4 = prev_diff_obj.lazy_travel_dist + curr.lazy_jump_dist;
            num3 = num3.max(num4 / curr.adjusted_delta_time);
        }

        let num5 = num2 / prev_diff_obj.adjusted_delta_time;
        let mut num6 = num3;

        num6 *= curr.small_circle_bonus.sqrt();

        let delta_diff = (curr.adjusted_delta_time.max(prev_diff_obj.adjusted_delta_time)
            - curr.adjusted_delta_time.min(prev_diff_obj.adjusted_delta_time))
            / 50.0;
        num6 *= 1.0 + (0.25_f64).min(delta_diff.powf(4.0));

        if let (Some(angle1), Some(angle2)) = (curr.angle, prev_diff_obj.angle) {
            let num7 = ((angle1 - angle2).abs() / 2.0).sin() * 180.0
                / (curr.adjusted_delta_time * 0.1);
            num6 *= 0.8 + (num7 / 270.0).sqrt();
        }

        let mut num8 = 1.0;
        if curr.idx > 2 {
            if let Some(second_obj) = second {
                let num9 = Self::calculate_overlap_factor(curr, prev_diff_obj);
                let num10 = Self::calculate_overlap_factor(curr, second_obj);
                let num11 = Self::calculate_overlap_factor(prev_diff_obj, second_obj);
                num8 = 1.0 - num9 * num10 * num11;
            }
        }

        if let Some(angle) = curr.angle {
            num6 += num3 * SnapAimEvaluator::calc_angle_acuteness(angle) * num8;
        }

        if num5.max(num3) != 0.0 {
            if with_slider_travel_distance {
                num3 = num / curr.adjusted_delta_time;
            }

            let num12 = smoothstep((num5 - num3).abs() / num5.max(num3), 0.0, 1.0);
            let num13 = (125.0
                / curr
                    .adjusted_delta_time
                    .min(prev_diff_obj.adjusted_delta_time))
            .min((num5 - num3).abs());

            num6 += num13 * num12 * num8 * 0.52;
        }

        if curr.base.is_slider() && with_slider_travel_distance {
            num6 += curr.travel_dist / curr.travel_time;
        }

        num6 = num6.powf(1.45);

        num6 * smootherstep(num, 0.0, 50.0)
    }

    fn calculate_overlap_factor(
        first: &OsuDifficultyObject<'_>,
        second: &OsuDifficultyObject<'_>,
    ) -> f64 {
        let radius = first.radius;
        let dist = f64::from((first.base.stacked_pos() - second.base.stacked_pos()).length());
        (1.0 - ((dist - radius).max(0.0) / radius).powf(2.0)).clamp(0.0, 1.0)
    }
}
