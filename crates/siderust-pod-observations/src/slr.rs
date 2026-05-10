//! Satellite Laser Ranging (SLR) two-way range measurement model.
//!
//! Predicts a two-way range observation between an Earth-fixed station and
//! a satellite given as inertial state. The model implemented here is
//! intentionally minimal so that an MVP-2 SLR validation pipeline can be
//! built without locking us into a specific atmosphere/relativity choice:
//!
//! * Light-time iteration (downlink + uplink) using a constant-velocity
//!   tangent at the receive epoch — sufficient at LEO/MEO ranges where
//!   the satellite moves <8 km/s and the bounce light-time is <0.1 s.
//! * Optional constant tropospheric range bias (m). Replace with a Mendes-
//!   Pavlis or Marini-Murray closure once `pod-qc::slr_validation` is wired.
//!
//! Frames: the station position is expressed in the *same inertial frame as
//! the satellite state at the bounce epoch*. The caller is responsible for
//! ITRF→GCRF rotation using a `siderust` frame transform provider.

use crate::model::{MeasurementModel, Partials, Prediction};
use siderust_pod_core::OrbitState;

/// One SLR range observation (two-way time-of-flight converted to metres).
#[derive(Debug, Clone)]
pub struct SlrRangeObs {
    /// Two-way range, metres.
    pub range_m: f64,
    /// Standard deviation of the observation, metres.
    pub sigma_m: f64,
}

/// SLR two-way range model.
#[derive(Debug, Clone)]
pub struct SlrRangeModel {
    /// Station position in the same inertial frame as the satellite state, km.
    pub station_inertial_km: [f64; 3],
    /// Constant tropospheric range delay (metres, applied symmetrically).
    pub trop_bias_m: f64,
    /// Assumed measurement standard deviation (metres).
    pub sigma_m: f64,
    /// Index of an additive station-bias parameter inside `extra_params`,
    /// or `None` if no station bias is being estimated.
    pub station_bias_index: Option<usize>,
}

impl SlrRangeModel {
    /// New model with no bias estimation.
    pub fn new(station_inertial_km: [f64; 3], trop_bias_m: f64, sigma_m: f64) -> Self {
        Self {
            station_inertial_km,
            trop_bias_m,
            sigma_m,
            station_bias_index: None,
        }
    }
}

const C_M_S: f64 = 299_792_458.0;

impl MeasurementModel for SlrRangeModel {
    fn predict(&self, state: &OrbitState, extra: &[f64]) -> Prediction {
        let r_sat_m = [
            state.rx_km * 1000.0,
            state.ry_km * 1000.0,
            state.rz_km * 1000.0,
        ];
        let v_sat_m_s = [
            state.vx_km_s * 1000.0,
            state.vy_km_s * 1000.0,
            state.vz_km_s * 1000.0,
        ];
        let r_sta_m = [
            self.station_inertial_km[0] * 1000.0,
            self.station_inertial_km[1] * 1000.0,
            self.station_inertial_km[2] * 1000.0,
        ];

        let mut down_dt = 0.0_f64;
        for _ in 0..3 {
            let r_bounce = [
                r_sat_m[0] - v_sat_m_s[0] * down_dt,
                r_sat_m[1] - v_sat_m_s[1] * down_dt,
                r_sat_m[2] - v_sat_m_s[2] * down_dt,
            ];
            let d = sub_norm(&r_bounce, &r_sta_m);
            down_dt = d / C_M_S;
        }
        let r_bounce = [
            r_sat_m[0] - v_sat_m_s[0] * down_dt,
            r_sat_m[1] - v_sat_m_s[1] * down_dt,
            r_sat_m[2] - v_sat_m_s[2] * down_dt,
        ];

        let mut up_dt = 0.0_f64;
        for _ in 0..3 {
            let r_arrive = [
                r_bounce[0] + v_sat_m_s[0] * up_dt,
                r_bounce[1] + v_sat_m_s[1] * up_dt,
                r_bounce[2] + v_sat_m_s[2] * up_dt,
            ];
            let d = sub_norm(&r_arrive, &r_sta_m);
            up_dt = d / C_M_S;
        }
        let r_arrive = [
            r_bounce[0] + v_sat_m_s[0] * up_dt,
            r_bounce[1] + v_sat_m_s[1] * up_dt,
            r_bounce[2] + v_sat_m_s[2] * up_dt,
        ];

        let down = sub_norm(&r_bounce, &r_sta_m);
        let up = sub_norm(&r_arrive, &r_sta_m);
        let mut value = down + up + 2.0 * self.trop_bias_m;

        let mut entries = Vec::with_capacity(4);
        let los_down = unit(&[
            r_bounce[0] - r_sta_m[0],
            r_bounce[1] - r_sta_m[1],
            r_bounce[2] - r_sta_m[2],
        ]);
        let los_up = unit(&[
            r_arrive[0] - r_sta_m[0],
            r_arrive[1] - r_sta_m[1],
            r_arrive[2] - r_sta_m[2],
        ]);
        for i in 0..3 {
            // ∂range/∂r_sat — sum of downlink and uplink unit vectors,
            // converted to per-km because state position is in km.
            let g = (los_down[i] + los_up[i]) * 1000.0;
            entries.push((i, g));
        }

        if let Some(idx) = self.station_bias_index {
            let global = 6 + idx;
            value += extra.get(idx).copied().unwrap_or(0.0);
            entries.push((global, 1.0));
        }

        Prediction {
            value,
            partials: Partials { entries },
        }
    }

    fn sigma(&self) -> f64 {
        self.sigma_m
    }
}

fn sub_norm(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn unit(v: &[f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if n > 0.0 {
        [v[0] / n, v[1] / n, v[2] / n]
    } else {
        [0.0; 3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::time::JulianDate;

    #[test]
    fn predicts_two_way_range_at_rest() {
        // Stationary satellite 1000 km above the station along +z;
        // expected two-way range ≈ 2 000 000 m.
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            [0.0, 0.0, 7378.137],
            [0.0, 0.0, 0.0],
        );
        let m = SlrRangeModel::new([0.0, 0.0, 6378.137], 0.0, 0.01);
        let p = m.predict(&s, &[]);
        assert!((p.value - 2_000_000.0).abs() < 1e-3);
        assert_eq!(p.partials.entries.len(), 3);
    }

    #[test]
    fn partial_matches_finite_difference() {
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            [7100.0, 200.0, 50.0],
            [0.5, 7.5, 0.1],
        );
        let m = SlrRangeModel::new([6378.0, 100.0, 0.0], 0.0, 0.01);
        let pred = m.predict(&s, &[]);
        let h = 1e-3; // 1 m perturbation in km-units
        for (axis, idx) in [(0, 0), (1, 1), (2, 2)] {
            let mut up = s;
            let mut dn = s;
            match axis {
                0 => {
                    up.rx_km += h;
                    dn.rx_km -= h;
                }
                1 => {
                    up.ry_km += h;
                    dn.ry_km -= h;
                }
                _ => {
                    up.rz_km += h;
                    dn.rz_km -= h;
                }
            }
            let fd = (m.predict(&up, &[]).value - m.predict(&dn, &[]).value) / (2.0 * h);
            let analytic = pred
                .partials
                .entries
                .iter()
                .find(|(i, _)| *i == idx)
                .unwrap()
                .1;
            assert!(
                (fd - analytic).abs() < 1e-2,
                "axis {axis}: fd={fd}, analytic={analytic}"
            );
        }
    }
}
