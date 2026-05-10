//! `siderust-pod-qc` — residual statistics, comparison metrics, and QC JSON.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod html;
pub mod orbit_compare;
pub mod residuals;
pub mod slr_validation;

pub use html::render_html;
pub use orbit_compare::{rtn_diff, rtn_summary, RtnDiff, RtnSummary};
pub use residuals::{ResidualStats, ResidualsByGroup};
pub use slr_validation::{SlrResidual, SlrValidationReport};
