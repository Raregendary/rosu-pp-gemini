//! Map-wide difficulty constants.

/// Values that are identical for every difficulty object of a calculation.
///
/// [`OsuDifficultyObject`](super::object::OsuDifficultyObject) stores several
/// per-map constants (`hit_window_great`, `preempt`, `clock_rate`, `radius`,
/// `small_circle_bonus`) so evaluators can read them off the object they
/// already have a reference to. Everything *derived* from those is different:
/// it does not depend on the hit object at all, but the evaluators recompute it
/// for every single one of them.
///
/// `powf`, `sqrt` and a division per hit object add up to several percent of the
/// osu!standard difficulty pass, so the derived values are evaluated once here
/// and passed down instead.
///
/// Every field is the exact expression it replaces, so the results are
/// bit-identical to evaluating it inline.
#[derive(Copy, Clone, Debug)]
pub struct OsuDifficultyContext {
    /// `preempt * clock_rate`
    pub opacity_raw_preempt: f64,
    /// `400.0 * (opacity_raw_preempt / 450.0).min(1.0)`
    pub opacity_divisor: f64,
    /// `opacity_raw_preempt * 0.4`
    pub opacity_fade_in: f64,
    /// `opacity_raw_preempt * 0.3`
    pub opacity_fade_out: f64,
    /// `((500.0 - preempt + (preempt - 500.0).abs()) / 2.0).powf(2.5) / 140_000.0`
    pub preempt_difficulty: f64,
    /// `preempt.powf(2.2) * 0.01`
    pub hidden_preempt_difficulty: f64,
    /// `1.0 + (30.0 - radius) / 70.0`, clamped to at least `1.0`
    pub small_circle_bonus: f64,
    /// `small_circle_bonus.sqrt()`
    pub small_circle_bonus_sqrt: f64,
    /// `small_circle_bonus.powf(1.5)`
    pub small_circle_bonus_pow_15: f64,
    /// `0.985 + overall_difficulty.max(0.0).powf(2.0) / 4000.0`
    pub aim_od_factor: f64,
    /// `0.825 + overall_difficulty.max(0.0).powf(2.2) / 1125.0`
    pub reading_od_factor: f64,
    /// `52.0 / radius`, the flashlight scaling factor
    pub scaling_factor: f64,
}

impl OsuDifficultyContext {
    pub fn new(hit_window_great: f64, preempt: f64, clock_rate: f64, radius: f64) -> Self {
        // `OsuDifficultyObject::overall_difficulty`
        let overall_difficulty = (79.5 - hit_window_great / 2.0) / 6.0;

        let raw_preempt = preempt * clock_rate;
        let small_circle_bonus = (1.0 + (30.0 - radius) / 70.0).max(1.0);

        Self {
            opacity_raw_preempt: raw_preempt,
            opacity_divisor: 400.0 * (raw_preempt / 450.0).min(1.0),
            opacity_fade_in: raw_preempt * 0.4,
            opacity_fade_out: raw_preempt * 0.3,
            preempt_difficulty: ((500.0 - preempt + (preempt - 500.0).abs()) / 2.0).powf(2.5)
                / 140_000.0,
            hidden_preempt_difficulty: preempt.powf(2.2) * 0.01,
            small_circle_bonus,
            small_circle_bonus_sqrt: small_circle_bonus.sqrt(),
            small_circle_bonus_pow_15: small_circle_bonus.powf(1.5),
            aim_od_factor: 0.985 + overall_difficulty.max(0.0).powf(2.0) / 4000.0,
            reading_od_factor: 0.825 + overall_difficulty.max(0.0).powf(2.2) / 1125.0,
            scaling_factor: 52.0 / radius,
        }
    }
}
