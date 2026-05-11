//! # siderust-pod product writers
//!
//! ## Scientific scope
//!
//! This crate turns in-memory estimation results into interchange and
//! reporting artifacts. The scientific content is inherited from the
//! upstream orbit solution and residual statistics; this layer focuses on
//! packaging those results into standard or workspace-defined product
//! shapes.
//!
//! Current outputs target deterministic MVP workflows: precise orbit
//! histories, CCSDS ephemerides, grouped QC JSON, and tabular residual
//! exports.
//!
//! ## Technical scope
//!
//! The crate re-exports helpers for writing SP3 and OEM orbit products,
//! residual CSV tables, and the workspace `qc.json` summary. Inputs are
//! typed orbit states, metadata, and precomputed statistics supplied by
//! service and QC modules.
//!
//! No estimation, propagation, or measurement modelling is performed here.
//!
//! ## References
//!
//! - Consultative Committee for Space Data Systems. (2010). Orbit Data
//!   Messages, CCSDS 502.0-B-2 / 502.0-B-3.
//! - International GNSS Service. (2020). SP3-c / SP3-d Orbit Format
//!   Specification.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod orbit;
pub mod qc_json;
pub mod residuals_csv;

pub use orbit::{write_oem_from_states, write_sp3_from_states};
pub use qc_json::write_qc_json;
pub use residuals_csv::{write_residuals_csv, ResidualRow};
