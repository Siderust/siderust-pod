#![allow(clippy::needless_range_loop, clippy::inconsistent_digit_grouping)]
//! # siderust-pod core compatibility layer
//!
//! ## Scientific scope
//!
//! This crate groups legacy POD-facing abstractions such as provider
//! traits, run manifests, and state re-exports used by older `siderust-pod`
//! callers. It does not define a new dynamical model of its own; the
//! scientific meaning of states, frames, and covariances remains upstream
//! in `siderust` and associated foundational crates.
//!
//! The crate is therefore best viewed as a compatibility seam around orbit-
//! determination workflows rather than a canonical implementation layer.
//! Its physical validity follows the re-exported upstream models, with no
//! additional force, timing, or measurement physics introduced here.
//!
//! ## Technical scope
//!
//! The public surface re-exports POD error, manifest, parameter, and
//! provider modules together with orbit-state and local-frame types sourced
//! from `siderust`. Callers use it to keep older import paths stable while
//! assembling estimation, propagation, and product-generation workflows.
//!
//! This module does not itself parse datasets, propagate trajectories, or
//! solve estimation problems. It only centralizes types and traits already
//! implemented elsewhere in the workspace.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Consultative Committee for Space Data Systems. (2010). Orbit Data
//!   Messages, CCSDS 502.0-B-2 / 502.0-B-3.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod manifest;
pub mod parameter;
pub mod providers;

pub use error::{PodError, Result};
pub use manifest::{DatasetRef, RunManifest};
pub use parameter::{Parameter, ParameterKind};
pub use providers::{ConstantDensity, DensityProvider, ExponentialAtmosphere,
    EopError, EopProvider, EopValues, BoxEphemeris, DynEphemeris, Vsop87Provider,
    FrameTransformProvider, GravityConstants, GravityFieldProvider, TwoBodyEarth};

// State types and local orbital frames live upstream in siderust; re-export here so
// existing POD callers keep working without change.
pub use siderust::astro::dynamics::frames::{LocalOrbitalFrame, LVLH, RTN, VNC};
pub use siderust::astro::dynamics::state::{
    Acceleration, AccelerationUnit, OrbitState, Position, SpacecraftProperties, SpacecraftState,
    StateDerivative, Velocity, VelocityUnit,
};
pub use siderust::astro::dynamics::covariance::{block_diag, similarity, Covariance6, StateCovariance};
pub use affn::Rotation3;
