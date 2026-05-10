//! Residuals CSV writer.

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
