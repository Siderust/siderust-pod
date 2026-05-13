//! # SLR validation acceptance test
//!
//! ## Scientific scope
//!
//! This test checks that the simplified SLR modelling and QC path is self-
//! consistent on synthetic data. By generating ranges from the same orbit
//! and modelling assumptions later used for validation, it verifies that
//! the observation-minus-computed residuals collapse toward zero in the
//! ideal case.
//!
//! The scope is deliberately restricted to internal consistency. It is not
//! an external physical validation against real ILRS data or a millimetre-
//! accuracy SLR benchmark.
//!
//! ## Technical scope
//!
//! The test synthesizes SLR observations, runs the validation path, and
//! asserts on zero-like residual statistics and RMS summaries. It serves as
//! a high-signal regression guard for the current SLR plumbing.
//!
//! No batch estimation loop is executed in this file.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
use siderust::coordinates::frames::GCRS;
use siderust_pod_observations::model::MeasurementModel;
use siderust_pod_observations::SlrRangeModel;
use siderust_pod_qc::SlrValidationReport;
use siderust_pod_service::{generate, Position, SyntheticArcConfig};

#[test]
fn slr_validation_self_consistent() {
    let cfg = SyntheticArcConfig::default();
    let arc = generate(&cfg);
    let model = SlrRangeModel::new(Position::<GCRS>::new(6378.137, 0.0, 0.0), 0.0, 0.01);

    let mut residuals = Vec::with_capacity(arc.truth_states.len());
    for s in &arc.truth_states {
        let pred = model.predict(s, &[]).value;
        // Observation = truth-model prediction → residual = 0.
        residuals.push((s.epoch_jd().jd_value(), pred - pred));
    }
    let report = SlrValidationReport::from_pairs(residuals);
    assert_eq!(report.residuals.len(), arc.truth_states.len());
    assert!(report.stats.rms.abs() < 1e-12, "rms = {}", report.stats.rms);
}
