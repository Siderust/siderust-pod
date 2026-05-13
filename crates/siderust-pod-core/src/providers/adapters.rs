//! Default thin adapters around public `siderust` services.
//!
//! These exist for prototyping / testing; production POD pipelines layer
//! their own context-aware adapters (kernel selection, EOP tables, …) on
//! top of the provider traits in
//! [`crate::providers`].
//!
//! # Examples
//!
//! ```
//! use siderust_pod_core::providers::{
//!     adapters::SiderustGravityField, GravityFieldProvider,
//! };
//! let g = SiderustGravityField::two_body();
//! assert_eq!(g.max_degree(), 0);
//! ```

use siderust::astro::dynamics::gravity::{GravityFieldProvider, LowDegreeEarth, TwoBodyEarth};
use siderust::astro::dynamics::units::GravitationalParameter;

/// Default gravity-field adapter selecting one of the canonical
/// `siderust` Earth models.
///
/// Use [`Self::two_body`] for fast point-mass propagation or
/// [`Self::low_degree`] for an EGM2008-truncated 4×4 model.
///
/// # Examples
///
/// ```
/// use siderust_pod_core::providers::adapters::SiderustGravityField;
/// use siderust_pod_core::providers::GravityFieldProvider;
///
/// assert_eq!(SiderustGravityField::two_body().max_degree(), 0);
/// assert_eq!(SiderustGravityField::low_degree().max_degree(), 4);
/// ```
#[derive(Debug, Clone)]
pub enum SiderustGravityField {
    /// Point-mass Earth (`C̄₀₀ = 1`, all other coefficients zero).
    TwoBody(TwoBodyEarth),
    /// EGM2008-truncated degree/order-4 Earth model.
    LowDegree(LowDegreeEarth),
}

impl SiderustGravityField {
    /// Build a point-mass Earth adapter.
    pub fn two_body() -> Self {
        Self::TwoBody(TwoBodyEarth)
    }

    /// Build an EGM2008-truncated degree/order-4 Earth adapter.
    pub fn low_degree() -> Self {
        Self::LowDegree(LowDegreeEarth)
    }
}

impl GravityFieldProvider for SiderustGravityField {
    fn gm(&self) -> GravitationalParameter {
        match self {
            Self::TwoBody(g) => g.gm(),
            Self::LowDegree(g) => g.gm(),
        }
    }

    fn reference_radius(&self) -> siderust::qtty::Kilometers {
        match self {
            Self::TwoBody(g) => g.reference_radius(),
            Self::LowDegree(g) => g.reference_radius(),
        }
    }

    fn max_degree(&self) -> usize {
        match self {
            Self::TwoBody(g) => g.max_degree(),
            Self::LowDegree(g) => g.max_degree(),
        }
    }

    fn max_order(&self) -> usize {
        match self {
            Self::TwoBody(g) => g.max_order(),
            Self::LowDegree(g) => g.max_order(),
        }
    }

    fn c_normalized(&self, n: usize, m: usize) -> f64 {
        match self {
            Self::TwoBody(g) => g.c_normalized(n, m),
            Self::LowDegree(g) => g.c_normalized(n, m),
        }
    }

    fn s_normalized(&self, n: usize, m: usize) -> f64 {
        match self {
            Self::TwoBody(g) => g.s_normalized(n, m),
            Self::LowDegree(g) => g.s_normalized(n, m),
        }
    }
}
