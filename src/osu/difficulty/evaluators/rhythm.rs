use std::cmp;

use crate::{
    any::difficulty::object::IDifficultyObject,
    osu::difficulty::object::OsuDifficultyObject,
    util::difficulty::{logistic, reverse_lerp, smoothstep_bell_curve_single},
};

pub struct RhythmEvaluator;

impl RhythmEvaluator {
    const HISTORY_TIME_MAX: f64 = 5000.0;
    const HISTORY_OBJECTS_MAX: usize = 32;

    pub fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let mut num = 0.0;
        let num2 = curr.hit_window_great * 0.3;

        let mut island = RhythmIsland::default();
        let mut island2 = RhythmIsland::default();
        let mut list = Vec::<RhythmIsland>::new();

        let mut num3 = 0.0;
        let mut flag = false;

        let num4 = cmp::min(curr.idx, Self::HISTORY_OBJECTS_MAX);
        let mut i = 0;

        while i + 2 < num4 {
            let Some(prev) = curr.previous(i, diff_objects) else { break };
            if curr.start_time - prev.start_time >= Self::HISTORY_TIME_MAX {
                break;
            }
            i += 1;
        }

        let (Some(mut osu_diff_obj), Some(mut osu_diff_obj2)) =
            (curr.previous(i, diff_objects), curr.previous(i + 1, diff_objects))
        else {
            return 1.0;
        };

        for num5 in (1..=i).rev() {
            let Some(osu_diff_obj3) = curr.previous(num5 - 1, diff_objects) else {
                break;
            };

            if !osu_diff_obj3.base.is_spinner() {
                let val = (Self::HISTORY_TIME_MAX - (curr.start_time - osu_diff_obj3.start_time))
                    / Self::HISTORY_TIME_MAX;
                let num6 = ((num4 - num5) as f64 / num4 as f64).min(val);

                let num7 = osu_diff_obj3.delta_time.max(1e-7);
                let num8 = osu_diff_obj.delta_time.max(1e-7);
                let num9 = (num8 - num7).abs();

                if island.delta == i32::MAX {
                    island = RhythmIsland::new(num7 as i32);
                }

                let num10 = num8.max(num7) / num8.min(num7);
                let num11 = (2.0 - num10 / 8.0).clamp(0.0, 1.0);
                let num12 = ((num9 - num2) / num2).clamp(0.0, 1.0);
                let mut num13 = Self::get_effective_difficulty(num10) * num12 * num11;

                if osu_diff_obj.base.is_slider() {
                    let min_jump_time = osu_diff_obj3.min_jump_time;
                    let delta_diff_ratio = min_jump_time.max(num7) / min_jump_time.min(num7);
                    let last_obj_end_delta_time = osu_diff_obj3.last_object_end_delta_time;
                    let end_delta_ratio =
                        last_obj_end_delta_time.max(num7) / last_obj_end_delta_time.min(num7);

                    num13 = Self::get_effective_difficulty(end_delta_ratio)
                        .min(Self::get_effective_difficulty(delta_diff_ratio))
                        .min(num13);
                }

                if num9 < num2 {
                    island.add_delta(num7 as i32);
                }

                if flag {
                    if num9 > num2 {
                        if osu_diff_obj3.base.is_slider() {
                            num13 *= 0.5;
                        }
                        if island.is_similar_polarity(&island2, num2) {
                            num13 *= 0.5;
                        }
                        if osu_diff_obj2.delta_time.max(1e-7) > num8 + num2 && num8 > num7 + num2 {
                            num13 *= 0.125;
                        }
                        if island2.delta_count == island.delta_count {
                            num13 *= 0.5;
                        }
                        if num8 > num7 + num2 {
                            num13 *= 0.65;
                        }

                        let mut flag2 = false;
                        for item in list.iter_mut() {
                            if item.almost_equals(&island, num2) {
                                if island2.almost_equals(&island, num2) {
                                    item.occurrences += 1;
                                }

                                let exponent = logistic(f64::from(island.delta), 58.33, 0.24, Some(2.75));
                                num13 *= (3.0 / item.occurrences as f64)
                                    .min((1.0 / item.occurrences as f64).powf(exponent));
                                flag2 = true;
                                break;
                            }
                        }

                        if !flag2 && island.delta_count > 0 {
                            list.push(island);
                        }

                        num13 *= 1.0
                            - osu_diff_obj.calculate_double_tap_feasibility(Some(osu_diff_obj3))
                                * 0.75;

                        num = if island.delta_count <= 1 {
                            num + 0.7 * num6
                        } else {
                            num + (num13 * num3).sqrt() * num6
                        };

                        num3 = num13;

                        if num8 + num2 < num7 {
                            flag = false;
                        }

                        island2 = island;
                        island = RhythmIsland::new(num7 as i32);
                    }
                } else if num8 > num7 + num2 {
                    flag = true;
                    if osu_diff_obj3.base.is_slider() {
                        num13 *= 0.6;
                    }
                    if osu_diff_obj.base.is_slider() {
                        num13 *= 0.6;
                    }
                    num3 = num13;
                    island = RhythmIsland::new(num7 as i32);
                }

                osu_diff_obj2 = osu_diff_obj;
                osu_diff_obj = osu_diff_obj3;
            }
        }

        num *= reverse_lerp(f64::from(island.delta_count), 22.0, 3.0);

        (4.0 + num * 0.95).sqrt() / 2.0
    }

    #[inline]
    fn get_effective_difficulty(delta_difference_ratio: f64) -> f64 {
        let x = delta_difference_ratio - delta_difference_ratio.trunc();
        1.0 + 26.0 * (0.5_f64).min(smoothstep_bell_curve_single(x))
    }
}

#[derive(Copy, Clone, Debug)]
struct RhythmIsland {
    delta: i32,
    delta_count: i32,
    occurrences: i32,
}

impl Default for RhythmIsland {
    fn default() -> Self {
        Self {
            delta: i32::MAX,
            delta_count: 1,
            occurrences: 1,
        }
    }
}

impl RhythmIsland {
    fn new(delta: i32) -> Self {
        Self {
            delta: delta.max(25),
            delta_count: 1,
            occurrences: 1,
        }
    }

    fn add_delta(&mut self, delta: i32) {
        if self.delta == i32::MAX {
            self.delta = delta.max(25);
        }
        self.delta_count += 1;
    }

    fn is_similar_polarity(&self, other: &Self, epsilon: f64) -> bool {
        if self.delta_count <= 1 || other.delta_count <= 1 {
            return false;
        }

        if f64::from((self.delta - other.delta).abs()) < epsilon {
            return self.delta_count % 2 == other.delta_count % 2;
        }

        false
    }

    fn almost_equals(&self, other: &Self, epsilon: f64) -> bool {
        if f64::from((self.delta - other.delta).abs()) < epsilon {
            return self.delta_count == other.delta_count;
        }

        false
    }
}
