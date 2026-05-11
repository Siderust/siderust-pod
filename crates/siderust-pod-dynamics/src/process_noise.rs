//! EKF process-noise configuration.
//!
//! The numerical EKF lives in `siderust-pod-estimation::sequential`. This
//! module owns the *configuration* surface so that runs can be described
//! declaratively (and hashed into the manifest) without pulling in the
//! estimator dependency.

/// Diagonal continuous-time process-noise spectral densities for the
/// reference Cartesian state (km^2/s^3 for velocity components, km^2/s for
/// position components — concrete units depend on the integrator chosen).
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessNoiseConfig {
    pub position_psd: [f64; 3],
    pub velocity_psd: [f64; 3],
    /// Optional drag-scale random-walk PSD.
    pub drag_scale_psd: Option<f64>,
    /// Optional SRP-scale random-walk PSD.
    pub srp_scale_psd: Option<f64>,
}

impl Default for ProcessNoiseConfig {
    fn default() -> Self {
        Self {
            position_psd: [0.0; 3],
            velocity_psd: [1e-12; 3],
            drag_scale_psd: None,
            srp_scale_psd: None,
        }
    }
}
