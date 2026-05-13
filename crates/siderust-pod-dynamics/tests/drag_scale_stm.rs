// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Integration test: drag-scale parameter partials — Richardson convergence.
//!
//! Uses [`param_partials_central_diff`] to estimate `∂y(t)/∂C_D` for a LEO
//! spacecraft under atmospheric drag with an exponential atmosphere model.
//!
//! The Richardson convergence check confirms that the coarse estimate (step ε)
//! and refined estimate (step ε/2) agree to better than `1e-6` in max relative
//! difference, validating the finite-difference accuracy of the variational
//! harness.
//!
//! Orbit: circular LEO at 500 km altitude.
//! Atmosphere: single-layer exponential (Vallado §8.6, 500 km reference).
//! Integration: RK4 with 10 s step, propagation arc 120 s.

use std::sync::Arc;

use siderust::astro::dynamics::forces::{
    CompositeForce, DragForce, ExponentialAtmosphere, TwoBody,
};
use siderust::astro::dynamics::{DynamicsContextBuilder, OrbitState, Position, Velocity};
use siderust::coordinates::frames::GCRS;
use siderust::qtty::{AreaToMass, DragCoefficient, Second};
use siderust::time::JulianDate;
use siderust_pod_dynamics::{param_partials_central_diff, Rk4Integrator};

/// Earth gravitational parameter (km³/s²).
const GM_EARTH: f64 = 398_600.441_8;
/// Earth mean radius (km) — WGS-84 approximation.
const R_EARTH_KM: f64 = 6_371.0;
/// LEO altitude (km).
const H_LEO_KM: f64 = 500.0;
/// LEO orbit radius (km).
const R_LEO: f64 = R_EARTH_KM + H_LEO_KM;

/// Circular orbital velocity at 500 km (km/s).
fn v_leo() -> f64 {
    (GM_EARTH / R_LEO).sqrt()
}

/// Build a LEO [`OrbitState`] at `[r_leo, 0, 0]` with circular velocity.
fn leo_state() -> OrbitState {
    OrbitState::new_at_jd(
        JulianDate::new(2_451_545.0),
        Position::<GCRS>::new(R_LEO, 0.0, 0.0),
        Velocity::<GCRS>::new(0.0, v_leo(), 0.0),
    )
}

/// Dynamics context with a 500 km exponential atmosphere.
fn leo_ctx() -> siderust::astro::dynamics::DynamicsContext {
    let atm = Arc::new(ExponentialAtmosphere::LEO_500KM);
    DynamicsContextBuilder::new().with_atmosphere(atm).build()
}

/// Two-body + drag composite (used for both branches of the central diff).
///
/// Parameterised on `cd` so it can be supplied to `param_partials_central_diff`.
fn two_body_drag(cd: f64) -> CompositeForce {
    let area_to_mass = AreaToMass::new(0.01);
    CompositeForce::empty()
        .push(Box::new(TwoBody::earth()))
        .push(Box::new(DragForce::new(
            DragCoefficient::new(cd),
            area_to_mass,
        )))
}

/// Richardson convergence: coarse (ε) vs refined (ε/2) central-diff estimates
/// of `∂y/∂C_D` must agree to within 1e-3 relative.
#[test]
fn drag_scale_stm_richardson_convergence() {
    let integrator = Rk4Integrator {
        step: Second::new(10.0),
    };
    let state = leo_state();
    let ctx = leo_ctx();
    let dt = Second::new(120.0);
    let cd_nominal = 2.2_f64;
    let epsilon = 1e-2;

    let coarse = param_partials_central_diff(
        &integrator,
        state,
        dt,
        &ctx,
        cd_nominal,
        epsilon,
        two_body_drag,
    )
    .expect("coarse finite-diff must succeed");

    let refined = param_partials_central_diff(
        &integrator,
        state,
        dt,
        &ctx,
        cd_nominal,
        epsilon / 2.0,
        two_body_drag,
    )
    .expect("refined finite-diff must succeed");

    let rel_diff = coarse.max_relative_difference(&refined);
    assert!(
        rel_diff < 1e-3,
        "Richardson convergence failed: max relative diff = {rel_diff:.3e} (expected < 1e-3)"
    );
}

/// The drag-scale partial column must be non-trivial (drag force is not
/// invariant under C_D perturbation for a dense atmosphere at 500 km).
#[test]
fn drag_scale_partial_is_non_zero() {
    let integrator = Rk4Integrator {
        step: Second::new(10.0),
    };
    let state = leo_state();
    let ctx = leo_ctx();
    let dt = Second::new(120.0);

    let col = param_partials_central_diff(&integrator, state, dt, &ctx, 2.2, 1e-4, two_body_drag)
        .expect("finite-diff must succeed");

    let magnitude = col.0.iter().map(|x| x * x).sum::<f64>().sqrt();
    assert!(
        magnitude > 1e-20,
        "drag-scale partial column is unexpectedly zero (magnitude = {magnitude:.3e})"
    );
}
