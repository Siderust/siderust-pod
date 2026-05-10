//! GNSS pseudorange and carrier-phase models for MVP-1.
//!
//! These are intentionally compact: a single-frequency, troposphere-free,
//! one-way geometry between a known GPS satellite Cartesian position
//! (provided by the caller from SP3 / synthetic generator) and the LEO
//! receiver position carried inside the `OrbitState`. They include:
//!
//! - geometric range,
//! - receiver clock bias (parameter index `clock_bias_index`),
//! - Sagnac correction (light-time iteration is *not* performed; we use
//!   a one-step Sagnac approximation suitable for low Earth orbiters
//!   and a 30 s sampling),
//! - relativistic GPS satellite clock correction (Δt = −2 r·v / c²),
//! - float carrier ambiguity (parameter index `ambiguity_index`).
//!
//! Tropospheric and ionospheric delays are zero in MVP-1; they are added in
//! M3 follow-up tasks together with a real ITRF-vs-GCRF rotation supplied by
//! the [`siderust_pod_core::providers::FrameTransformProvider`].

use crate::model::{MeasurementModel, Partials, Prediction};
use siderust_pod_core::{Position, Velocity, VelocityUnit, OrbitState};
use siderust::coordinates::frames::GCRS;

/// Speed of light, m/s.
pub const C_M_S: f64 = 299_792_458.0;
/// Speed of light, km/s.
pub const C_KM_S: f64 = C_M_S / 1_000.0;
/// Earth rotation rate, rad/s.
pub const OMEGA_EARTH_RAD_S: f64 = 7.292_115_146_706_979e-5;

/// Pseudorange observation between a LEO receiver and a GPS satellite.
#[derive(Debug, Clone, Copy)]
pub struct PseudorangeObs {
    /// GPS satellite GCRF position at signal-emission instant, km.
    pub gps_pos_km: Position<GCRS>,
    /// GPS satellite GCRF velocity, km/s. Used for the relativistic correction.
    pub gps_vel_km_s: Velocity<GCRS, VelocityUnit>,
    /// Measured pseudorange, metres.
    pub measured_m: f64,
    /// Measurement standard deviation, metres.
    pub sigma_m: f64,
}

/// Carrier-phase observation in metres (already scaled by wavelength).
#[derive(Debug, Clone, Copy)]
pub struct CarrierPhaseObs {
    /// GPS satellite GCRF position, km.
    pub gps_pos_km: Position<GCRS>,
    /// GPS satellite GCRF velocity, km/s.
    pub gps_vel_km_s: Velocity<GCRS, VelocityUnit>,
    /// Measured carrier-phase range, metres.
    pub measured_m: f64,
    /// Measurement standard deviation, metres.
    pub sigma_m: f64,
}

/// GNSS pseudorange model. Carries indices into the parameter vector for the
/// receiver clock bias.
#[derive(Debug, Clone)]
pub struct GnssCodeModel {
    /// Observation data.
    pub obs: PseudorangeObs,
    /// Index, in `extra_params`, of the receiver clock bias (metres).
    pub clock_bias_index: usize,
}

/// GNSS carrier-phase model. Carries indices into the parameter vector for
/// the receiver clock bias and the per-pass float ambiguity.
#[derive(Debug, Clone)]
pub struct GnssCarrierModel {
    /// Observation data.
    pub obs: CarrierPhaseObs,
    /// Index, in `extra_params`, of the receiver clock bias (metres).
    pub clock_bias_index: usize,
    /// Index, in `extra_params`, of the float ambiguity (metres).
    pub ambiguity_index: usize,
}

