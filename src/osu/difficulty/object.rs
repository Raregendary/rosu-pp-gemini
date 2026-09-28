use std::borrow::Cow;

use rosu_map::util::Pos;

use crate::{
    any::difficulty::object::{HasStartTime, IDifficultyObject},
    osu::object::{OsuObject, OsuObjectKind},
    util::difficulty::{reverse_lerp, smootherstep},
};

use super::{context::OsuDifficultyContext, scaling_factor::ScalingFactor};

pub struct OsuDifficultyObject<'a> {
    pub idx: usize,
    pub base: &'a OsuObject,
    pub start_time: f64,
    pub delta_time: f64,

    pub adjusted_delta_time: f64,
    pub last_object_end_delta_time: f64,
    pub jump_dist: f64,
    pub lazy_jump_dist: f64,
    pub min_jump_dist: f64,
    pub min_jump_time: f64,
    pub travel_dist: f64,
    pub travel_time: f64,
    pub lazy_end_pos: Option<Pos>,
    pub lazy_travel_dist: f64,
    pub lazy_travel_time: f64,
    pub angle: Option<f64>,
    pub normalised_vector_angle: Option<f64>,

    pub hit_window_great: f64,
    pub preempt: f64,
    pub radius: f64,

    /// `adjusted_delta_time.powf(1.45)`, used twice by
    /// [`SnapAimEvaluator`](super::evaluators::SnapAimEvaluator) for the current
    /// and once for the previous object, so recomputing it per object halves
    /// the number of `powf` calls.
    pub adjusted_delta_time_pow_145: f64,
    /// The exponent of [`Self::calculate_double_tap_feasibility`], which depends
    /// only on `self` and is therefore computed once up front.
    pub double_tap_power: f64,
    /// `smootherstep(lazy_jump_dist, 15.0, 150.0)`.
    ///
    /// The reading evaluator applies this to every object inside the visibility
    /// window, which is a different object on every call, so caching it on the
    /// difficulty object replaces one `smootherstep` per element per call with
    /// one field read.
    pub reading_density_scale: f64,
}

impl<'a> OsuDifficultyObject<'a> {
    pub const NORMALIZED_RADIUS: i32 = 50;

    pub const MIN_DELTA_TIME: f64 = 25.0;
    const MAX_SLIDER_RADIUS: f32 = Self::NORMALIZED_RADIUS as f32 * 2.4;
    const ASSUMED_SLIDER_RADIUS: f32 = Self::NORMALIZED_RADIUS as f32 * 1.8;

    pub fn new(
        hit_object: &'a OsuObject,
        last_object: &'a OsuObject,
        last_diff_obj: Option<&OsuDifficultyObject>,
        last_last_diff_obj: Option<&OsuDifficultyObject>,
        clock_rate: f64,
        idx: usize,
        scaling_factor: &ScalingFactor,
        hit_window_great: f64,
        preempt: f64,
    ) -> Self {
        let delta_time = (hit_object.start_time - last_object.start_time) / clock_rate;
        let start_time = hit_object.start_time / clock_rate;

        let strain_time = delta_time.max(Self::MIN_DELTA_TIME);
        let last_object_end_delta_time = last_diff_obj.map_or(strain_time, |prev| {
            ((hit_object.start_time - prev.base.end_time()) / clock_rate).max(Self::MIN_DELTA_TIME)
        });
        let mut this = Self {
            idx,
            base: hit_object,
            start_time,
            delta_time,
            adjusted_delta_time: strain_time,
            last_object_end_delta_time,
            jump_dist: 0.0,
            lazy_jump_dist: 0.0,
            min_jump_dist: 0.0,
            min_jump_time: 0.0,
            travel_dist: 0.0,
            travel_time: 0.0,
            lazy_end_pos: None,
            lazy_travel_dist: 0.0,
            lazy_travel_time: 0.0,
            angle: None,
            normalised_vector_angle: None,
            hit_window_great,
            preempt,
            radius: scaling_factor.radius,
            adjusted_delta_time_pow_145: 0.0,
            double_tap_power: 0.0,
            reading_density_scale: 0.0,
        };

        this.compute_slider_cursor_pos(scaling_factor.radius);
        this.set_distances(
            last_object,
            last_diff_obj,
            last_last_diff_obj,
            clock_rate,
            scaling_factor,
        );

        // All of these depend on fields that `set_distances` only just filled in.
        this.adjusted_delta_time_pow_145 = this.adjusted_delta_time.powf(1.45);
        this.reading_density_scale = smootherstep(this.lazy_jump_dist, 15.0, 150.0);
        this.double_tap_power = {
            // `delta_time.max(1.0)`, not `adjusted_delta_time`: the two differ
            // whenever the raw delta time is below the minimum strain time.
            let num = this.delta_time.max(1.0);
            let num2 = (num / hit_window_great).min(1.0).powf(5.0);
            let num3 = reverse_lerp(this.lazy_jump_dist, 100.0, 50.0).powf(2.0);

            num3 * (1.0 - num2)
        };

        this
    }

