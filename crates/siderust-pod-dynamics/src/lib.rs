//! # siderust-pod-dynamics
//!
//! POD-specific dynamics composition. This crate is the bridge between
//! the generic propagation/force-model machinery in `siderust` (and the
//! future reusable `siderust-dynamics` crate) and the POD estimator.
//!
//! ## Scope
//!
//! - configurable force-model registry assembled from [`ForceModelConfig`],
//! - thrust-arc parameterisation that produces estimable parameters
//!   ([`thrust_arc::ThrustArcConfig`]),
//! - EKF process-noise wrappers ([`process_noise::ProcessNoiseConfig`]).
//!
//! ## Non-scope
//!
//! Generic propagation, integrators, analytic STM, two-body/J2/SRP/drag
//! mathematics live in `siderust::astro::dynamics` (today) and will move
//! into a dedicated reusable `siderust-dynamics` crate in a follow-up.
//! POD-side code must consume those, never reimplement them.

#![forbid(unsafe_code)]

pub mod force_config;
pub mod process_noise;
pub mod thrust_arc;

pub use force_config::ForceModelConfig;
pub use process_noise::ProcessNoiseConfig;
pub use thrust_arc::ThrustArcConfig;
