//! Gravity-field provider re-export.
//!
//! The canonical fully-normalised spherical-harmonic provider trait lives
//! in [`siderust::astro::dynamics::gravity`]. Re-exporting it here keeps a
//! single source of truth and ensures POD code, `siderust-pod-dynamics`,
//! and `siderust` itself agree on the trait contract.
//!
//! # Examples
//!
//! ```
//! use siderust_pod::core::providers::GravityFieldProvider;
//! use siderust::astro::dynamics::gravity::TwoBodyEarth;
//!
//! let p = TwoBodyEarth;
//! // Spherical-harmonic two-body model has C̄₀₀ = 1, S̄_{nm} = 0.
//! assert!((p.c_normalized(0, 0) - 1.0).abs() < 1e-12);
//! assert_eq!(p.s_normalized(2, 1), 0.0);
//! ```

pub use siderust::astro::dynamics::gravity::{GravityConstants, GravityFieldProvider};
