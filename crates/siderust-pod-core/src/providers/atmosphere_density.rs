//! Atmosphere-density provider trait.
//!
//! Returns mass density `ρ` (kg/m³) at a given altitude above the WGS-84
//! ellipsoid for use by the drag force model. The MVP-1 default is a constant
//! density adequate for very-low LEO regression tests; later milestones plug
//! in NRLMSISE-00 / DTM via `siderust-pod-io::atmosphere`.

use crate::error::Result;
use qtty::length::Kilometers;

/// Provider returning atmospheric mass density at a height.
pub trait AtmosphereDensityProvider: Send + Sync {
    /// Mass density `ρ` in kg/m³ at the given geodetic altitude.
    fn density_kg_m3(&self, height: Kilometers) -> Result<f64>;
}

/// Constant-density model. Use only for synthetic tests.
#[derive(Debug, Clone, Copy)]
pub struct ConstantDensity {
    /// Density value returned for every altitude, in kg/m³.
    pub rho: f64,
}

impl AtmosphereDensityProvider for ConstantDensity {
    fn density_kg_m3(&self, _h: Kilometers) -> Result<f64> {
        Ok(self.rho)
    }
}
