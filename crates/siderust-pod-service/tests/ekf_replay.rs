//! # EKF replay acceptance test
//!
//! ## Scientific scope
//!
//! This test checks that the sequential estimator reduces uncertainty and
//! converges toward repeated noisy measurements using a synthetic 6D orbit
//! state. The scientific purpose is to validate the estimator plumbing before
//! it is embedded in richer POD replay scenarios.
//!
//! Because the scenario is synthetic, passing this test does not certify full
//! operational orbit-filter performance. It only confirms that the current EKF
//! algebra behaves consistently on a controlled case.
//!
//! ## Technical scope
//!
//! The test instantiates the orbit estimator, feeds it deterministic scalar
//! updates on the x-position component, and asserts on posterior variance and
//! state convergence. It is a narrow regression guard for the service-facing
//! EKF path.
//!
//! No file I/O, REST handling, or product generation is involved.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
use siderust::astro::dynamics::{OrbitState, Position, Velocity};
use siderust::coordinates::frames::GCRS;
use siderust::time::JulianDate;
use siderust_pod_estimation::OrbitEkf;

#[test]
fn ekf_random_walk_converges() {
    let epoch = JulianDate::new(2_451_545.0);
    let pos = Position::<GCRS>::new(7000.0, 0.0, 0.0);
    let vel = Velocity::<GCRS>::new(0.0, 7.5, 0.0);
    let state0 = OrbitState::new(epoch, pos, vel);

    let mut f = OrbitEkf::from_stddevs(state0.clone(), [10.0, 10.0, 10.0], [0.1, 0.1, 0.1]);
    let truth_x = 7000.0 + std::f64::consts::PI; // ~7003.14 km
    let r = 0.01_f64;
    let mut last_var = f.covariance().to_row_major()[0][0];

    for _ in 0..50 {
        // No-op predict: identity STM, small Q on position.
        let mut phi = [[0.0f64; 6]; 6];
        for i in 0..6 {
            phi[i][i] = 1.0;
        }
        let q = siderust::astro::dynamics::covariance::StateCovariance::<GCRS>::from_stddevs(
            [0.001, 0.001, 0.001],
            [1e-6, 1e-6, 1e-6],
        );
        f.predict(f.state().clone(), phi, Some(q));

        // Observe x-component: h = [1,0,0,0,0,0].
        let innov = truth_x - f.state().position.x().value();
        f.update_scalar([1.0, 0.0, 0.0, 0.0, 0.0, 0.0], innov, r).unwrap();

        let var = f.covariance().to_row_major()[0][0];
        assert!(var <= last_var + 0.0011, "variance must not blow up");
        last_var = var;
    }

    assert!(
        (f.state().position.x().value() - truth_x).abs() < 1e-3,
        "x = {}",
        f.state().position.x().value()
    );
    assert!(f.covariance().to_row_major()[0][0] < 0.05, "posterior variance too large");
}
