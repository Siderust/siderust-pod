//! Criterion benchmark skeleton for `siderust-pod-dynamics`.
//!
//! Runs RK4 propagation of a 1-day LEO arc under (a) two-body alone and
//! (b) two-body + J2. Establishes a baseline for the audit-driven
//! performance work scheduled in milestone M10 / M11.
//!
//! Run with:
//!
//! ```text
//! cargo bench -p siderust-pod-dynamics
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use siderust::time::JulianDate;
use siderust_pod_core::OrbitState;
use siderust_pod_dynamics::prelude::{rk4_propagate, CompositeForce, TwoBody, J2};

fn make_initial() -> OrbitState {
    let mu: f64 = 398_600.441_8;
    let r: f64 = 6_378.137 + 500.0;
    let v: f64 = (mu / r).sqrt();
    OrbitState::new(JulianDate::new(2_451_545.0), [r, 0.0, 0.0], [0.0, v, 0.0])
}

fn bench_rk4_two_body(c: &mut Criterion) {
    let force = CompositeForce::empty().push(Box::new(TwoBody::earth()));
    let s0 = make_initial();
    let dt = 30.0;
    let n = 24 * 3600 / 30;
    c.bench_function("rk4_two_body_1day_30s", |b| {
        b.iter(|| black_box(rk4_propagate(&force, s0, dt, n)));
    });
}

fn bench_rk4_two_body_j2(c: &mut Criterion) {
    let force = CompositeForce::empty()
        .push(Box::new(TwoBody::earth()))
        .push(Box::new(J2::earth()));
    let s0 = make_initial();
    let dt = 30.0;
    let n = 24 * 3600 / 30;
    c.bench_function("rk4_two_body_plus_j2_1day_30s", |b| {
        b.iter(|| black_box(rk4_propagate(&force, s0, dt, n)));
    });
}

criterion_group!(benches, bench_rk4_two_body, bench_rk4_two_body_j2);
criterion_main!(benches);
