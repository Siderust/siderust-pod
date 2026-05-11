//! # Generic scalar measurement-model interface
//!
//! ## Scientific scope
//!
//! Orbit-determination measurements share a common algebraic pattern:
//! predict a scalar observable from the current state, then supply partial
//! derivatives with respect to solved-for parameters. This module captures
//! that shared structure without committing to a specific sensor type.
//!
//! The scientific validity is inherited from the concrete model
//! implementations. By design, this layer is about the estimation interface
//! rather than about any one measurement physics.
//!
//! ## Technical scope
//!
//! The public items are `Prediction`, `Partials`, and the
//! `MeasurementModel` trait. Implementors return a scalar predicted value
//! and sparse partial derivatives, which the estimation layer can assemble
//! into normal equations or sequential updates.
//!
//! This module deliberately avoids owning file formats, propagation, or
//! bias models so that GNSS, SLR, and future sensors can share the same
//! estimator seam.
//!
//! ## References
//!
//! - Misra, P., & Enge, P. (2012). Global Positioning System: Signals,
//!   Measurements, and Performance (2nd ed.). Ganga-Jamuna Press.
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
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
