//! `qc.json` writer — wraps grouped residual statistics with run metadata.

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
