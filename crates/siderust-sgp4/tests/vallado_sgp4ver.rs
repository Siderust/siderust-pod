// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Regression test against Vallado's SGP4-VER reference vectors.
//!
//! We propagate each `[[list]]` entry in `test-data/vallado_sgp4ver_subset.toml`
//! to every `[[list.states]]` epoch and compare the resulting TEME state
//! component-wise against the published reference.
//!
//! Tolerances:
//!
//! * position — `1e-6 km` per component
//! * velocity — `1e-9 km·s⁻¹` per component
//!
//! These are the bit-exact tolerances guaranteed by the upstream `sgp4`
//! crate against Vallado's `tcppver.out`; they confirm that our typed
//! wrapper is non-lossy.

use serde::Deserialize;
use siderust_sgp4::Sgp4Propagator;

const POS_TOL_KM: f64 = 1e-6;
const VEL_TOL_KMPS: f64 = 1e-9;

#[derive(Deserialize)]
struct Cases {
    list: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    line1: String,
    line2: String,
    states: Vec<State>,
}

#[derive(Deserialize)]
struct State {
    time: f64,
    position: [f64; 3],
    velocity: [f64; 3],
}

#[test]
fn sgp4ver_subset_matches_reference() {
    let raw = include_str!("../test-data/vallado_sgp4ver_subset.toml");
    let cases: Cases = toml::from_str(raw).expect("fixture parses as TOML");

    let mut total = 0usize;
    let mut sat_count = 0usize;
    for case in &cases.list {
        sat_count += 1;
        // Use siderust-tle's typed parser.
        let tle = siderust_tle::parse_tle(&case.line1, &case.line2)
            .expect("siderust-tle parses vendored TLE");

        let prop = Sgp4Propagator::from_tle(&tle).unwrap_or_else(|e| {
            panic!("init failed for NORAD {}: {e}", tle.norad_id.0);
        });

        for state in &case.states {
            let predicted = prop.propagate_minutes(state.time).unwrap_or_else(|e| {
                panic!(
                    "propagation failed for NORAD {} at t={} min: {e}",
                    tle.norad_id.0, state.time
                );
            });
            let p = predicted.position().as_array();
            let v = predicted.velocity().as_array();
            for i in 0..3 {
                let dp = (p[i].value() - state.position[i]).abs();
                let dv = (v[i].value() - state.velocity[i]).abs();
                assert!(
                    dp <= POS_TOL_KM,
                    "NORAD {}, t={} min, pos[{i}]: |Δ| = {:e} > {:e} (got {}, want {})",
                    tle.norad_id.0,
                    state.time,
                    dp,
                    POS_TOL_KM,
                    p[i].value(),
                    state.position[i]
                );
                assert!(
                    dv <= VEL_TOL_KMPS,
                    "NORAD {}, t={} min, vel[{i}]: |Δ| = {:e} > {:e} (got {}, want {})",
                    tle.norad_id.0,
                    state.time,
                    dv,
                    VEL_TOL_KMPS,
                    v[i].value(),
                    state.velocity[i]
                );
            }
            total += 1;
        }
    }
    assert!(
        sat_count >= 5,
        "expected ≥ 5 reference satellites, got {sat_count}"
    );
    assert!(total >= 15, "expected ≥ 15 epoch checks total, got {total}");
}

/// Smoke test for [`Sgp4Propagator::propagate_at`]: stepping a UTC Julian
/// date forwards by one orbital period must reproduce the
/// `propagate_minutes` result to within numerical noise.
#[test]
fn propagate_at_matches_propagate_minutes() {
    use qtty::Quantity;
    use qtty_core::units::time::Day;
    use tempoch::{JulianDate, UTC};

    let tle = siderust_tle::parse_tle(
        "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927",
        "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537",
    )
    .unwrap();
    let prop = Sgp4Propagator::from_tle(&tle).unwrap();

    let dt_min = 90.0_f64;
    let by_minutes = prop.propagate_minutes(dt_min).unwrap();
    let target = JulianDate::<UTC>::try_new(Quantity::<Day>::new(
        prop.epoch_jd_utc().raw().value() + dt_min / 1_440.0,
    ))
    .unwrap();
    let by_jd = prop.propagate_at(target).unwrap();
    for i in 0..3 {
        let dp =
            by_minutes.position().as_array()[i].value() - by_jd.position().as_array()[i].value();
        let dv =
            by_minutes.velocity().as_array()[i].value() - by_jd.velocity().as_array()[i].value();
        assert!(dp.abs() < 1e-9, "pos[{i}] mismatch: {dp:e}");
        assert!(dv.abs() < 1e-12, "vel[{i}] mismatch: {dv:e}");
    }
}
