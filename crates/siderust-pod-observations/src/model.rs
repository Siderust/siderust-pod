//! Generic measurement-model trait used by the estimator.
//!
//! A `MeasurementModel` predicts a scalar observable from the current state
//! and parameter block, and provides the row of partial derivatives the
//! estimator needs to assemble its design matrix.

use siderust::astro::dynamics::OrbitState;

/// Predicted observation value with attached partial-derivative row.
#[derive(Debug, Clone)]
pub struct Prediction {
    /// Predicted observable in the same units as the measurement.
    pub value: f64,
    /// Partial derivatives.
    pub partials: Partials,
}

/// Sparse partial-derivative row for one measurement.
///
/// Indexing matches the parameter ordering chosen by the estimator. The
/// 6 leading entries correspond to the orbit state ([rx, ry, rz, vx, vy, vz]);
/// later entries correspond to additional parameters (clock bias, float
/// ambiguities, …) appended by the model.
#[derive(Debug, Clone, Default)]
pub struct Partials {
    /// Sparse `(parameter_index, value)` pairs.
    pub entries: Vec<(usize, f64)>,
}

impl Partials {
    /// Convenience constructor.
    pub fn from_pairs(pairs: impl IntoIterator<Item = (usize, f64)>) -> Self {
        Self {
            entries: pairs.into_iter().collect(),
        }
    }
}

/// A model that predicts one scalar measurement value.
pub trait MeasurementModel {
    /// Predict the measurement value and partials at the given orbit state and
    /// `extra_params` slice (which contains every non-state parameter the
    /// estimator currently exposes, in the same order it appended them).
    fn predict(&self, state: &OrbitState, extra_params: &[f64]) -> Prediction;

    /// Standard deviation of the assumed measurement noise (same units as
    /// the observable).
    fn sigma(&self) -> f64;
}
