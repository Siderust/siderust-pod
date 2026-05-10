//! Numerical state-transition matrices.
//!
//! For MVP we expose a simple finite-difference STM around a reference state
//! that is propagated by the supplied force model and integrator. This avoids
//! coding analytic variational equations for every force model and is plenty
//! accurate for batch POD windows up to a few hours.
//!
//! The Jacobian is computed by central differences with a relative
//! perturbation of `1e-6` on each component of the initial 6-state.

use crate::forces::ForceModel;
use crate::integrators::{rk4_propagate, rk4_propagate_series};
use siderust_pod_core::{OrbitState, Position, Velocity};
use siderust::coordinates::frames::GCRS;

/// Return component `j` of the 6-state `[rx, ry, rz, vx, vy, vz]`.
fn state_component(s: &OrbitState, j: usize) -> f64 {
    match j {
        0 => s.position.x().value(),
        1 => s.position.y().value(),
        2 => s.position.z().value(),
        3 => s.velocity.x().value(),
        4 => s.velocity.y().value(),
        5 => s.velocity.z().value(),
        _ => panic!("index out of range"),
    }
}

/// Return a copy of `s` with component `j` shifted by `delta`.
fn perturb_component(s: &OrbitState, j: usize, delta: f64) -> OrbitState {
    let rx = s.position.x().value() + if j == 0 { delta } else { 0.0 };
    let ry = s.position.y().value() + if j == 1 { delta } else { 0.0 };
    let rz = s.position.z().value() + if j == 2 { delta } else { 0.0 };
    let vx = s.velocity.x().value() + if j == 3 { delta } else { 0.0 };
    let vy = s.velocity.y().value() + if j == 4 { delta } else { 0.0 };
    let vz = s.velocity.z().value() + if j == 5 { delta } else { 0.0 };
    OrbitState::new(
        s.epoch_tt,
        Position::<GCRS>::new(rx, ry, rz),
        Velocity::<GCRS>::new(vx, vy, vz),
    )
}

/// Finite-difference state-transition matrix Φ(t,t0).
///
/// `dt_s` is the integrator step size; `n_steps` is the number of steps over
/// the propagation interval. Returns a 6×6 row-major Jacobian
/// `∂x(t)/∂x(t0)`.
pub fn finite_diff_stm<F: ForceModel>(
    force: &F,
    s0: OrbitState,
    dt_s: f64,
    n_steps: usize,
) -> [[f64; 6]; 6] {
    let mut stm = [[0.0; 6]; 6];
    for j in 0..6 {
        let x0j = state_component(&s0, j);
        let scale = x0j.abs().max(1.0);
        let h = 1e-6 * scale;

        let s_plus = rk4_propagate(force, perturb_component(&s0, j, h), dt_s, n_steps);
        let s_minus = rk4_propagate(force, perturb_component(&s0, j, -h), dt_s, n_steps);

        for i in 0..6 {
            stm[i][j] = (state_component(&s_plus, i) - state_component(&s_minus, i)) / (2.0 * h);
        }
    }
    stm
}

/// Compute the STM Φ(t_k, t_0) at every step `k = 0..=n_steps`.
///
/// Each Φ_k is returned as a 6×6 row-major matrix. Φ_0 is the identity.
/// This shares the 12 perturbation propagations across all output epochs,
/// which is far cheaper than calling [`finite_diff_stm`] per epoch.
pub fn finite_diff_stm_series<F: ForceModel>(
    force: &F,
    s0: OrbitState,
    dt_s: f64,
    n_steps: usize,
) -> Vec<[[f64; 6]; 6]> {
    let mut perturbed: [[Vec<[f64; 6]>; 2]; 6] = Default::default();
    let mut hs = [0.0_f64; 6];
    for j in 0..6 {
        let x0j = state_component(&s0, j);
        let scale = x0j.abs().max(1.0);
        let h = 1e-6 * scale;
        hs[j] = h;
        for (sign_idx, sign) in [-1.0_f64, 1.0_f64].iter().enumerate() {
            let sp = perturb_component(&s0, j, sign * h);
            let series = rk4_propagate_series(force, sp, dt_s, n_steps);
            perturbed[j][sign_idx] = series
                .into_iter()
                .map(|s| [
                    state_component(&s, 0),
                    state_component(&s, 1),
                    state_component(&s, 2),
                    state_component(&s, 3),
                    state_component(&s, 4),
                    state_component(&s, 5),
                ])
                .collect();
        }
    }
    let mut out = Vec::with_capacity(n_steps + 1);
    for k in 0..=n_steps {
        let mut phi = [[0.0; 6]; 6];
        for j in 0..6 {
            let plus = perturbed[j][1][k];
            let minus = perturbed[j][0][k];
            for i in 0..6 {
                phi[i][j] = (plus[i] - minus[i]) / (2.0 * hs[j]);
            }
        }
        out.push(phi);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forces::TwoBody;
    use siderust::time::JulianDate;
    use siderust_pod_core::{Position, Velocity};

    #[test]
    fn two_body_stm_is_close_to_identity_for_zero_dt() {
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            Position::new(7000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        );
        let f = TwoBody::earth();
        let phi = finite_diff_stm(&f, s, 1.0, 0);
        for i in 0..6 {
            for j in 0..6 {
                let target = if i == j { 1.0 } else { 0.0 };
                assert!((phi[i][j] - target).abs() < 1e-9);
            }
        }
    }
}
