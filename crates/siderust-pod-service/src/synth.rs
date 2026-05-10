//! Synthetic GNSS arc generator for end-to-end testing.
//!
//! Generates noise-free or noisy GPS pseudorange + carrier-phase observations
//! between a truth LEO orbit and a small constellation of static-orbit GPS
//! satellites. Both truth and GPS orbits are integrated with two-body
//! gravity only — sufficient for proving the WLS pipeline converges to the
//! truth state on a synthetic arc.

use crate::pipeline::{ArcEpoch, GpsSatellite};
use siderust::coordinates::frames::GCRS;
use siderust::time::JulianDate;
use siderust_pod_core::OrbitState;
use siderust_pod_core::{Position, Velocity, VelocityUnit};
use siderust_pod_dynamics::forces::TwoBody;
use siderust_pod_dynamics::integrators::rk4_propagate_series;
use siderust_pod_observations::gnss::{CarrierPhaseObs, GnssCodeModel, PseudorangeObs};
use siderust_pod_observations::model::MeasurementModel;

/// Configuration for a synthetic arc.
#[derive(Debug, Clone)]
pub struct SyntheticArcConfig {
    /// Truth LEO state at the arc start (epoch in TT).
    pub truth_initial: OrbitState,
    /// Step size, seconds.
    pub dt_s: f64,
    /// Number of steps (arc length = dt_s * n_steps).
    pub n_steps: usize,
    /// Number of GPS satellites to simulate (placed on circular orbits).
    pub n_gps_sats: usize,
    /// Standard deviation of code noise, metres.
    pub code_sigma_m: f64,
    /// Standard deviation of carrier noise, metres.
    pub carrier_sigma_m: f64,
    /// Receiver clock bias (truth), metres.
    pub clock_bias_m: f64,
    /// Carrier float ambiguity per satellite (truth), metres.
    pub ambiguity_m: f64,
    /// Random seed.
    pub seed: u64,
}

impl Default for SyntheticArcConfig {
    fn default() -> Self {
        let r0 = 6_378.137 + 500.0;
        let v0 = (398_600.441_8_f64 / r0).sqrt();
        Self {
            truth_initial: OrbitState::new(
                JulianDate::new(2_451_545.0),
                Position::new(r0, 0.0, 0.0),
                Velocity::new(0.0, v0, 0.0),
            ),
            dt_s: 30.0,
            n_steps: 120,
            n_gps_sats: 6,
            code_sigma_m: 0.5,
            carrier_sigma_m: 0.005,
            clock_bias_m: 12_345.6,
            ambiguity_m: 0.0,
            seed: 0xC0FFEE,
        }
    }
}

/// Synthetic arc bundle: truth states + per-epoch observations.
#[derive(Debug, Clone)]
pub struct SyntheticArc {
    /// Truth states at every epoch.
    pub truth_states: Vec<OrbitState>,
    /// One bundle per epoch.
    pub epochs: Vec<ArcEpoch>,
    /// GPS satellites used.
    pub gps_sats: Vec<GpsSatellite>,
    /// Truth receiver clock bias used.
    pub truth_clock_bias_m: f64,
}

fn gps_state_at(jd: JulianDate, slot: usize, n: usize) -> (Position<GCRS>, Velocity<GCRS, VelocityUnit>) {
    // Simple circular orbits at GPS altitude (~26 600 km), evenly spaced
    // in argument of latitude across two planes.
    let r = 26_600.0_f64;
    let mu = 398_600.441_8_f64;
    let n_mean = (mu / (r * r * r)).sqrt(); // rad/s
    let theta0 = 2.0 * std::f64::consts::PI * (slot as f64) / (n as f64);
    let inc: f64 = if slot % 2 == 0 { 0.95 } else { 1.05 }; // ~55°
    let dt = (jd.jd_value() - 2_451_545.0) * 86_400.0;
    let theta = theta0 + n_mean * dt;
    let cos_t = theta.cos();
    let sin_t = theta.sin();
    let (cos_i, sin_i) = (inc.cos(), inc.sin());
    let vmag = (mu / r).sqrt();
    (
        Position::<GCRS>::new(r * cos_t, r * sin_t * cos_i, r * sin_t * sin_i),
        Velocity::<GCRS, VelocityUnit>::new(-vmag * sin_t, vmag * cos_t * cos_i, vmag * cos_t * sin_i),
    )
}

/// Tiny linear-congruential PRNG (deterministic, no external dep).
struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1))
    }
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 11) as f64) / (1_u64 << 53) as f64
    }
    /// Standard normal via Box-Muller.
    fn normal(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-300);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// Generate the synthetic arc.
pub fn generate(cfg: &SyntheticArcConfig) -> SyntheticArc {
    let force = TwoBody::earth();
    let truth = rk4_propagate_series(&force, cfg.truth_initial, cfg.dt_s, cfg.n_steps);
    let mut rng = Lcg::new(cfg.seed);
    let gps_sats: Vec<GpsSatellite> = (0..cfg.n_gps_sats)
        .map(|i| GpsSatellite {
            id: format!("G{:02}", i + 1),
            slot: i,
        })
        .collect();
    let mut epochs = Vec::with_capacity(truth.len());
    for s in &truth {
        let mut code = Vec::new();
        let mut carrier = Vec::new();
        for sat in &gps_sats {
            let (gps_pos, gps_vel) = gps_state_at(s.epoch_tt, sat.slot, cfg.n_gps_sats);
            // Use the analytic prediction at *truth* state and add noise +
            // truth clock bias to obtain the synthetic measurement.
            let geom = code_truth_m(s, gps_pos, gps_vel) + cfg.clock_bias_m;
            let measured_code = geom + cfg.code_sigma_m * rng.normal();
            let measured_phase = geom + cfg.ambiguity_m + cfg.carrier_sigma_m * rng.normal();
            code.push((
                sat.clone(),
                PseudorangeObs {
                    gps_pos_km: gps_pos,
                    gps_vel_km_s: gps_vel,
                    measured_m: measured_code,
                    sigma_m: cfg.code_sigma_m,
                },
            ));
            carrier.push((
                sat.clone(),
                CarrierPhaseObs {
                    gps_pos_km: gps_pos,
                    gps_vel_km_s: gps_vel,
                    measured_m: measured_phase,
                    sigma_m: cfg.carrier_sigma_m,
                },
            ));
        }
        epochs.push(ArcEpoch {
            state_index: epochs.len(),
            code,
            carrier,
        });
    }
    SyntheticArc {
        truth_states: truth,
        epochs,
        gps_sats,
        truth_clock_bias_m: cfg.clock_bias_m,
    }
}

fn code_truth_m(state: &OrbitState, gps_pos: Position<GCRS>, gps_vel: Velocity<GCRS, VelocityUnit>) -> f64 {
    let model = GnssCodeModel {
        obs: PseudorangeObs {
            gps_pos_km: gps_pos,
            gps_vel_km_s: gps_vel,
            measured_m: 0.0,
            sigma_m: 1.0,
        },
        clock_bias_index: 0,
    };
    let p = model.predict(state, &[0.0]);
    p.value
}
