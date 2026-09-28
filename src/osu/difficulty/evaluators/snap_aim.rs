use crate::{
    osu::difficulty::{context::OsuDifficultyContext, object::OsuDifficultyObject},
    util::difficulty::{milliseconds_to_bpm, reverse_lerp, smootherstep, smoothstep},
};

pub struct SnapAimEvaluator;

impl SnapAimEvaluator {
    /// `high_bpm_bonus` is passed in rather than computed here.
    ///
    /// It depends only on `curr.adjusted_delta_time`, and the aim pair evaluates
    /// this function twice per object whenever the two aims cannot share a
    /// result - so the caller computes it once and hands it to both calls. It is
    /// also two `powf` calls, which is enough to be worth not repeating.
    ///
    /// It is deliberately *not* cached on the difficulty object: object
    /// construction is serial and outside the thread pool, so work moved there
    /// shows up directly in wall clock even when it is taken off a skill that
    /// runs in parallel. Measured, hoisting it to the object cost the `rayon`
    /// build 6% and bought the sequential build 0.7%.
    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        with_slider_travel_distance: bool,
        high_bpm_bonus: f64,
        ctx: &OsuDifficultyContext,
    ) -> f64 {
        if curr.base.is_spinner() || curr.idx <= 1 {
            return 0.0;
        }

        // `previous(0)` == `&diff_objects[idx - 1]` and `previous(2)` ==
        // `&diff_objects[idx - 3]`. `curr.idx <= 1` guarantees `curr.idx - 1`
        // is in bounds, but `curr.idx - 3` requires `curr.idx >= 3`.
        let prev_diff_obj = &diff_objects[curr.idx - 1];

        if prev_diff_obj.base.is_spinner() {
            return 0.0;
        }

        let prev_prev_diff_obj = curr.idx.checked_sub(3).and_then(|i| diff_objects.get(i));

        let num = if with_slider_travel_distance {
            curr.lazy_jump_dist
        } else {
            curr.jump_dist
        };

        let mut num2 = num / curr.adjusted_delta_time;

        if prev_diff_obj.base.is_slider() && with_slider_travel_distance {
            let num3 = prev_diff_obj.lazy_travel_dist + curr.lazy_jump_dist;
            num2 = num2.max(num3 / curr.adjusted_delta_time);
        }

        let num4 = if with_slider_travel_distance {
            prev_diff_obj.lazy_jump_dist
        } else {
            prev_diff_obj.jump_dist
        };

        let num5 = num4 / prev_diff_obj.adjusted_delta_time;
        let mut num6 = num2;

        num6 *= Self::vector_angle_repetition(curr, prev_diff_obj, diff_objects);

        if let (Some(value), Some(value2)) = (curr.angle, prev_diff_obj.angle) {
            let num7 = num2.min(num5);
            let mut num8 = 0.0;

            if curr
                .adjusted_delta_time
                .max(prev_diff_obj.adjusted_delta_time)
                < 1.25
                    * curr
                        .adjusted_delta_time
                        .min(prev_diff_obj.adjusted_delta_time)
            {
                num8 = Self::calc_angle_acuteness(value);
                num8 *=
                    0.08 + 0.92 * (1.0 - num8.min(Self::calc_angle_acuteness(value2).powf(3.0)));
                num8 *=
                    num7 * smootherstep(
                        milliseconds_to_bpm(curr.adjusted_delta_time, Some(2)),
                        300.0,
                        400.0,
                    ) * smootherstep(num, 0.0, 200.0);
            }

            let mut num9 = Self::calc_angle_wideness(value);
            num9 *= 0.25 + 0.75 * (1.0 - num9.min(Self::calc_angle_wideness(value2).powf(3.0)));

            // `adjusted_delta_time.powf(1.45)` is needed for the current object
            // here and for the previous object below. The previous object is the
            // current object of the next call, so both sides read the value that
            // was computed once, on the difficulty object itself.
            let curr_dt_pow = curr.adjusted_delta_time_pow_145;

            let mut val = num / curr_dt_pow;
            let val2 = num4 / prev_diff_obj.adjusted_delta_time_pow_145;

            if prev_diff_obj.base.is_slider() && with_slider_travel_distance {
                let num10 = prev_diff_obj.lazy_travel_dist + curr.lazy_jump_dist;
                val = val.max(num10 / curr_dt_pow);
            }

            num9 *= val.min(val2);

            if let Some(prev_prev_prev) = prev_prev_diff_obj {
                let val3 = prev_prev_prev.base.stacked_pos() - prev_diff_obj.base.stacked_pos();
                let length = val3.length();
                if length < 1.0 {
                    num9 *= 1.0 - 0.55 * f64::from(1.0 - length);
                }
            }

            num6 += (num8 * 2.41).max(num9 * 9.67);

            let rad_110 = 110.0_f64.to_radians();
            let rad_60 = 60.0_f64.to_radians();

            let num11 = num7
                * smootherstep(num, 50.0, 100.0)
                * reverse_lerp(num, 300.0, 100.0).powf(1.8)
                * smootherstep(value, rad_110, rad_60)
                * smootherstep(num4, 50.0, 100.0)
                * reverse_lerp(num4, 300.0, 100.0).powf(1.8)
                * smootherstep(value2, rad_110, rad_60);

            num6 += num11 * 1.02;
        }

        if num5.max(num2) != 0.0 {
            if with_slider_travel_distance {
                num2 = num / curr.adjusted_delta_time;
            }

            let num12 = smoothstep((num5 - num2).abs() / num5.max(num2), 0.0, 1.0);
            let mut num13 = (125.0
                / curr
                    .adjusted_delta_time
                    .min(prev_diff_obj.adjusted_delta_time))
            .min((num5 - num2).abs())
                * num12;

            num13 *= (curr
                .adjusted_delta_time
                .min(prev_diff_obj.adjusted_delta_time)
                / curr
                    .adjusted_delta_time
                    .max(prev_diff_obj.adjusted_delta_time))
            .powf(2.0);

            num6 += num13 * 0.9;
        }

        if curr.base.is_slider() && with_slider_travel_distance {
            let num14 = curr.travel_dist / curr.travel_time;
            num6 += if num14 < 1.0 { num14 } else { num14.powf(0.75) } * 1.5;
        }

        num6 *= ctx.small_circle_bonus;
        num6 * high_bpm_bonus
    }

    pub(crate) fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - 0.03_f64.powf((ms / 1000.0).powf(0.65)))
    }

    fn vector_angle_repetition<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        prev: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        let (Some(curr_angle), Some(prev_angle)) = (curr.angle, prev.angle) else {
            return 1.0;
        };

        let mut num = 0.0;
        let rad_11_25 = 11.25_f64.to_radians();

        for prev_i in diff_objects[..curr.idx].iter().rev().take(6) {
            if curr.adjusted_delta_time.max(prev_i.adjusted_delta_time)
                > 1.1 * curr.adjusted_delta_time.min(prev_i.adjusted_delta_time)
            {
                break;
            }

            if let (Some(norm_angle_curr), Some(norm_angle_prev)) =
                (curr.normalised_vector_angle, prev_i.normalised_vector_angle)
            {
                let val = (norm_angle_curr - norm_angle_prev).abs();
                num += (8.0 * rad_11_25.min(val)).cos();
            }
        }

        let num2 = ((0.5 / num).min(1.0)).powf(2.0);
        let num3 = smootherstep(curr.lazy_jump_dist, 0.0, 100.0);
        let num4 =
            (2.0 * (45.0_f64.to_radians()).min((curr_angle - prev_angle).abs() * num3)).cos();
        let num5 = 1.0 - 0.15 * Self::calc_angle_acuteness(prev_angle) * num4;

        (num5 + (1.0 - num5) * num2 * 0.5 * num3).powf(2.0)
    }

    fn calc_angle_wideness(angle: f64) -> f64 {
        smoothstep(angle, 40.0_f64.to_radians(), 140.0_f64.to_radians())
    }

    pub fn calc_angle_acuteness(angle: f64) -> f64 {
        smoothstep(angle, 140.0_f64.to_radians(), 40.0_f64.to_radians())
    }
}
