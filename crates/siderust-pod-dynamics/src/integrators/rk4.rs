//! Classical Runge-Kutta 4 integrator for the orbital state.

use crate::forces::ForceModel;
use siderust::time::JulianDate;
use siderust_pod_core::{OrbitState, StateDerivative};

/// Day-fraction equivalent to one second.
const SEC_PER_DAY: f64 = 86_400.0;

/// Step the orbit state by `dt_s` seconds using RK4 with the given force.
pub fn rk4_step<F: ForceModel>(force: &F, s: &OrbitState, dt_s: f64) -> OrbitState {
    let k1 = derivative(force, s);
    let s1 = s.advance(&k1, dt_s * 0.5);
    let k2 = derivative(force, &s1);
    let s2 = s.advance(&k2, dt_s * 0.5);
    let k3 = derivative(force, &s2);
    let s3 = s.advance(&k3, dt_s);
    let k4 = derivative(force, &s3);

    let combined = StateDerivative::rk4_combine(&k1, &k2, &k3, &k4);
    let new_jd = JulianDate::new(s.epoch_tt.jd_value() + dt_s / SEC_PER_DAY);
    let advanced = s.advance(&combined, dt_s);
    OrbitState::new(new_jd, advanced.position, advanced.velocity)
}

fn derivative<F: ForceModel>(force: &F, s: &OrbitState) -> StateDerivative {
    StateDerivative::from_components(
        [s.velocity.x().value(), s.velocity.y().value(), s.velocity.z().value()],
        force.acceleration(s),
    )
}

/// Propagate `state` over `n_steps` of `dt_s` seconds each.
pub fn rk4_propagate<F: ForceModel>(
    force: &F,
    state: OrbitState,
    dt_s: f64,
    n_steps: usize,
) -> OrbitState {
    let mut s = state;
    for _ in 0..n_steps {
        s = rk4_step(force, &s, dt_s);
    }
    s
}

/// Propagate and collect the state at the start and after each step.
/// Output length is `n_steps + 1`.
pub fn rk4_propagate_series<F: ForceModel>(
    force: &F,
    state: OrbitState,
    dt_s: f64,
    n_steps: usize,
) -> Vec<OrbitState> {
    let mut out = Vec::with_capacity(n_steps + 1);
    out.push(state);
    let mut s = state;
    for _ in 0..n_steps {
        s = rk4_step(force, &s, dt_s);
        out.push(s);
    }
    out
}