    #[inline]
    pub fn opacity_at(&self, time: f64, hidden: bool, ctx: &OsuDifficultyContext) -> f64 {
        if time > self.base.start_time {
            return 0.0;
        }

        let num = self.base.start_time - ctx.opacity_raw_preempt;

        if hidden {
            let num3 = num + ctx.opacity_fade_in;
            let fade_in = ((time - num) / ctx.opacity_divisor).clamp(0.0, 1.0);
            let fade_out = ((time - num3) / ctx.opacity_fade_out).clamp(0.0, 1.0);
            fade_in.min(1.0 - fade_out)
        } else {
            ((time - num) / ctx.opacity_divisor).clamp(0.0, 1.0)
        }
    }

    pub fn calculate_double_tap_feasibility(&self, next: Option<&Self>) -> f64 {
        let Some(next) = next else { return 0.0 };

        let num = self.delta_time.max(1.0);
        let val = (next.delta_time.max(1.0) - num).abs();
        let x = num / num.max(val);

        // `x == 1.0` gives `1.0 - 1.0.powf(p) == 0.0` for every exponent `p`,
        // including `NaN` and the infinities, so the result is exactly `0.0`
        // and both callers multiply by `1.0 - 0.0 * k`, which is a no-op. The
        // `powf` is therefore unobservable and can be skipped.
        //
        // The test is on `x` rather than on `val <= num` so that a `NaN`
        // denominator still falls through to the original expression instead of
        // silently taking the shortcut.
        //
        // This is the common case, not a rare one: `x == 1.0` whenever the
        // neighbouring delta time is within a factor of two, which covers every
        // stream that speeds up or holds a steady rate.
        #[allow(
            clippy::float_cmp,
            reason = "exact equality is the condition being tested for; an approximate one would change the result"
        )]
        if x == 1.0 {
            return 0.0;
        }

        1.0 - x.powf(self.double_tap_power)
    }

    fn calculate_angle(curr_pos: Pos, last_pos: Pos, last_last_pos: Pos) -> f64 {
        let v1 = last_last_pos - last_pos;
        let v2 = curr_pos - last_pos;
        let dot = v1.dot(v2);
        let det = v1.x * v2.y - v1.y * v2.x;
        f64::from(det).atan2(f64::from(dot)).abs()
    }

    fn calculate_slider_angle(
        curr_pos: Pos,
        last_object: &OsuObject,
        last_diff_obj: &OsuDifficultyObject,
        end_cursor_pos: Pos,
    ) -> f64 {
        let end_cursor_pos_last = Self::get_end_cursor_pos(last_diff_obj);
        let mut last_last_cursor_pos = end_cursor_pos;

        if let OsuObjectKind::Slider(ref slider) = last_object.kind {
            if last_diff_obj.travel_dist > 0.0 {
                if slider.nested_objects.len() >= 2 {
                    last_last_cursor_pos = slider.nested_objects[slider.nested_objects.len() - 2]
                        .pos
                        + last_object.stack_offset;
                } else {
                    last_last_cursor_pos = last_object.stacked_pos();
                }
            }
        }

        Self::calculate_angle(curr_pos, end_cursor_pos_last, last_last_cursor_pos)
    }

    fn set_distances(
        &mut self,
        last_object: &OsuObject,
        last_diff_obj: Option<&OsuDifficultyObject>,
        last_last_diff_obj: Option<&OsuDifficultyObject>,
        clock_rate: f64,
        scaling_factor: &ScalingFactor,
    ) {
        if let OsuObjectKind::Slider(ref slider) = self.base.kind {
            self.travel_dist =
                self.lazy_travel_dist * (1.0_f64).max((slider.repeat_count() as f64).powf(0.3));

            self.travel_time =
                (self.lazy_travel_time / clock_rate).max(OsuDifficultyObject::MIN_DELTA_TIME);
        }

        if self.base.is_spinner() || last_object.is_spinner() {
            return;
        }

        let scaling_factor_val = scaling_factor.factor;

        let last_cursor_pos = if let Some(last_diff_obj) = last_diff_obj {
            Self::get_end_cursor_pos(last_diff_obj)
        } else {
            last_object.stacked_pos()
        };

        self.jump_dist = f64::from(
            (last_object.stacked_pos() * scaling_factor_val
                - self.base.stacked_pos() * scaling_factor_val)
                .length(),
        );

        self.lazy_jump_dist = f64::from(
            (self.base.stacked_pos() * scaling_factor_val - last_cursor_pos * scaling_factor_val)
                .length(),
        );
        self.min_jump_time = self.adjusted_delta_time;
        self.min_jump_dist = self.lazy_jump_dist;

        let Some(last_diff_obj) = last_diff_obj else {
            return;
        };

        if let OsuObjectKind::Slider(ref last_slider) = last_object.kind {
            let last_travel_time = (last_diff_obj.lazy_travel_time / clock_rate)
                .max(OsuDifficultyObject::MIN_DELTA_TIME);
            self.min_jump_time = (self.adjusted_delta_time - last_travel_time)
                .max(OsuDifficultyObject::MIN_DELTA_TIME);

            let tail_pos = last_slider.tail().map_or(last_object.pos, |tail| tail.pos);
            let stacked_tail_pos = tail_pos + last_object.stack_offset;

            let tail_jump_dist =
                (stacked_tail_pos - self.base.stacked_pos()).length() * scaling_factor_val;

            let diff = f64::from(
                OsuDifficultyObject::MAX_SLIDER_RADIUS - OsuDifficultyObject::ASSUMED_SLIDER_RADIUS,
            );

            let min = f64::from(tail_jump_dist - OsuDifficultyObject::MAX_SLIDER_RADIUS);
            self.min_jump_dist = ((self.lazy_jump_dist - diff).min(min)).max(0.0);
        }

        let Some(last_last_diff_obj) = last_last_diff_obj else {
            return;
        };

        if !last_last_diff_obj.base.is_spinner() {
            let mut val = last_cursor_pos;
            if last_object.is_slider() && last_diff_obj.travel_dist > 0.0 {
                val = last_object.stacked_pos();
            }

            let end_cursor_pos = Self::get_end_cursor_pos(last_last_diff_obj);
            let val3 = Self::calculate_angle(self.base.stacked_pos(), val, end_cursor_pos);
            let val4 = Self::calculate_slider_angle(
                self.base.stacked_pos(),
                last_object,
                last_diff_obj,
                end_cursor_pos,
            );
            let val5 = self.base.stacked_pos() - val;
            self.normalised_vector_angle =
                Some(f64::atan2(f64::from(val5.y.abs()), f64::from(val5.x.abs())));
            self.angle = Some(val3.min(val4));
        }
    }

    pub fn compute_slider_cursor_pos(&mut self, radius: f64) {
        const TAIL_LENIENCY: f64 = -36.0;

        let OsuObjectKind::Slider(ref slider) = self.base.kind else {
            return;
        };

        if self.lazy_end_pos.is_some() {
            return;
        }

        let pos = self.base.pos;
        let stack_offset = self.base.stack_offset;
        let start_time = self.base.start_time;
        let duration = slider.end_time - start_time;

        let mut nested_objects = Cow::Borrowed(slider.nested_objects.as_slice());

        let mut tracking_end_time =
            (start_time + duration + TAIL_LENIENCY).max(start_time + duration / 2.0);

        let last_real_tick = nested_objects
            .iter()
            .enumerate()
            .rfind(|(_, nested)| nested.is_tick());

        if let Some((idx, last_real_tick)) =
            last_real_tick.filter(|(_, tick)| tick.start_time > tracking_end_time)
        {
            tracking_end_time = last_real_tick.start_time;

            // * When the last tick falls after the tracking end time, we need to re-sort the nested objects
            // * based on time. This creates a somewhat weird ordering which is counter to how a user would
            // * understand the slider, but allows a zero-diff with known diffcalc output.
            // *
            // * To reiterate, this is definitely not correct from a difficulty calculation perspective
            // * and should be revisited at a later date (likely by replacing this whole code with the commented
            // * version above).
            nested_objects.to_mut()[idx..].rotate_left(1);
        }

        self.lazy_travel_time = tracking_end_time - start_time;

        let nested_objects = nested_objects.as_ref();

        let span_duration = duration / slider.span_count;

        let mut end_time_min = self.lazy_travel_time / span_duration;

        if end_time_min % 2.0 >= 1.0 {
            end_time_min = 1.0 - end_time_min % 1.0;
        } else {
            end_time_min %= 1.0;
        }

        let mut lazy_end_pos = pos + stack_offset + slider.path.position_at(end_time_min);

        let mut curr_cursor_pos = pos + stack_offset;
        let scaling_factor = f64::from(OsuDifficultyObject::NORMALIZED_RADIUS) / radius;

        for (curr_movement_obj, i) in nested_objects.iter().zip(1..) {
            let mut curr_movement = curr_movement_obj.pos + stack_offset - curr_cursor_pos;
            let mut curr_movement_len = scaling_factor * f64::from(curr_movement.length());
            let mut required_movement = f64::from(OsuDifficultyObject::ASSUMED_SLIDER_RADIUS);

            if i == nested_objects.len() {
                let lazy_movement = lazy_end_pos - curr_cursor_pos;

                if lazy_movement.length() < curr_movement.length() {
                    curr_movement = lazy_movement;
                }

                curr_movement_len = scaling_factor * f64::from(curr_movement.length());
            } else if curr_movement_obj.is_repeat() {
                required_movement = f64::from(OsuDifficultyObject::NORMALIZED_RADIUS);
            }

            if curr_movement_len > required_movement {
                curr_cursor_pos += curr_movement
                    * ((curr_movement_len - required_movement) / curr_movement_len) as f32;
                curr_movement_len *= (curr_movement_len - required_movement) / curr_movement_len;
                self.lazy_travel_dist += curr_movement_len;
            }

            if i == nested_objects.len() {
                lazy_end_pos = curr_cursor_pos;
            }
        }

        self.lazy_end_pos = Some(lazy_end_pos);
    }

    const fn get_end_cursor_pos(hit_object: &OsuDifficultyObject) -> Pos {
        if let Some(lazy_end_pos) = hit_object.lazy_end_pos {
            lazy_end_pos
        } else {
            hit_object.base.stacked_pos()
        }
    }
}

impl IDifficultyObject for OsuDifficultyObject<'_> {
    type DifficultyObjects = [Self];

    fn idx(&self) -> usize {
        self.idx
    }
}

impl HasStartTime for OsuDifficultyObject<'_> {
    fn start_time(&self) -> f64 {
        self.start_time
    }
}
