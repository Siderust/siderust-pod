//! `siderust-pod-products` — output product writers.
//!
//! Wraps the io-layer writers with POD-domain conveniences:
//! * Build an SP3 record from an in-memory orbit time series.
//! * Build an OEM file from the same series.
//! * Emit a residuals CSV.
//! * Emit a `qc.json` with grouped statistics.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod orbit;
pub mod qc_json;
pub mod residuals_csv;

pub use orbit::{write_oem_from_states, write_sp3_from_states};
pub use qc_json::write_qc_json;
pub use residuals_csv::{write_residuals_csv, ResidualRow};
