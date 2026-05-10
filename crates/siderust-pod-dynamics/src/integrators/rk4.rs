//! Classical Runge-Kutta 4 integrator for the orbital `[r,v]` 6-vector.

use crate::forces::ForceModel;
use siderust::time::JulianDate;
use siderust_pod_core::OrbitState;

/// Day-fraction equivalent to one second.
const SEC_PER_DAY: f64 = 86_400.0;

/// Step the orbit state by `dt_s` seconds using RK4 with the given force.
pub fn rk4_step<F: ForceModel>(force: &F, s: &OrbitState, dt_s: f64) -> OrbitState {
    let y0 = s.to_array6();
    let k1 = derivative(force, s);
    let s1 = step_with(s, &y0, &k1, dt_s * 0.5);
    let k2 = derivative(force, &s1);
    let s2 = step_with(s, &y0, &k2, dt_s * 0.5);
    let k3 = derivative(force, &s2);
    let s3 = step_with(s, &y0, &k3, dt_s);
    let k4 = derivative(force, &s3);

    let mut y = [0.0f64; 6];
    for i in 0..6 {
        y[i] = y0[i] + dt_s / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
    }
    let new_jd = JulianDate::new(s.epoch_tt.jd_value() + dt_s / SEC_PER_DAY);
    OrbitState::from_array6(new_jd, y)
}

fn derivative<F: ForceModel>(force: &F, s: &OrbitState) -> [f64; 6] {
    let a = force.acceleration(s);
    let [vx, vy, vz] = s.velocity_km_s();
    [vx, vy, vz, a[0], a[1], a[2]]
}

fn step_with(base: &OrbitState, y0: &[f64; 6], k: &[f64; 6], dt: f64) -> OrbitState {
    let mut y = [0.0f64; 6];
    for i in 0..6 {
        y[i] = y0[i] + dt * k[i];
    }
    OrbitState::from_array6(base.epoch_tt, y)
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
