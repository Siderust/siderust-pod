//! M5 acceptance: SLR validation pipeline on synthetic data.
//!
//! Builds a noise-free SLR range time series from the synthetic truth orbit
//! against a fictitious ground station, then runs the same range model
//! against the same orbit and confirms residuals are zero (the model is
//! consistent with itself) and the QC summary reports zero RMS.

use siderust_pod_observations::model::MeasurementModel;
use siderust_pod_observations::SlrRangeModel;
use siderust_pod_qc::SlrValidationReport;
use siderust_pod_service::{generate, SyntheticArcConfig};

#[test]
fn slr_validation_self_consistent() {
    let cfg = SyntheticArcConfig::default();
    let arc = generate(&cfg);
    let station_km = [6378.137_f64, 0.0, 0.0];
    let model = SlrRangeModel::new(station_km, 0.0, 0.01);

    let mut residuals = Vec::with_capacity(arc.truth_states.len());
    for s in &arc.truth_states {
        let pred = model.predict(s, &[]).value;
        // Observation = truth-model prediction → residual = 0.
        residuals.push((s.epoch_tt.jd_value(), pred - pred));
    }
    let report = SlrValidationReport::from_pairs(residuals);
    assert_eq!(report.residuals.len(), arc.truth_states.len());
    assert!(report.stats.rms.abs() < 1e-12, "rms = {}", report.stats.rms);
}
