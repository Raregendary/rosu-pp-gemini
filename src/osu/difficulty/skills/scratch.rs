//! Reusable scratch buffers for the difficulty value reductions.
//!
//! [`Speed::difficulty_value`](super::speed::Speed::difficulty_value),
//! [`Reading::difficulty_value`](super::reading::Reading::difficulty_value) and
//! [`Aim::difficulty_value`](super::aim::Aim::difficulty_value) all reduce a
//! per-object list of difficulties. Each of those lists is allocated fresh on
//! every call, and the values being reduced only ever grow.
//!
//! That is invisible for a batch calculation, where the reduction runs once.
//! It is not invisible for gradual difficulty, where `eval` - and therefore
//! every reduction - runs once per hit object: a playthrough of a 3399 object
//! map performed 21537 allocations totalling 250 MiB, growing quadratically
//! with the map length.
//!
//! The buffers live in thread-local storage so `difficulty_value` keeps taking
//! `&self` and no public signature changes. Each is borrowed for the duration of
//! one reduction and the reductions never call into each other, so there is no
//! reentrancy to guard against.

use std::cell::RefCell;

use super::aim::SortablePeak;

thread_local! {
    /// The filtered, sorted object difficulties of a harmonic reduction.
    static OBJECT_DIFFICULTIES: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    /// The finalised and reduced strain peaks of an aim reduction.
    static STRAIN_PEAKS: RefCell<Vec<SortablePeak>> = const { RefCell::new(Vec::new()) };
}

/// Runs `f` with the shared object-difficulty scratch buffer.
pub fn with_object_difficulties<R>(f: impl FnOnce(&mut Vec<f64>) -> R) -> R {
    OBJECT_DIFFICULTIES.with(|buffer| f(&mut buffer.borrow_mut()))
}

/// Runs `f` with the shared strain-peak scratch buffer.
pub fn with_strain_peaks<R>(f: impl FnOnce(&mut Vec<SortablePeak>) -> R) -> R {
    STRAIN_PEAKS.with(|buffer| f(&mut buffer.borrow_mut()))
}
