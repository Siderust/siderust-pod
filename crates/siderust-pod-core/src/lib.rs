//! # siderust-pod-core
//!
//! Core POD domain primitives that the rest of the POD workspace builds on.
//!
//! ## Scope
//!
//! This crate intentionally does **not** own numerical algorithms, file
//! parsing, propagation, or estimation. Those live in the
//! `siderust-pod-{dynamics,io,observations,estimation,qc,products}` crates.
//!
//! It owns:
//!
//! - the POD error taxonomy ([`error::PodError`]),
//! - run-level provenance ([`manifest::RunManifest`], [`dataset::DatasetRef`]),
//! - parameter typing for the estimator ([`parameter::ParameterKind`],
//!   [`parameter::Parameter`], [`covariance::ParameterCovariance`]),
//! - provider trait re-exports / adapters bridging POD code to public
//!   `siderust` services ([`providers`]).
//!
//! Geometric primitives (RTN/LVLH/VNC, `StateCovariance`, `OrbitState`,
//! `SpacecraftState`, finite-difference STM) live in `siderust` and are
//! re-exported here for ergonomics — they are *not* redefined.

#![forbid(unsafe_code)]

pub mod covariance;
pub mod dataset;
pub mod error;
pub mod manifest;
pub mod parameter;
pub mod providers;

pub use error::PodError;
