//! SLR validation: feed an estimated orbit + station fixture + CRD ranges into
//! the SLR range model and produce O−C residual statistics.
//!
//! This is a lightweight wrapper that doesn't try to model the full SLR
//! processing chain (no atmosphere, no relativity, no station eccentricity
//! correction). Its purpose is to expose a typed, testable summary for QC
//! consumers; the underlying numerics live in `pod-observations::slr`.

use crate::residuals::ResidualStats;

/// One O−C residual at an SLR observation epoch.
#[derive(Debug, Clone)]
pub struct SlrResidual {
    /// Julian Date (TT) of the bounce epoch.
    pub jd_tt: f64,
    /// O−C residual, metres.
    pub residual_m: f64,
}

/// Aggregate SLR validation report.
#[derive(Debug, Clone)]
pub struct SlrValidationReport {
    /// Per-pass residuals.
    pub residuals: Vec<SlrResidual>,
    /// Aggregate statistics.
    pub stats: ResidualStats,
}

impl SlrValidationReport {
    /// Build the report from a list of `(jd_tt, residual_m)` pairs.
    pub fn from_pairs<I: IntoIterator<Item = (f64, f64)>>(pairs: I) -> Self {
        let residuals: Vec<SlrResidual> = pairs
            .into_iter()
            .map(|(t, r)| SlrResidual {
                jd_tt: t,
                residual_m: r,
            })
            .collect();
        let values: Vec<f64> = residuals.iter().map(|r| r.residual_m).collect();
        let stats = ResidualStats::from_slice(&values);
        Self { residuals, stats }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_zero_residuals() {
        let r = SlrValidationReport::from_pairs((0..5).map(|i| (i as f64, 0.0)));
        assert_eq!(r.residuals.len(), 5);
        assert!(r.stats.rms.abs() < 1e-12);
    }
}
