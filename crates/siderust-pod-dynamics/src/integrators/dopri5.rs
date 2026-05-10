//! Adaptive RK4(5) Dormand-Prince integrator.
//!
//! This is a mainstream DOPRI5 with PI step-size controller. Sufficient for
//! MVP-1 LEO POD; a higher-order DOP853 will replace it in M5 once
//! validated against IERS test orbits.

use crate::forces::ForceModel;
use siderust::time::JulianDate;
use siderust_pod_core::OrbitState;

const SEC_PER_DAY: f64 = 86_400.0;

/// Tolerances for adaptive integration.
#[derive(Debug, Clone, Copy)]
pub struct Tolerance {
    /// Relative tolerance.
    pub rel: f64,
    /// Absolute tolerance (km, km/s).
    pub abs: f64,
}

impl Default for Tolerance {
    fn default() -> Self {
        Self {
            rel: 1e-10,
            abs: 1e-10,
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn rhs<F: ForceModel>(force: &F, s: &OrbitState) -> [f64; 6] {
    let a = force.acceleration(s);
    [s.vx_km_s, s.vy_km_s, s.vz_km_s, a[0], a[1], a[2]]
}

/// Single adaptive DOPRI5 step. Returns `(new_state, h_used, h_next)`.
#[allow(clippy::too_many_lines)]
pub fn dopri5_step<F: ForceModel>(
    force: &F,
    s: &OrbitState,
    h_try: f64,
    tol: Tolerance,
) -> (OrbitState, f64, f64) {
    // Dormand-Prince 5(4) Butcher tableau coefficients.
    let c2 = 1.0 / 5.0;
    let c3 = 3.0 / 10.0;
    let c4 = 4.0 / 5.0;
    let c5 = 8.0 / 9.0;

    let a21 = 1.0 / 5.0;
    let a31 = 3.0 / 40.0;
    let a32 = 9.0 / 40.0;
    let a41 = 44.0 / 45.0;
    let a42 = -56.0 / 15.0;
    let a43 = 32.0 / 9.0;
    let a51 = 19_372.0 / 6_561.0;
    let a52 = -25_360.0 / 2_187.0;
    let a53 = 64_448.0 / 6_561.0;
    let a54 = -212.0 / 729.0;
    let a61 = 9_017.0 / 3_168.0;
    let a62 = -355.0 / 33.0;
    let a63 = 46_732.0 / 5_247.0;
    let a64 = 49.0 / 176.0;
    let a65 = -5_103.0 / 18_656.0;
    let a71 = 35.0 / 384.0;
    let a73 = 500.0 / 1_113.0;
    let a74 = 125.0 / 192.0;
    let a75 = -2_187.0 / 6_784.0;
    let a76 = 11.0 / 84.0;

    // 5th-order weights = a7*
    // 4th-order embedded weights:
    let e1 = 71.0 / 57_600.0;
    let e3 = -71.0 / 16_695.0;
    let e4 = 71.0 / 1_920.0;
    let e5 = -17_253.0 / 339_200.0;
    let e6 = 22.0 / 525.0;
    let e7 = -1.0 / 40.0;

    let mut h = h_try;
    let y0 = s.to_array6();

    loop {
        let s_at = |dt: f64, y: [f64; 6]| -> OrbitState {
            let jd = JulianDate::new(s.epoch_tt.jd_value() + dt / SEC_PER_DAY);
            OrbitState::from_array6(jd, y)
        };

        let k1 = rhs(force, s);
        let mut y2 = [0.0; 6];
        for i in 0..6 {
            y2[i] = y0[i] + h * a21 * k1[i];
        }
        let k2 = rhs(force, &s_at(c2 * h, y2));

        let mut y3 = [0.0; 6];
        for i in 0..6 {
            y3[i] = y0[i] + h * (a31 * k1[i] + a32 * k2[i]);
        }
        let k3 = rhs(force, &s_at(c3 * h, y3));

        let mut y4 = [0.0; 6];
        for i in 0..6 {
            y4[i] = y0[i] + h * (a41 * k1[i] + a42 * k2[i] + a43 * k3[i]);
        }
        let k4 = rhs(force, &s_at(c4 * h, y4));

        let mut y5 = [0.0; 6];
        for i in 0..6 {
            y5[i] = y0[i] + h * (a51 * k1[i] + a52 * k2[i] + a53 * k3[i] + a54 * k4[i]);
        }
        let k5 = rhs(force, &s_at(c5 * h, y5));

        let mut y6 = [0.0; 6];
        for i in 0..6 {
            y6[i] =
                y0[i] + h * (a61 * k1[i] + a62 * k2[i] + a63 * k3[i] + a64 * k4[i] + a65 * k5[i]);
        }
        let k6 = rhs(force, &s_at(h, y6));

        let mut y7 = [0.0; 6];
        for i in 0..6 {
            y7[i] =
                y0[i] + h * (a71 * k1[i] + a73 * k3[i] + a74 * k4[i] + a75 * k5[i] + a76 * k6[i]);
        }
        let k7 = rhs(force, &s_at(h, y7));

        // Error estimate.
        let mut err_norm = 0.0;
        for i in 0..6 {
            let err =
                h * (e1 * k1[i] + e3 * k3[i] + e4 * k4[i] + e5 * k5[i] + e6 * k6[i] + e7 * k7[i]);
            let sc = tol.abs + tol.rel * y0[i].abs().max(y7[i].abs());
            let r = err / sc;
            err_norm += r * r;
        }
        err_norm = (err_norm / 6.0).sqrt();

        if err_norm <= 1.0 || h.abs() < 1e-9 {
            // Accept.
            let h_next = if err_norm == 0.0 {
                h * 5.0
            } else {
                let factor = 0.9 * err_norm.powf(-0.2);
                h * factor.clamp(0.2, 5.0)
            };
            let new_state = s_at(h, y7);
            return (new_state, h, h_next);
        } else {
            // Reject and shrink.
            let factor = 0.9 * err_norm.powf(-0.2);
            h *= factor.clamp(0.1, 0.9);
        }
    }
}

/// Propagate from `state` for `total_dt_s` seconds with adaptive steps.
pub fn dopri5_propagate<F: ForceModel>(
    force: &F,
    state: OrbitState,
    total_dt_s: f64,
    tol: Tolerance,
) -> OrbitState {
    let mut s = state;
    let mut t = 0.0;
    let mut h = total_dt_s.signum() * 30.0_f64.min(total_dt_s.abs());
    while (total_dt_s - t).abs() > 1e-9 {
        if (t + h - total_dt_s) * total_dt_s.signum() > 0.0 {
            h = total_dt_s - t;
        }
        let (s_new, h_used, h_next) = dopri5_step(force, &s, h, tol);
        s = s_new;
        t += h_used;
        h = h_next;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forces::TwoBody;

    #[test]
    fn dopri5_one_orbit_closes() {
        let mu: f64 = 398_600.4418;
        let r: f64 = 7_000.0;
        let v: f64 = (mu / r).sqrt();
        let s0 = OrbitState::new(JulianDate::new(2_451_545.0), [r, 0.0, 0.0], [0.0, v, 0.0]);
        let period = 2.0 * std::f64::consts::PI * (r.powi(3) / mu).sqrt();
        let s = dopri5_propagate(&TwoBody::earth(), s0, period, Tolerance::default());
        let dr = ((s.rx_km - r).powi(2) + s.ry_km.powi(2) + s.rz_km.powi(2)).sqrt();
        assert!(dr < 1.0, "orbit closure error {} km exceeds 1 km", dr);
    }
}
