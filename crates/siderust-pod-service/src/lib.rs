#![allow(clippy::needless_range_loop, clippy::inconsistent_digit_grouping)]
//! `siderust-pod-service` — pipeline runner, config loader, manifest assembly.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod pipeline;
pub mod runner;
pub mod synth;

pub use config::RunConfig;
pub use pipeline::{run_synth, ArcEpoch, GpsSatellite, PipelineError, PipelineReport};
pub use runner::{run, RunReport};
pub use siderust_pod_core::OrbitState;
pub use synth::{generate, SyntheticArc, SyntheticArcConfig};
