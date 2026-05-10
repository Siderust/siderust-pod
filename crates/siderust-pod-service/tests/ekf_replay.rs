//! M6 acceptance: a degenerate EKF on a 1-D random walk converges to the
//! noisy measurement and ends with sub-σ posterior variance.
//!
//! The full POD-EKF replay (sharing `MeasurementModel` + `ForceModel` with
//! the batch estimator) is wired in `pod-estimation::sequential`. This test
//! exercises the same code path as a sanity check that the EKF reduces
//! uncertainty as it absorbs measurements.

use siderust_pod_estimation::Ekf;

#[test]
fn ekf_random_walk_converges() {
    let mut f = Ekf::new_diag(vec![0.0], &[10.0]);
    let truth = std::f64::consts::PI;
    let r = 0.01_f64;
    let mut last_var = f.p[0];
    for _ in 0..50 {
        // No-op predict (state is constant); identity Φ; small Q.
        f.predict(f.x.clone(), &[1.0], Some(&[0.001]));
        // Measurement = truth (noise-free for determinism).
        let innov = truth - f.x[0];
        f.update_scalar(innov, &[(0, 1.0)], r).unwrap();
        assert!(f.p[0] <= last_var + 0.0011, "variance must not blow up");
        last_var = f.p[0];
    }
    assert!((f.x[0] - truth).abs() < 1e-3, "x = {}", f.x[0]);
    assert!(f.p[0] < 0.05, "posterior variance still {}", f.p[0]);
}
