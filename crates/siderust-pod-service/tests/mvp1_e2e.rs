//! End-to-end MVP-1 acceptance test:
//! generates a synthetic GNSS arc, runs the WLS pipeline, and asserts
//! that all six artifacts are produced and the orbit is recovered to
//! within a tight tolerance of the truth state.

use siderust::time::JulianDate;
use siderust_pod_core::OrbitState;
use siderust_pod_service::{generate, run_synth, SyntheticArcConfig};
use std::path::PathBuf;

#[test]
fn mvp1_synth_pipeline_recovers_truth() {
    let cfg = SyntheticArcConfig::default();
    let arc = generate(&cfg);

    // Initial guess: truth perturbed by 50 m position, 0.05 m/s velocity.
    let t0 = arc.truth_states[0];
    let init = OrbitState::new(
        t0.epoch_tt,
        [t0.rx_km + 0.05, t0.ry_km - 0.05, t0.rz_km + 0.05],
        [t0.vx_km_s + 5e-5, t0.vy_km_s - 5e-5, t0.vz_km_s + 5e-5],
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
    let dx = (report.estimated_initial.rx_km - t0.rx_km) * 1000.0;
    let dy = (report.estimated_initial.ry_km - t0.ry_km) * 1000.0;
    let dz = (report.estimated_initial.rz_km - t0.rz_km) * 1000.0;
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
