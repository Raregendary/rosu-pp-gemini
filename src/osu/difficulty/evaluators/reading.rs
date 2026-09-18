use std::f64::consts::PI;

use crate::{
    any::difficulty::object::IDifficultyObject,
    osu::difficulty::object::OsuDifficultyObject,
    util::difficulty::{norm, reverse_lerp, smootherstep},
};

pub struct ReadingEvaluator;

impl ReadingEvaluator {
    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        hidden: bool,
    ) -> f64 {
        if curr.base.is_spinner() || curr.idx == 0 {
            return 0.0;
        }

        let next_obj = curr.next(0, diff_objects);
        let velocity = (curr.lazy_jump_dist / curr.adjusted_delta_time).max(1.0);
        let current_visible_object_density =
            Self::retrieve_current_visible_object_density(curr, diff_objects);
        let past_object_difficulty_influence =
            Self::get_past_object_difficulty_influence(curr, diff_objects);
        let constant_angle_nerf_factor =
            Self::get_constant_angle_nerf_factor(curr, diff_objects);

        let num = Self::calculate_density_difficulty(
            next_obj,
            velocity,
            constant_angle_nerf_factor,
            past_object_difficulty_influence,
            current_visible_object_density,
        );

        let num2 = if hidden {
            Self::calculate_hidden_difficulty(
                curr,
                diff_objects,
                past_object_difficulty_influence,
                current_visible_object_density,
                velocity,
                constant_angle_nerf_factor,
            )
        } else {
            0.0
        };

        let num3 = Self::calculate_preempt_difficulty(
            velocity,
            constant_angle_nerf_factor,
            curr.preempt,
        );

        norm(1.5, [num3, num2, num]) * Self::high_bpm_bonus(curr.adjusted_delta_time)
    }

    fn calculate_density_difficulty(
        next_obj: Option<&OsuDifficultyObject<'_>>,
        velocity: f64,
        constant_angle_nerf_factor: f64,
        past_object_difficulty_influence: f64,
        current_visible_object_density: f64,
    ) -> f64 {
        let mut num = current_visible_object_density.sqrt();
        if let Some(next) = next_obj {
            num *= smootherstep(next.lazy_jump_dist, 15.0, 150.0);
        }

        let mut num2 = (past_object_difficulty_influence + num).powf(1.7)
            * 0.4
            * constant_angle_nerf_factor
            * velocity;
        num2 = (num2 - 2.5).max(0.0);

        num2.powf(0.45) * 2.4
    }

    fn calculate_preempt_difficulty(
        velocity: f64,
        constant_angle_nerf_factor: f64,
        preempt: f64,
    ) -> f64 {
        ((500.0 - preempt + (preempt - 500.0).abs()) / 2.0).powf(2.5) / 140_000.0
            * (constant_angle_nerf_factor * velocity)
    }

    fn calculate_hidden_difficulty(
        curr_obj: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        past_object_difficulty_influence: f64,
        current_visible_object_density: f64,
        velocity: f64,
        constant_angle_nerf_factor: f64,
    ) -> f64 {
        let num = curr_obj.preempt.powf(2.2) * 0.01;
        let num2 = (current_visible_object_density + past_object_difficulty_influence).powf(3.3) * 3.0;
        let mut x = (num + num2) * constant_angle_nerf_factor * velocity * 0.01;
        x = x.powf(0.4) * 0.28;

        if let Some(prev) = curr_obj.previous(0, diff_objects) {
            if curr_obj.lazy_jump_dist == 0.0
                && curr_obj.opacity_at(prev.base.start_time, true) == 0.0
                && prev.start_time > curr_obj.start_time - curr_obj.preempt
            {
                x += 700.0000000000001 / curr_obj.adjusted_delta_time.powf(1.5);
            }
        }

        x
    }

    fn get_past_object_difficulty_influence(
        curr_obj: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let mut num = 0.0;
        for i in 0..curr_obj.idx {
            let Some(item) = curr_obj.previous(i, diff_objects) else {
                break;
            };

            if curr_obj.start_time - item.start_time > 3000.0
                || item.start_time < curr_obj.start_time - curr_obj.preempt
            {
                break;
            }

            let mut num2 = curr_obj.opacity_at(item.base.start_time, false);
            num2 *= smootherstep(item.lazy_jump_dist, 15.0, 150.0);
            let time_nerf_factor = Self::get_time_nerf_factor(curr_obj.start_time - item.start_time);
            num2 *= time_nerf_factor;
            num += num2;
        }

        num
    }

    fn retrieve_current_visible_object_density(
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let mut num = 0.0;
        let mut fwd_idx = 0;

        while let Some(next_obj) = curr.next(fwd_idx, diff_objects) {
            if next_obj.start_time - curr.start_time > 3000.0
                || curr.start_time < next_obj.start_time - next_obj.preempt
            {
                break;
            }

            let time_nerf_factor =
                Self::get_time_nerf_factor(next_obj.start_time - curr.start_time);
            num += next_obj.opacity_at(curr.base.start_time, false) * time_nerf_factor;
            fwd_idx += 1;
        }

        num
    }

    fn get_constant_angle_nerf_factor(
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let mut num = 0.0;
        let mut num2 = 0;
        let mut num3 = 0.0;
        let mut osu_diff_obj = curr;
        let mut osu_diff_obj2: Option<&OsuDifficultyObject<'_>> = None;
        let mut osu_diff_obj3: Option<&OsuDifficultyObject<'_>> = None;

        let rad_30 = 30.0_f64.to_radians();

        while num3 < 2000.0 {
            let Some(osu_diff_obj4) = curr.previous(num2, diff_objects) else {
                break;
            };

            let num4 = 1.0 - reverse_lerp(osu_diff_obj4.adjusted_delta_time, 200.0, 2000.0);

            if let (Some(angle4), Some(curr_angle)) = (osu_diff_obj4.angle, curr.angle) {
                let val = (curr_angle - angle4).abs();
                let mut val2 = PI;

                if let (Some(angle_curr_step), Some(obj2), Some(obj3)) =
                    (osu_diff_obj.angle, osu_diff_obj2, osu_diff_obj3)
                {
                    if let (Some(angle2), Some(angle3)) = (obj2.angle, obj3.angle) {
                        val2 = (angle2 - angle4).abs();
                        val2 += (angle3 - angle_curr_step).abs();

                        let mut num5 = 1.0;
                        let min_angle = angle4.min(angle_curr_step) * 180.0 / PI;
                        let max_angle = angle4.max(angle_curr_step) * 180.0 / PI;

                        num5 *= reverse_lerp(min_angle, 20.0, 5.0);
                        num5 *= reverse_lerp(max_angle, 60.0, 120.0);

                        val2 = PI + (0.1 * val2 - PI) * num5;
                    }
                }

                let num6 = smootherstep(osu_diff_obj4.lazy_jump_dist, 0.0, 50.0);
                num += (3.0 * rad_30.min(val.min(val2) * num6)).cos() * num4;
            }

            num3 = curr.start_time - osu_diff_obj4.start_time;
            num2 += 1;
            osu_diff_obj3 = osu_diff_obj2;
            osu_diff_obj2 = Some(osu_diff_obj);
            osu_diff_obj = osu_diff_obj4;
        }

        (2.0 / num).clamp(0.2, 1.0)
    }

    fn get_time_nerf_factor(delta_time: f64) -> f64 {
        (2.0 - delta_time / 1500.0).clamp(0.0, 1.0)
    }

    fn high_bpm_bonus(ms: f64) -> f64 {
        1.0 / (1.0 - 0.8_f64.powf(ms / 1000.0))
    }
}
