//! Cached per-index weights for the harmonic decay reduction shared by
//! [`Speed`](super::speed::Speed) and [`Reading`](super::reading::Reading).

use std::{cell::RefCell, thread::LocalKey};

thread_local! {
    /// Weights for [`Speed`](super::speed::Speed), which uses
    /// `HARMONIC_SCALE = 20`.
    static SPEED_WEIGHTS: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    /// Weights for [`Reading`](super::reading::Reading), which uses
    /// `HARMONIC_SCALE = 1`.
    static READING_WEIGHTS: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
}

pub const fn speed_weights() -> &'static LocalKey<RefCell<Vec<f64>>> {
    &SPEED_WEIGHTS
}

pub const fn reading_weights() -> &'static LocalKey<RefCell<Vec<f64>>> {
    &READING_WEIGHTS
}

/// The weight of index `i`.
///
/// This is the exact expression the callers used to evaluate inline, so a
/// cached value is bit-identical to recomputing it.
#[inline]
fn compute_weight(i: usize, harmonic_scale: f64, decay_exponent: f64) -> f64 {
    let scale_term = harmonic_scale / (1 + i) as f64;
    (1.0 + scale_term) / ((i as f64).powf(decay_exponent) + 1.0 + scale_term)
}

/// The weighted sum of `diffs` together with the sum of the weights that
/// produced it.
///
/// The weight of index `i` depends only on `i` and on the skill's harmonic
/// scale, never on the values being reduced, so the weights are memoised per
/// thread. The first call pays exactly what the inline expression used to pay;
/// every later call is a contiguous read instead of a `powf`.
///
/// This matters for gradual difficulty, where `difficulty_value` runs once per
/// hit object: without the cache the reduction performs `O(n^2)` `powf` calls
/// over a playthrough.
pub fn weighted_sum(
    cache: &'static LocalKey<RefCell<Vec<f64>>>,
    harmonic_scale: f64,
    decay_exponent: f64,
    diffs: &[f64],
) -> (f64, f64) {
    cache.with(|cell| {
        let mut weights = cell.borrow_mut();

        let missing = diffs.len().saturating_sub(weights.len());

        if missing > 0 {
            weights.reserve(missing);

            for _ in 0..missing {
                let i = weights.len();
                weights.push(compute_weight(i, harmonic_scale, decay_exponent));
            }
        }

        let weights = &weights[..diffs.len()];

        let mut num = 0.0;
        let mut object_weight_sum = 0.0;

        for (&item, &weight) in diffs.iter().zip(weights) {
            object_weight_sum += weight;
            num += item * weight;
        }

        (num, object_weight_sum)
    })
}
