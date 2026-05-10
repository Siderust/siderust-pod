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
use siderust_pod_core::OrbitState;

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
    let x0 = s0.to_array6();
    for j in 0..6 {
        let scale = x0[j].abs().max(1.0);
        let h = 1e-6 * scale;

        let mut xp = x0;
        xp[j] += h;
        let sp = OrbitState::from_array6(s0.epoch_tt, xp);
        let s_plus = rk4_propagate(force, sp, dt_s, n_steps).to_array6();

        let mut xm = x0;
        xm[j] -= h;
        let sm = OrbitState::from_array6(s0.epoch_tt, xm);
        let s_minus = rk4_propagate(force, sm, dt_s, n_steps).to_array6();

        for i in 0..6 {
            stm[i][j] = (s_plus[i] - s_minus[i]) / (2.0 * h);
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
    let x0 = s0.to_array6();
    let mut perturbed: [[Vec<[f64; 6]>; 2]; 6] = Default::default();
    let mut hs = [0.0_f64; 6];
    for j in 0..6 {
        let scale = x0[j].abs().max(1.0);
        let h = 1e-6 * scale;
        hs[j] = h;
        for (sign_idx, sign) in [-1.0_f64, 1.0_f64].iter().enumerate() {
            let mut xp = x0;
            xp[j] += sign * h;
            let sp = OrbitState::from_array6(s0.epoch_tt, xp);
            let series = rk4_propagate_series(force, sp, dt_s, n_steps);
            perturbed[j][sign_idx] = series.into_iter().map(|s| s.to_array6()).collect();
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
