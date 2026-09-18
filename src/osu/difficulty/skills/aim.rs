use crate::{
    any::difficulty::object::IDifficultyObject,
    model::mods::GameMods,
    osu::difficulty::{
        evaluators::{AgilityEvaluator, FlowAimEvaluator, SnapAimEvaluator},
        object::OsuDifficultyObject,
    },
    util::{
        difficulty::{logistic, logistic_exp, norm},
        float_ext::FloatExt,
    },
};

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
    decay_weight: f64,
    max_section_length: f64,
    max_stored_length: f64,
    is_touch_device: bool,
    is_relax: bool,
    is_autopilot: bool,
}

impl Aim {
    pub fn new(mods: &GameMods, include_sliders: bool) -> Self {
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
            decay_weight,
            max_section_length,
            max_stored_length,
            is_touch_device: mods.td(),
            is_relax: mods.rx(),
            is_autopilot: mods.ap(),
        }
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

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        if self.is_autopilot {
            return 0.0;
        }

        let num = Self::strain_decay(curr.adjusted_delta_time);
        self.current_strain *= num;
        self.current_strain += self.calculate_adjusted_difficulty(curr, diff_objects) * (1.0 - num);

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
        let snap_difficulty =
            SnapAimEvaluator::evaluate_diff_of(curr, diff_objects, self.include_sliders) * 70.9;
        let agility_difficulty =
            AgilityEvaluator::evaluate_diff_of(curr, diff_objects) * 2.35;
        let flow_difficulty =
            FlowAimEvaluator::evaluate_diff_of(curr, diff_objects, self.include_sliders) * 242.0;

        let num = self.calculate_total_value(snap_difficulty, agility_difficulty, flow_difficulty);

        num * (0.985 + curr.overall_difficulty().max(0.0).powf(2.0) / 4000.0)
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

    pub fn process(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        diff_objects: &[OsuDifficultyObject<'_>],
    ) {
        if curr.idx == 0 {
            self.current_section_begin = curr.start_time;
            self.current_section_end = self.current_section_begin + self.max_section_length;
            self.current_section_peak = self.strain_value_at(curr, diff_objects);
            self.object_difficulties.push(self.current_section_peak);
            return;
        }

        self.backfill_peaks(curr, diff_objects);

        let num = self.strain_value_at(curr, diff_objects);
        self.object_difficulties.push(num);

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

    pub fn get_current_strain_peaks(&mut self) -> &[StrainPeak] {
        if !self.peaks_finalised {
            self.save_current_peak(self.current_section_end - self.current_section_begin);
            self.peaks_finalised = true;
        }
        &self.strain_peaks
    }

    pub fn into_current_strain_peaks(mut self) -> Vec<f64> {
        self.get_current_strain_peaks().iter().map(|p| p.value).collect()
    }

    #[allow(dead_code)]
    pub fn cloned_difficulty_value(&self) -> f64 {
        let mut clone = self.clone();
        clone.difficulty_value()
    }

    pub fn difficulty_value(&mut self) -> f64 {
        let mut num = 0.0;
        let mut num2 = 0.0;

        for reduced_strain_peak in self.get_reduced_strain_peaks() {
            let exponent = num2;
            let num3 = num2 + reduced_strain_peak.section_length / self.max_section_length;
            let num4 = self.decay_weight.powf(exponent) - self.decay_weight.powf(num3);
            num += reduced_strain_peak.value * num4;
            num2 = num3;
        }

        num / (1.0 - self.decay_weight)
    }

    fn get_reduced_strain_peaks(&mut self) -> Vec<StrainPeak> {
        let peaks = self.get_current_strain_peaks();
        let mut list: Vec<StrainPeak> = peaks.iter().copied().filter(|p| p.value > 0.0).collect();

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
                list.push(StrainPeak::new(
                    strain_peak.value * (0.727 + (1.0 - 0.727) * num4),
                    20.0_f64.min(strain_peak.section_length - num3),
                ));
                num3 += 20.0;
            }

            num += strain_peak.section_length;
            num2 += 1;
        }

        let mut remaining: Vec<StrainPeak> = list.into_iter().skip(num2).collect();
        remaining.sort_by(|a, b| b.value.total_cmp(&a.value));
        remaining
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
