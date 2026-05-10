#![allow(clippy::needless_range_loop, clippy::inconsistent_digit_grouping)]
//! `siderust-pod-service` — pipeline runner, config loader, manifest assembly.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod manifest;
pub mod pipeline;
pub mod runner;
pub mod synth;

pub use config::RunConfig;
pub use manifest::{canonical_json, DatasetRef, RunManifest};
pub use pipeline::{run_synth, ArcEpoch, GpsSatellite, PipelineError, PipelineReport};
pub use runner::{run, RunReport};
pub use synth::{generate, SyntheticArc, SyntheticArcConfig};

// Re-export the core orbit state types so downstream crates (pod-rest, pod-py)
// that only depend on this crate can access them without adding siderust directly.
pub use siderust::astro::dynamics::{OrbitState, Position, Velocity};
pub use siderust::astro::dynamics::state::VelocityUnit;
