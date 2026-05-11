//! # EKF replay acceptance test
//!
//! ## Scientific scope
//!
//! This test checks that the sequential estimator reduces uncertainty and
//! converges toward repeated noisy measurements in a deliberately simple
//! one-dimensional random-walk setting. The scientific purpose is to
//! validate the estimator plumbing before it is embedded in richer POD
//! replay scenarios.
//!
//! Because the scenario is synthetic and low dimensional, passing this test
//! does not certify full operational orbit-filter performance. It only
//! confirms that the current EKF algebra behaves consistently on a
//! controlled case.
//!
//! ## Technical scope
//!
//! The test instantiates the sequential estimator, feeds it deterministic
//! updates, and asserts on posterior variance and state convergence. It is
//! a narrow regression guard for the service-facing EKF path.
//!
//! No file I/O, REST handling, or product generation is involved.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
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
