//! # Residual CSV writer
//!
//! ## Scientific scope
//!
//! Residual time series are a standard diagnostic product in POD, allowing
//! analysts to inspect measurement fit quality by epoch, sensor, and
//! observable type. This module serializes those diagnostics into a simple
//! flat table for downstream plotting or audit workflows.
//!
//! It assumes residuals and sigmas have already been computed by the
//! service and QC layers. The writer does not reinterpret or normalize the
//! measurements.
//!
//! ## Technical scope
//!
//! The public surface consists of `ResidualRow` and `write_residuals_csv`.
//! Rows are caller-assembled records containing timestamps, labels,
//! residual values, and related metadata, and the writer streams them into
//! CSV form.
//!
//! Formatting is intentionally simple and stable; statistical aggregation
//! remains the responsibility of QC modules.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
use serde::{Deserialize, Serialize};
use std::io::Write;

/// One residual row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualRow {
    /// Epoch (Julian Date, TT scale).
    pub jd_tt: f64,
    /// Measurement type tag (e.g. "code-G01" or "phase-G05").
    pub kind: String,
    /// Measured value (metres).
    pub measured_m: f64,
    /// Predicted value (metres).
    pub predicted_m: f64,
    /// Residual (measured − predicted), metres.
    pub residual_m: f64,
    /// Standard deviation used (metres).
    pub sigma_m: f64,
}

/// Write a residuals CSV file.
pub fn write_residuals_csv<W: Write>(
    w: &mut W,
    rows: &[ResidualRow],
) -> Result<(), std::io::Error> {
    writeln!(w, "jd_tt,kind,measured_m,predicted_m,residual_m,sigma_m")?;
    for r in rows {
        writeln!(
            w,
            "{:.9},{},{:.6},{:.6},{:.6},{:.6}",
            r.jd_tt, r.kind, r.measured_m, r.predicted_m, r.residual_m, r.sigma_m
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_header_plus_rows() {
        let rows = vec![ResidualRow {
            jd_tt: 2_451_545.0,
            kind: "code-G01".into(),
            measured_m: 20_000_000.0,
            predicted_m: 20_000_001.0,
            residual_m: -1.0,
            sigma_m: 1.0,
        }];
        let mut buf = Vec::new();
        write_residuals_csv(&mut buf, &rows).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("jd_tt,kind,"));
        assert!(text.lines().nth(1).unwrap().contains("code-G01"));
    }
}
