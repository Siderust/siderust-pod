#![allow(clippy::needless_range_loop, clippy::inconsistent_digit_grouping)]
//! # siderust-pod-core
//!
//! POD domain primitives, provider traits, manifests, and shared error types
//! for the Siderust POD product family.
//!
//! See `docs/architecture/boundaries.md` for the dependency rules and the
//! responsibilities of this crate.
//!
//! This crate is the only POD crate that depends directly on every foundational
//! crate (`qtty`, `tempoch`, `affn`, `cheby`, `siderust`). All other POD crates
//! consume foundations through the abstractions defined here.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod covariance;
pub mod error;
pub mod frames;
pub mod manifest;
pub mod parameter;
pub mod providers;

pub use error::{PodError, Result};
pub use frames::{RTN, VNC, LVLH};
pub use manifest::{DatasetRef, RunManifest};
pub use parameter::{Parameter, ParameterKind};

// State types live upstream in siderust; re-export here so existing POD
// callers keep working without change.
pub use siderust::astro::dynamics::state::{
    Acceleration, AccelerationUnit, OrbitState, Position, SpacecraftProperties, SpacecraftState,
    StateDerivative, Velocity, VelocityUnit,
};
