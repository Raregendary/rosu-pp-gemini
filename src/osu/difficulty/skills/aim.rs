use crate::{
    any::difficulty::object::IDifficultyObject,
    model::mods::GameMods,
    osu::difficulty::{
        context::OsuDifficultyContext,
        evaluators::{AgilityEvaluator, FlowAimEvaluator, SnapAimEvaluator},
        object::OsuDifficultyObject,
    },
    util::{
        difficulty::{logistic, logistic_exp, norm},
        float_ext::FloatExt,
    },
};

use super::scratch;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StrainPeak {
    pub value: f64,
    pub section_length: f64,
}

impl StrainPeak {
    pub fn new(value: f64, section_length: f64) -> Self {
        Self {
            value,
            section_length: section_length.round(),
        }
    }
}

/// A strain peak carrying the position it held before sorting.
///
/// The position is what turns the sort comparator into a strict total order,
/// which is what lets [`Aim::difficulty_value`] reproduce a stable sort with
/// `sort_unstable_by`. See the comment there.
#[derive(Copy, Clone)]
pub(super) struct SortablePeak {
    value: f64,
    section_length: f64,
    order: u32,
}

impl From<StrainPeak> for SortablePeak {
    fn from(peak: StrainPeak) -> Self {
        Self {
            value: peak.value,
            section_length: peak.section_length,
            order: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Aim {
    include_sliders: bool,
    current_strain: f64,
    current_section_peak: f64,
    current_section_begin: f64,
    current_section_end: f64,
    total_length: f64,
    strain_peaks: Vec<StrainPeak>,
    queued_strains: Vec<(f64, f64)>,
    peaks_finalised: bool,
    object_difficulties: Vec<f64>,
    slider_strains: Vec<f64>,
    timeline_peaks: Vec<f64>,
    timeline_section_end: f64,
    timeline_section_peak: f64,
    decay_weight: f64,
    max_section_length: f64,
    max_stored_length: f64,
    is_touch_device: bool,
    is_relax: bool,
    is_autopilot: bool,
    ctx: OsuDifficultyContext,
}

impl Aim {
    /// Runs [`Self::process`] over the first `take` objects.
    ///
    /// Exists so that [`OsuSkills::process_all`](super::OsuSkills::process_all)
    /// can hand whole skills to a thread pool: each skill is sequential in the
    /// objects, so spreading *skills* is the only parallelism available.
    ///
    /// `objects` stays the full list rather than being truncated to `take`,
    /// because evaluators look at neighbouring objects and a shortened slice
    /// would change the last processed object's difficulty.
    //
    // Single-aim entry point. `OsuSkills::process_all` only reaches it in the
    // `rayon` build, where the two aims are spread as two tasks; the sequential
    // build goes through `process_pair_all` instead.
    #[allow(dead_code, reason = "only used by the `rayon` build")]
    pub fn process_all(&mut self, objects: &[OsuDifficultyObject<'_>], take: usize) {
        for curr in objects.iter().take(take) {
            self.process(curr, objects);
        }
    }

    /// Runs the `include_sliders` / `!include_sliders` aim pair over the first
    /// `take` objects.
    ///
    /// The pair is the unit of work, not the single aim: see
    /// [`Self::evaluate_pair`].
    #[allow(dead_code, reason = "only used without the `rayon` feature")]
    pub fn process_pair_all(
        &mut self,
        no_sliders: &mut Aim,
        objects: &[OsuDifficultyObject<'_>],
        take: usize,
    ) {
        for curr in objects.iter().take(take) {
            self.process_pair(no_sliders, curr, objects);
        }
    }

    pub fn new(mods: &GameMods, include_sliders: bool, ctx: OsuDifficultyContext) -> Self {
        let decay_weight = 0.9;
        let max_section_length = 400.0;
        let max_stored_length = 11.0 / (1.0 - decay_weight);

        Self {
            include_sliders,
            current_strain: 0.0,
            current_section_peak: 0.0,
            current_section_begin: 0.0,
            current_section_end: 0.0,
            total_length: 0.0,
            strain_peaks: Vec::new(),
            queued_strains: Vec::new(),
            peaks_finalised: false,
            object_difficulties: Vec::new(),
            slider_strains: Vec::new(),
            timeline_peaks: Vec::new(),
            timeline_section_end: 0.0,
            timeline_section_peak: 0.0,
            decay_weight,
            max_section_length,
            max_stored_length,
            is_touch_device: mods.td(),
            is_relax: mods.rx(),
            is_autopilot: mods.ap(),
            ctx,
        }
    }

    pub fn with_capacity(
        mods: &GameMods,
        include_sliders: bool,
        total_objects: usize,
        ctx: OsuDifficultyContext,
    ) -> Self {
        let mut this = Self::new(mods, include_sliders, ctx);

        // Every one of these grows by at most one entry per processed object, so
        // reserving up front avoids the repeated reallocation (and copying) that
        // a growing `Vec` would otherwise perform on every calculation.
        this.strain_peaks = Vec::with_capacity(64);
        this.queued_strains = Vec::with_capacity(64);
        this.object_difficulties = Vec::with_capacity(total_objects);
        this.slider_strains = Vec::with_capacity(total_objects);
        this.timeline_peaks = Vec::with_capacity(total_objects / 4 + 1);

        this
    }

    fn strain_decay(ms: f64) -> f64 {
        0.2_f64.powf(ms / 1000.0)
    }

    fn calculate_initial_strain(
        &self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let prev_start_time = curr.previous(0, diff_objects).map_or(0.0, |p| p.start_time);
        self.current_strain * Self::strain_decay(time - prev_start_time)
    }

    #[allow(dead_code, reason = "single-aim path; see `process`")]
    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        if self.is_autopilot {
            return 0.0;
        }

        let adjusted = self.calculate_adjusted_difficulty(curr, diff_objects);
        self.strain_value_at_from(curr, adjusted)
    }

    /// The tail of [`Self::strain_value_at`], with the adjusted difficulty
    /// already evaluated.
    ///
    /// Split out so [`Self::process_pair`] can hand in the value it computed for
    /// both aims at once. The arithmetic and its order are unchanged.
    fn strain_value_at_from(&mut self, curr: &OsuDifficultyObject<'_>, adjusted: f64) -> f64 {
        if self.is_autopilot {
            return 0.0;
        }

        let num = Self::strain_decay(curr.adjusted_delta_time);
        self.current_strain *= num;
        self.current_strain += adjusted * (1.0 - num);

        if curr.base.is_slider() {
            self.slider_strains.push(self.current_strain);
        }

        self.current_strain
    }

    fn calculate_adjusted_difficulty(
        &self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let snap_difficulty = SnapAimEvaluator::evaluate_diff_of(
            curr,
            diff_objects,
            self.include_sliders,
            Self::snap_high_bpm_bonus(curr),
            &self.ctx,
        ) * 70.9;
        let agility_difficulty = AgilityEvaluator::evaluate_diff_of(
            curr,
            diff_objects,
            Self::agility_high_bpm_bonus(curr),
            &self.ctx,
        ) * 2.35;
        let flow_difficulty =
            FlowAimEvaluator::evaluate_diff_of(curr, diff_objects, self.include_sliders, &self.ctx)
                * 242.0;

        let num = self.calculate_total_value(snap_difficulty, agility_difficulty, flow_difficulty);

        num * self.ctx.aim_od_factor
    }

    #[inline]
    fn snap_high_bpm_bonus(curr: &OsuDifficultyObject<'_>) -> f64 {
        SnapAimEvaluator::high_bpm_bonus(curr.adjusted_delta_time)
    }

    #[inline]
    fn agility_high_bpm_bonus(curr: &OsuDifficultyObject<'_>) -> f64 {
        AgilityEvaluator::high_bpm_bonus(curr.adjusted_delta_time)
    }

    fn calculate_total_value(
        &self,
        mut snap_difficulty: f64,
        agility_difficulty: f64,
        mut flow_difficulty: f64,
    ) -> f64 {
        let mut num = norm(1.2, [snap_difficulty, agility_difficulty]);
        let num2 = Self::calculate_snap_flow_probability(flow_difficulty / num);
        let num3 = 1.0 - num2;

        if self.is_touch_device {
            snap_difficulty = snap_difficulty.powf(0.89);
            num = norm(1.2, [snap_difficulty, agility_difficulty]);
        }

        if self.is_relax {
            num *= 0.75;
            flow_difficulty *= 0.6;
        }

        (num * num2 + flow_difficulty * num3) * 1.12
    }

    fn calculate_snap_flow_probability(ratio: f64) -> f64 {
        if ratio == 0.0 {
            return 0.0;
        }
        if ratio.is_nan() {
            return 1.0;
        }
        logistic_exp(-7.27 * ratio.ln(), None)
    }

    #[allow(dead_code, reason = "single-aim path; see `process`")]
    pub fn process(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        let adjusted = if self.is_autopilot {
            0.0
        } else {
            self.calculate_adjusted_difficulty(curr, diff_objects)
        };

        self.process_from(curr, diff_objects, adjusted);
    }

    /// [`Self::process`] with the adjusted difficulty already evaluated.
    fn process_from(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
        adjusted: f64,
    ) {
        if curr.idx == 0 {
            self.timeline_section_end = (curr.start_time / 400.0).ceil() * 400.0;
        }

        while curr.start_time > self.timeline_section_end {
            self.timeline_peaks.push(self.timeline_section_peak);
            self.timeline_section_peak =
                self.calculate_initial_strain(self.timeline_section_end, curr, diff_objects);
            self.timeline_section_end += 400.0;
        }

        if curr.idx == 0 {
            self.current_section_begin = curr.start_time;
            self.current_section_end = self.current_section_begin + self.max_section_length;
            self.current_section_peak = self.strain_value_at_from(curr, adjusted);
            self.object_difficulties.push(self.current_section_peak);
            self.timeline_section_peak = self.current_section_peak;
            return;
        }

        self.backfill_peaks(curr, diff_objects);

        let num = self.strain_value_at_from(curr, adjusted);
        self.object_difficulties.push(num);
        self.timeline_section_peak = f64::max(self.timeline_section_peak, num);

        if num > self.current_section_peak {
            self.queued_strains.clear();
            self.save_current_peak(curr.start_time - self.current_section_begin);
            self.current_section_begin = curr.start_time;
            self.current_section_end = self.current_section_begin + self.max_section_length;
            self.current_section_peak = num;
        } else {
            while let Some(last) = self.queued_strains.last() {
                if last.0 >= num {
                    break;
                }
                self.queued_strains.pop();
            }
            self.queued_strains.push((num, curr.start_time));
        }
    }

    /// Runs both aims over a single object, sharing the evaluator work that
    /// does not depend on `include_sliders`.
    ///
    /// `self` must be the `include_sliders` aim and `no_sliders` the other one;
    /// they are built that way by [`OsuSkills::new`](super::OsuSkills::new) and
    /// are never mixed up.
    pub fn process_pair(
        &mut self,
        no_sliders: &mut Aim,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        debug_assert!(self.include_sliders);
        debug_assert!(!no_sliders.include_sliders);

        if self.is_autopilot {
            // `strain_value_at` returns `0.0` before reaching any evaluator, so
            // there is nothing to compute and both aims stay at zero strain.
            self.process_from(curr, diff_objects, 0.0);
            no_sliders.process_from(curr, diff_objects, 0.0);
            return;
        }

        let (with, without) = self.evaluate_pair(curr, diff_objects);

        self.process_from(curr, diff_objects, with);
        no_sliders.process_from(curr, diff_objects, without);
    }

    /// The two [`Self::calculate_adjusted_difficulty`] results, evaluated with
    /// the duplicated work removed.
    ///
    /// The two aims differ only by the `with_slider_travel_distance` flag, and
    /// that flag only reaches three places in the two evaluators it feeds:
    ///
    /// * `AgilityEvaluator` does not read the flag at all, so its result is
    ///   literally the same expression twice.
    /// * In `SnapAimEvaluator` and `FlowAimEvaluator` the flag selects between
    ///   `lazy_jump_dist` and `jump_dist` (on the current and the previous
    ///   object) and gates three `if` bodies that are all `is_slider()` checks.
    ///
    /// So whenever neither object is a slider *and* the two jump distances
    /// agree, the `false` path is textually the `true` path with equal
    /// operands - every `&& with_slider_travel_distance` body is skipped and
    /// `num`/`num4` are the same values. That case is evaluated once and the
    /// single result handed to both aims.
    ///
    /// For a non-slider previous object `get_end_cursor_pos` returns
    /// `stacked_pos()`, so `lazy_jump_dist` and `jump_dist` are the same length
    /// of a negated difference - bit-identical, not merely close. The equality
    /// is still tested explicitly so that a change to the difficulty object's
    /// distance computation can only ever make this fall back, never silently
    /// produce a different value.
    fn evaluate_pair(
        &self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> (f64, f64) {
        // Evaluated once here rather than inside the evaluators, because both
        // aim variants multiply by the same value and the non-shared case below
        // runs each evaluator twice.
        let snap_bonus = Self::snap_high_bpm_bonus(curr);
        let agility_bonus = Self::agility_high_bpm_bonus(curr);

        let agility =
            AgilityEvaluator::evaluate_diff_of(curr, diff_objects, agility_bonus, &self.ctx) * 2.35;

        if Self::sharable_between_aims(curr, diff_objects) {
            let snap =
                SnapAimEvaluator::evaluate_diff_of(curr, diff_objects, true, snap_bonus, &self.ctx)
                    * 70.9;
            let flow =
                FlowAimEvaluator::evaluate_diff_of(curr, diff_objects, true, &self.ctx) * 242.0;

            // Identical inputs, identical `is_touch_device` / `is_relax`, so
            // identical output: there is no reason to run it twice.
            let total = self.calculate_total_value(snap, agility, flow) * self.ctx.aim_od_factor;

            return (total, total);
        }

        let snap =
            SnapAimEvaluator::evaluate_diff_of(curr, diff_objects, true, snap_bonus, &self.ctx)
                * 70.9;
        let flow = FlowAimEvaluator::evaluate_diff_of(curr, diff_objects, true, &self.ctx) * 242.0;
        let with = self.calculate_total_value(snap, agility, flow) * self.ctx.aim_od_factor;

        let snap =
            SnapAimEvaluator::evaluate_diff_of(curr, diff_objects, false, snap_bonus, &self.ctx)
                * 70.9;
        let flow = FlowAimEvaluator::evaluate_diff_of(curr, diff_objects, false, &self.ctx) * 242.0;
        let without = self.calculate_total_value(snap, agility, flow) * self.ctx.aim_od_factor;

        (with, without)
    }

    /// Whether the two aim evaluators are guaranteed to agree for this object.
    ///
    /// Deliberately conservative: a `false` result only costs the duplicated
    /// work that already happens today.
    #[allow(
        clippy::float_cmp,
        reason = "the distances are equal bit-for-bit here, and the check exists to prove it rather than to approximate it"
    )]
    fn sharable_between_aims(
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> bool {
        // Below `idx == 2` both evaluators return `0.0` before reading anything
        // flag dependent, and `idx == 0` has no previous object to compare to.
        if curr.idx <= 1 {
            return true;
        }

        let Some(prev) = diff_objects.get(curr.idx - 1) else {
            return true;
        };

        !curr.base.is_slider()
            && !prev.base.is_slider()
            && curr.lazy_jump_dist == curr.jump_dist
            && prev.lazy_jump_dist == prev.jump_dist
    }

    fn backfill_peaks(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        while curr.start_time > self.current_section_end {
            self.save_current_peak(self.current_section_end - self.current_section_begin);
            self.current_section_begin = self.current_section_end;

            if !self.queued_strains.is_empty() {
                let (val, num) = self.queued_strains.remove(0);
                self.current_section_end = num + self.max_section_length;
                self.start_new_section_from(self.current_section_begin, curr, diff_objects);
                self.current_section_peak = self.current_section_peak.max(val);
            } else {
                self.current_section_end = self.current_section_begin + self.max_section_length;
                self.start_new_section_from(self.current_section_begin, curr, diff_objects);
            }
        }
    }

    fn save_current_peak(&mut self, section_length: f64) {
        let peak = StrainPeak::new(self.current_section_peak, section_length);
        let idx = self
            .strain_peaks
            .binary_search_by(|p| p.value.total_cmp(&peak.value).reverse())
            .unwrap_or_else(|e| e);
        self.strain_peaks.insert(idx, peak);

        self.total_length += peak.section_length;

        while self.total_length > self.max_stored_length * self.max_section_length {
            let last_len = self.strain_peaks.last().unwrap().section_length;
            self.total_length -= last_len;
            self.strain_peaks.pop();
        }
    }

    fn start_new_section_from(
        &mut self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        self.current_section_peak = self.calculate_initial_strain(time, curr, diff_objects);
    }

    /// Finalises the stored strain peaks in place.
    ///
    /// [`Self::difficulty_value`] deliberately does not use this: `eval` may run
    /// more than once over the same skill (gradual difficulty), so finalising
    /// must not mutate shared state.
    #[expect(dead_code, reason = "kept for the in-place finalisation use case")]
    pub fn get_current_strain_peaks(&mut self) -> &[StrainPeak] {
        if !self.peaks_finalised {
            self.save_current_peak(self.current_section_end - self.current_section_begin);
            self.peaks_finalised = true;
        }
        &self.strain_peaks
    }

    pub fn into_current_strain_peaks(mut self) -> Vec<f64> {
        self.timeline_peaks.push(self.timeline_section_peak);
        self.timeline_peaks
    }

    /// The strain peaks in their finalised form, without mutating `self`.
    ///
    /// This mirrors what [`Self::get_current_strain_peaks`] and
    /// [`Self::save_current_peak`] would do, but on a copy. `eval` runs once per
    /// hit object while calculating gradual difficulty, so finalising in place
    /// would both mutate shared state and force callers to clone the whole
    /// skill.
    #[expect(
        dead_code,
        reason = "kept as the readable form of `finalised_strain_peaks_into`"
    )]
    fn finalised_strain_peaks(&self) -> Vec<StrainPeak> {
        let peaks: Vec<SortablePeak> = {
            let mut buffer = Vec::new();
            self.finalised_strain_peaks_into(&mut buffer);
            buffer
        };

        peaks
            .into_iter()
            .map(|peak| StrainPeak::new(peak.value, peak.section_length))
            .collect()
    }

    #[allow(dead_code)]
    pub fn cloned_difficulty_value(&self) -> f64 {
        self.difficulty_value()
    }

    pub fn difficulty_value(&self) -> f64 {
        // `eval` runs once per hit object while calculating gradual difficulty,
        // so the whole reduction works inside one reusable buffer and never
        // hands a `Vec` back to the caller.
        scratch::with_strain_peaks(|list| {
            self.finalised_strain_peaks_into(list);
            list.retain(|p| p.value > 0.0);

            let reduced_section_time = 4000.0;
            let mut num = 0.0;
            let mut num2 = 0;

            while num2 < list.len() {
                if num >= reduced_section_time {
                    break;
                }

                let strain_peak = list[num2];
                let mut num3 = 0.0;
                while num3 < strain_peak.section_length {
                    let clamped = ((num + num3) / reduced_section_time).clamp(0.0, 1.0);
                    let num4 = (1.0 + (10.0 - 1.0) * clamped).log10();
                    // Goes through `StrainPeak::new` so the section length is
                    // rounded exactly the way every other peak is.
                    let reduced = StrainPeak::new(
                        strain_peak.value * (0.727 + (1.0 - 0.727) * num4),
                        20.0_f64.min(strain_peak.section_length - num3),
                    );
                    list.push(SortablePeak::from(reduced));
                    num3 += 20.0;
                }

                num += strain_peak.section_length;
                num2 += 1;
            }

            // Same elements in the same order as `list.into_iter().skip(num2)`,
            // but without allocating a second vector.
            list.drain(..num2);

            for (i, peak) in list.iter_mut().enumerate() {
                peak.order = i as u32;
            }

            // * Ordering, not just sorting, has to match `sort_by` here. A stable
            // * sort on `value` alone yields descending `value` with equal
            // * values left in their original relative order, so a plain
            // * `sort_unstable_by(|a, b| b.value.total_cmp(&a.value))` would be a
            // * different permutation - and `StrainPeak` carries
            // * `section_length`, which feeds the running exponent below, so
            // * that difference is observable.
            // *
            // * Stamping the pre-sort position onto each peak turns the
            // * comparator into a *strict total order*: descending by `value`,
            // * ties broken by ascending original position. That describes
            // * exactly the arrangement a stable sort produces, and a correct
            // * sort under a strict total order has only one possible answer, so
            // * the unstable sort yields the identical sequence.
            // *
            // * The other reason to do this: `sort_by` is a merge sort, and it
            // * allocates a temporary buffer proportional to the slice length on
            // * every call. `difficulty_value` runs once per hit object while
            // * calculating gradual difficulty, and that buffer alone was 82% of
            // * all allocations in a playthrough.
            list.sort_unstable_by(|a, b| {
                b.value
                    .total_cmp(&a.value)
                    .then_with(|| a.order.cmp(&b.order))
            });

            let mut sum = 0.0;
            let mut exponent = 0.0;

            // `decay_weight.powf(exponent)` of one iteration is the very same
            // value as `decay_weight.powf(next)` of the previous one, so the
            // previous result is carried over instead of being computed twice.
            // The initial value is `powf`ed as well rather than assumed to be
            // `1.0` so that the result stays bit-identical regardless of the
            // platform's `pow`.
            let mut prev_decay = self.decay_weight.powf(exponent);

            for reduced_strain_peak in list.iter() {
                let next = exponent + reduced_strain_peak.section_length / self.max_section_length;
                let decay = self.decay_weight.powf(next);

                sum += reduced_strain_peak.value * (prev_decay - decay);
                exponent = next;
                prev_decay = decay;
            }

            sum / (1.0 - self.decay_weight)
        })
    }

    /// Writes the strain peaks in their finalised form into `out`, mirroring
    /// [`Self::get_current_strain_peaks`] and [`Self::save_current_peak`] but
    /// without mutating `self` and without allocating.
    ///
    /// [`Self::difficulty_value`] deliberately does not use the in-place
    /// variant: `eval` may run more than once over the same skill (gradual
    /// difficulty), so finalising in place would mutate shared state.
    fn finalised_strain_peaks_into(&self, out: &mut Vec<SortablePeak>) {
        out.clear();
        out.extend(self.strain_peaks.iter().copied().map(SortablePeak::from));

        if self.peaks_finalised {
            return;
        }

        let peak = SortablePeak::from(StrainPeak::new(
            self.current_section_peak,
            self.current_section_end - self.current_section_begin,
        ));

        let idx = out
            .binary_search_by(|p| p.value.total_cmp(&peak.value).reverse())
            .unwrap_or_else(|e| e);
        out.insert(idx, peak);

        let mut total_length = self.total_length + peak.section_length;

        while total_length > self.max_stored_length * self.max_section_length {
            total_length -= out.last().unwrap().section_length;
            out.pop();
        }
    }

    pub fn count_top_weighted_strains(&self, difficulty_value: f64) -> f64 {
        if self.object_difficulties.is_empty() {
            return 0.0;
        }
        let consistent_top_strain = difficulty_value * (1.0 - self.decay_weight);
        if FloatExt::eq(consistent_top_strain, 0.0) {
            return self.object_difficulties.len() as f64;
        }
        self.object_difficulties
            .iter()
            .map(|s| 1.1 / (1.0 + (-10.0 * (s / consistent_top_strain - 0.88)).exp()))
            .sum()
    }

    pub fn count_top_weighted_sliders(&self, difficulty_value: f64) -> f64 {
        if self.slider_strains.is_empty() {
            return 0.0;
        }
        let consistent_top_strain = difficulty_value * (1.0 - self.decay_weight);
        if FloatExt::eq(consistent_top_strain, 0.0) {
            return 0.0;
        }
        self.slider_strains
            .iter()
            .map(|s| logistic(*s / consistent_top_strain, 0.88, 10.0, Some(1.1)))
            .sum()
    }

    pub fn get_difficult_sliders(&self) -> f64 {
        if self.slider_strains.is_empty() {
            return 0.0;
        }
        let max_slider_strain = self.slider_strains.iter().copied().fold(0.0, f64::max);
        if FloatExt::eq(max_slider_strain, 0.0) {
            return 0.0;
        }
        self.slider_strains
            .iter()
            .copied()
            .map(|strain| 1.0 / (1.0 + (-(strain / max_slider_strain * 12.0 - 6.0)).exp()))
            .sum()
    }

    #[allow(dead_code)]
    pub fn difficulty_to_performance(difficulty: f64) -> f64 {
        4.0 * difficulty.powf(3.0)
    }
}
