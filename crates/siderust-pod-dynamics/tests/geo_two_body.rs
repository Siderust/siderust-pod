// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Integration test: GEO two-body propagation round-trip.
//!
//! Propagates a circular GEO orbit for exactly one sidereal day (one orbital
//! period) under pure two-body gravity using [`propagate_orbit`] and asserts
//! that the position error is ≤ 1 mm (1 × 10⁻⁶ km).
//!
//! GEO parameters used:
//! - Semi-major axis  `a = 42 164.17 km` (standard GEO)
//! - Circular velocity `v = sqrt(GM_⊕ / a) ≈ 3.0747 km/s`
//! - Orbital period   `T = 2π sqrt(a³/GM_⊕) ≈ 86 164.1 s`

use siderust::astro::dynamics::forces::TwoBody;
use siderust::astro::dynamics::{DynamicsContext, OrbitState, Position, Velocity};
use siderust::coordinates::frames::GCRS;
use siderust::qtty::Second;
use siderust::time::JulianDate;
use siderust_pod_dynamics::{propagate_orbit, Rk4Integrator};

/// Earth gravitational parameter (km³/s²).
const GM_EARTH: f64 = 398_600.441_8;
/// GEO semi-major axis (km).
const A_GEO: f64 = 42_164.17;

/// Circular orbital velocity at GEO (km/s).
fn v_geo() -> f64 {
    (GM_EARTH / A_GEO).sqrt()
}

/// Exact GEO orbital period (s).
fn t_geo() -> f64 {
    2.0 * std::f64::consts::PI * (A_GEO.powi(3) / GM_EARTH).sqrt()
}

/// Euclidean distance between two states (km).
fn position_error(a: &OrbitState, b: &OrbitState) -> f64 {
    let dx = a.position.x().value() - b.position.x().value();
    let dy = a.position.y().value() - b.position.y().value();
    let dz = a.position.z().value() - b.position.z().value();
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Propagate for one full GEO orbital period with RK4 step 30 s.
///
/// After one exact period the spacecraft should return within 1 mm of the
/// initial position under the two-body approximation.
#[test]
fn geo_round_trip_one_period_position_error_le_1mm() {
    let v = v_geo();
    let period = t_geo();

    let s0 = OrbitState::new_at_jd(
        JulianDate::new(2_451_545.0),
        Position::<GCRS>::new(A_GEO, 0.0, 0.0),
        Velocity::<GCRS>::new(0.0, v, 0.0),
    );

    // 30 s step — yields ~2872 steps for one GEO period.
    let integrator = Rk4Integrator {
        step: Second::new(30.0),
    };
    let ctx = DynamicsContext::empty();
    let force = TwoBody::earth();

    let s1 = propagate_orbit(&integrator, &force, s0, Second::new(period), &ctx)
        .expect("propagation must succeed");

    let err_km = position_error(&s0, &s1);
    assert!(
        err_km < 1e-3,
        "GEO round-trip position error {err_km:.3e} km exceeds 1 mm (1e-3 km)"
    );
}

/// Sanity-check that GEO period is close to one sidereal day (86 164 s).
#[test]
fn geo_period_sanity() {
    let period = t_geo();
    // IAU standard sidereal day is 86 164.1 s.
    assert!(
        (period - 86_164.1).abs() < 1.0,
        "Computed GEO period {period:.1} s differs from sidereal day by > 1 s"
    );
}
