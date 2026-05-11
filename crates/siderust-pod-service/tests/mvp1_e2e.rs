//! # MVP-1 end-to-end acceptance test
//!
//! ## Scientific scope
//!
//! This test runs the full synthetic MVP POD workflow from arc generation
//! through estimation and product emission, then checks that the recovered
//! orbit remains close to the synthetic truth. It is the main system-level
//! scientific regression for the current short-arc GNSS path.
//!
//! The scenario stays within the simplified MVP regime: deterministic
//! synthetic measurements, compact force modelling, and a fixed artifact
//! set. It is therefore an integration guard rather than an external
//! validation against real tracking data.
//!
//! ## Technical scope
//!
//! The test calls the top-level service runner on a synthetic
//! configuration, verifies that all expected artifacts are produced, and
//! checks that the final recovered state satisfies the current tolerance
//! envelope. It intentionally spans several crates so wiring regressions
//! surface quickly.
//!
//! This file is not a public API surface; it is a workspace acceptance
//! harness.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
use siderust::time::JulianDate;
use siderust_pod_service::{generate, run_synth, OrbitState, Position, SyntheticArcConfig, Velocity};
use std::path::PathBuf;

#[test]
fn mvp1_synth_pipeline_recovers_truth() {
    let cfg = SyntheticArcConfig::default();
    let arc = generate(&cfg);

    // Initial guess: truth perturbed by 50 m position, 0.05 m/s velocity.
    let t0 = arc.truth_states[0];
    let init = OrbitState::new(
        t0.epoch_tt,
        Position::new(
            t0.position.x().value() + 0.05,
            t0.position.y().value() - 0.05,
            t0.position.z().value() + 0.05,
        ),
        Velocity::new(
            t0.velocity.x().value() + 5e-5,
            t0.velocity.y().value() - 5e-5,
            t0.velocity.z().value() + 5e-5,
        ),
    );
    let _ = init.epoch_tt; // silence unused variant
    let _ = JulianDate::new(2_451_545.0);

    let out = std::env::temp_dir().join(format!("siderust_pod_mvp1_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);

    let report = run_synth(
        &arc,
        init,
        /* initial_clock_guess_m = */ 0.0,
        &out,
        "mvp1-test",
        /* enable_j2 = */ false,
    )
    .expect("pipeline run");

    // Files exist.
    let expected: &[PathBuf] = &[
        out.join("run.manifest.json"),
        out.join("products/orbit.sp3"),
        out.join("products/orbit.oem"),
        out.join("residuals/residuals.csv"),
        out.join("qc/qc.json"),
    ];
    for p in expected {
        assert!(p.exists(), "missing artifact {p:?}");
    }

    // Position recovery: estimated initial state should be within ~1 m of truth.
    let ei_rx = report.estimated_initial.position.x().value();
    let ei_ry = report.estimated_initial.position.y().value();
    let ei_rz = report.estimated_initial.position.z().value();
    let dx = (ei_rx - t0.position.x().value()) * 1000.0;
    let dy = (ei_ry - t0.position.y().value()) * 1000.0;
    let dz = (ei_rz - t0.position.z().value()) * 1000.0;
    let pos_err_m = (dx * dx + dy * dy + dz * dz).sqrt();
    assert!(
        pos_err_m < 5.0,
        "initial-position recovery error = {pos_err_m:.3} m (expected < 5 m)"
    );

    // Clock bias recovery within a few sigma of code noise.
    let dclk = (report.estimated_clock_bias_m - cfg.clock_bias_m).abs();
    assert!(
        dclk < 1.0,
        "clock-bias error = {dclk:.3} m (expected < 1 m)"
    );

    // Convergence sanity.
    assert!(report.estimator.iterations <= 8);
}
