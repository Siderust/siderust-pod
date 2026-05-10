use siderust::time::JulianDate;
use siderust_pod_core::OrbitState;
use siderust_pod_dynamics::prelude::{rk4_propagate, TwoBody};

/// Specific orbital energy: ½v² - μ/r.
fn energy(s: &OrbitState, gm: f64) -> f64 {
    let r = s.r2().sqrt();
    let [vx, vy, vz] = s.velocity_km_s();
    let v2 = vx.powi(2) + vy.powi(2) + vz.powi(2);
    0.5 * v2 - gm / r
}

#[test]
fn two_body_energy_conservation_one_orbit() {
    let mu: f64 = 398_600.441_8;
    let r: f64 = 6378.137 + 500.0;
    let v: f64 = (mu / r).sqrt();
    let s0 = OrbitState::new(JulianDate::new(2_451_545.0), [r, 0.0, 0.0], [0.0, v, 0.0]);

    let force = TwoBody::earth();
    // One orbital period
    let period = 2.0 * std::f64::consts::PI * (r.powi(3) / mu).sqrt();
    let dt = 10.0; // 10 s step
    let n = (period / dt) as usize;
    let s1 = rk4_propagate(&force, s0, dt, n);

    let e0 = energy(&s0, mu);
    let e1 = energy(&s1, mu);
    let drift = ((e1 - e0) / e0).abs();
    assert!(
        drift < 1e-9,
        "RK4 energy drift over 1 orbit too large: {}",
        drift
    );
}

#[test]
fn two_body_returns_to_origin_one_period() {
    let mu: f64 = 398_600.441_8;
    let r: f64 = 6378.137 + 500.0;
    let v: f64 = (mu / r).sqrt();
    let s0 = OrbitState::new(JulianDate::new(2_451_545.0), [r, 0.0, 0.0], [0.0, v, 0.0]);

    let force = TwoBody::earth();
    let period = 2.0 * std::f64::consts::PI * (r.powi(3) / mu).sqrt();
    let dt = period / 600.0; // exact division -> orbit closes
    let n = 600usize;
    let s1 = rk4_propagate(&force, s0, dt, n);

    let [x0, y0, z0] = s0.position_km();
    let [x1, y1, z1] = s1.position_km();
    let dx = x1 - x0;
    let dy = y1 - y0;
    let dz = z1 - z0;
    let err = (dx * dx + dy * dy + dz * dz).sqrt();
    assert!(err < 1.0, "position error {} km too large", err);
}
