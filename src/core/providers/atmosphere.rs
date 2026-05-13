//! Atmosphere density provider re-export.
//!
//! The canonical density-provider trait lives in
//! [`siderust::astro::dynamics::atmosphere::DensityProvider`]. POD code
//! consumes it under the more descriptive alias
//! [`AtmosphereDensityProvider`] to keep the role clear at call sites
//! (drag-force computation).
//!
//! # Examples
//!
//! ```
//! use siderust_pod::core::providers::AtmosphereDensityProvider;
//! use siderust::astro::dynamics::atmosphere::ConstantDensity;
//! use siderust::qtty::{Kilometers, KilogramsPerCubicMeter};
//!
//! let p = ConstantDensity { rho: KilogramsPerCubicMeter::new(1.0e-12) };
//! let rho = p.density(Kilometers::new(500.0)).unwrap();
//! assert!(rho.value() > 0.0);
//! ```

/// Alias for [`siderust::astro::dynamics::atmosphere::DensityProvider`]
/// emphasising the role at POD call sites.
pub use siderust::astro::dynamics::atmosphere::DensityProvider as AtmosphereDensityProvider;

/// Re-export of the dynamic-dispatch convenience alias from `siderust`.
pub use siderust::astro::dynamics::atmosphere::AtmosphereProvider;
