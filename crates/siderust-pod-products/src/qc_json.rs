//! # QC JSON product writer
//!
//! ## Scientific scope
//!
//! Quality-control summaries condense orbit-determination diagnostics into
//! a machine-readable product that downstream automation can archive or
//! render. This module defines that lightweight document shape for grouped
//! residual and validation statistics.
//!
//! Its scope is descriptive rather than inferential: the statistics are
//! assumed to have been computed upstream, and the writer only preserves
//! them in a stable JSON layout.
//!
//! ## Technical scope
//!
//! The public items are `QcDocument` and `write_qc_json`. Callers provide
//! serializable summary payloads and run metadata, and the module emits a
//! deterministic JSON document suitable for REST delivery, artifact
//! storage, or HTML rendering.
//!
//! No statistical aggregation or residual computation happens here.
//!
//! ## References
//!
//! - Ben-Kiki, O., Evans, C., & d'Otremont, I. (2021). YAML Ain't Markup
//!   Language (YAML) Version 1.2.2.
//! - Bray, T. (2017). The JavaScript Object Notation (JSON) Data
//!   Interchange Format. RFC 8259.
use serde::Serialize;
use std::io::Write;

/// Top-level shape of `qc.json`.
#[derive(Debug, Clone, Serialize)]
pub struct QcDocument<S> {
    /// Schema version of the QC document.
    pub schema_version: String,
    /// Run identifier (matches the run manifest).
    pub run_id: String,
    /// Software version that produced the document.
    pub software_version: String,
    /// Total number of measurements processed.
    pub n_obs: usize,
    /// Number of estimated parameters.
    pub n_params: usize,
    /// Reduced χ² of the converged solution.
    pub reduced_chi2: f64,
    /// Number of estimator iterations.
    pub iterations: usize,
    /// Per-group residual statistics (typically grouped by measurement kind).
    pub residuals: S,
}

/// Write a `qc.json` file pretty-printed.
pub fn write_qc_json<W: Write, S: Serialize>(
    w: &mut W,
    doc: &QcDocument<S>,
) -> Result<(), std::io::Error> {
    let bytes = serde_json::to_vec_pretty(doc)?;
    w.write_all(&bytes)?;
    w.write_all(b"\n")?;
    Ok(())
}