#[inline]
fn diff(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn sagnac_km(gps_pos_km: Position<GCRS>, rx_pos_km: [f64; 3]) -> f64 {
    OMEGA_EARTH_RAD_S
        * (gps_pos_km.x().value() * rx_pos_km[1] - gps_pos_km.y().value() * rx_pos_km[0])
        / C_KM_S
}

fn relativistic_gps_clock_m(pos_km: Position<GCRS>, vel_km_s: Velocity<GCRS, VelocityUnit>) -> f64 {
    let dot = pos_km.x().value() * vel_km_s.x().value()
        + pos_km.y().value() * vel_km_s.y().value()
        + pos_km.z().value() * vel_km_s.z().value();
    -2.0 * dot * 1_000.0 / C_M_S
}

/// Predict a pseudorange in metres given the current LEO state and clock bias.
fn predict_range_m(
    state: &OrbitState,
    gps_pos_km: Position<GCRS>,
    gps_vel_km_s: Velocity<GCRS, VelocityUnit>,
) -> (f64, [f64; 3]) {
    let rx_km = [state.position.x().value(), state.position.y().value(), state.position.z().value()];
    let gps = [gps_pos_km.x().value(), gps_pos_km.y().value(), gps_pos_km.z().value()];
    let los = diff(rx_km, gps);
    let geom_km = norm(los);
    let geom_m = geom_km * 1_000.0;
    let sagnac_m = sagnac_km(gps_pos_km, rx_km) * 1_000.0;
    let rel_m = relativistic_gps_clock_m(gps_pos_km, gps_vel_km_s);
    let u = if geom_km > 0.0 {
        [los[0] / geom_km, los[1] / geom_km, los[2] / geom_km]
    } else {
        [0.0; 3]
    };
    (geom_m + sagnac_m + rel_m, u)
}

impl MeasurementModel for GnssCodeModel {
    fn predict(&self, state: &OrbitState, extra_params: &[f64]) -> Prediction {
        let clk_m = extra_params
            .get(self.clock_bias_index)
            .copied()
            .unwrap_or(0.0);
        let (range_m, u) = predict_range_m(state, self.obs.gps_pos_km, self.obs.gps_vel_km_s);
        let value = range_m + clk_m;
        // Partials w.r.t. position (km → metres factor): d/dr (geom_m) = u * 1000.
        let partials = Partials::from_pairs(vec![
            (0, u[0] * 1_000.0),
            (1, u[1] * 1_000.0),
            (2, u[2] * 1_000.0),
            (self.clock_bias_index + 6, 1.0),
        ]);
        Prediction { value, partials }
    }
    fn sigma(&self) -> f64 {
        self.obs.sigma_m
    }
}

impl MeasurementModel for GnssCarrierModel {
    fn predict(&self, state: &OrbitState, extra_params: &[f64]) -> Prediction {
        let clk_m = extra_params
            .get(self.clock_bias_index)
            .copied()
            .unwrap_or(0.0);
        let amb_m = extra_params
            .get(self.ambiguity_index)
            .copied()
            .unwrap_or(0.0);
        let (range_m, u) = predict_range_m(state, self.obs.gps_pos_km, self.obs.gps_vel_km_s);
        let value = range_m + clk_m + amb_m;
        let partials = Partials::from_pairs(vec![
            (0, u[0] * 1_000.0),
            (1, u[1] * 1_000.0),
            (2, u[2] * 1_000.0),
            (self.clock_bias_index + 6, 1.0),
            (self.ambiguity_index + 6, 1.0),
        ]);
        Prediction { value, partials }
    }
    fn sigma(&self) -> f64 {
        self.obs.sigma_m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::time::JulianDate;
    use siderust_pod_core::{Position, Velocity};

    #[test]
    fn pseudorange_partials_match_finite_difference() {
        let state = OrbitState::new(
            JulianDate::new(2_451_545.0),
            Position::new(7000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        );
        let gps_pos = Position::<GCRS>::new(26_000.0, 1_000.0, 5_000.0);
        let gps_vel = Velocity::<GCRS, VelocityUnit>::new(0.0, 3.0, 0.0);
        let model = GnssCodeModel {
            obs: PseudorangeObs {
                gps_pos_km: gps_pos,
                gps_vel_km_s: gps_vel,
                measured_m: 0.0,
                sigma_m: 1.0,
            },
            clock_bias_index: 0,
        };
        let extra = [0.0];
        let p0 = model.predict(&state, &extra);
        let h = 1e-3;
        for i in 0..3 {
            let rx = state.position.x().value() + if i == 0 { h } else { 0.0 };
            let ry = state.position.y().value() + if i == 1 { h } else { 0.0 };
            let rz = state.position.z().value() + if i == 2 { h } else { 0.0 };
            let s = OrbitState::new(
                state.epoch_tt,
                Position::new(rx, ry, rz),
                state.velocity,
            );
            let p1 = model.predict(&s, &extra);
            let fd = (p1.value - p0.value) / h;
            let analytic = p0.partials.entries.iter().find(|(j, _)| *j == i).unwrap().1;
            assert!(
                (fd - analytic).abs() / analytic.abs() < 1e-3,
                "partial[{i}] fd={fd} analytic={analytic}"
            );
        }
    }
}
