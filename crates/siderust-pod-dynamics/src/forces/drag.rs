//! Atmospheric drag with an exponential-density atmosphere.
//!
//! Implements the standard cannonball drag acceleration
//!
//! ```text
//! a_drag = − ½ · Cd · (A/m) · ρ(h) · |v_rel| · v_rel
//! ```
//!
//! where:
//!
//! * `Cd` is the drag coefficient (typical LEO value ≈ 2.2);
//! * `A/m` is the area-to-mass ratio in m² / kg;
//! * `ρ(h) = ρ₀ · exp(−(h − h₀) / H)` is an exponential atmosphere with
//!   reference altitude `h₀` (km), scale height `H` (km), and reference
//!   density `ρ₀` (kg/m³);
//! * `h` is the geocentric altitude `|r| − R_⊕` (km);
//! * `v_rel = v − ω_⊕ × r` is the inertial velocity corrected for
//!   co-rotating atmosphere (Earth angular velocity vector
//!   ω_⊕ = (0, 0, 7.292 115 × 10⁻⁵) rad/s).
//!
//! This is a *minimum-viable* atmosphere: density only depends on the
//! magnitude of `r`, no diurnal/latitudinal/solar-activity effects. A
//! richer model (NRLMSISE-00 / DTM2000) belongs in a follow-up.

use crate::forces::ForceModel;
use siderust_pod_core::OrbitState;

/// Earth equatorial radius, km.
pub const R_EARTH_KM: f64 = 6_378.137;

/// Earth angular speed, rad/s (sidereal).
pub const OMEGA_EARTH_RAD_S: f64 = 7.292_115e-5;

/// Exponential-atmosphere drag.
#[derive(Debug, Clone, Copy)]
pub struct ExponentialDrag {
    /// Drag coefficient (dimensionless).
    pub cd: f64,
    /// Area-to-mass ratio, m² / kg.
    pub area_to_mass_m2_kg: f64,
    /// Reference density, kg / m³.
    pub rho0_kg_m3: f64,
    /// Reference altitude, km.
    pub h0_km: f64,
    /// Scale height, km.
    pub scale_height_km: f64,
}

impl ExponentialDrag {
    /// "USSA-like" defaults near 500 km altitude. The numbers here are
    /// representative, not authoritative — production runs should
    /// inject a calibrated table or NRLMSISE-00.
    pub fn leo_500km(cd: f64, area_to_mass_m2_kg: f64) -> Self {
        Self {
            cd,
            area_to_mass_m2_kg,
            rho0_kg_m3: 6.967e-13,
            h0_km: 500.0,
            scale_height_km: 63.822,
        }
    }

    /// Density model — exposed to keep tests honest.
    #[inline]
    pub fn density_kg_m3(&self, altitude_km: f64) -> f64 {
        self.rho0_kg_m3 * (-(altitude_km - self.h0_km) / self.scale_height_km).exp()
    }
}

impl ForceModel for ExponentialDrag {
    fn acceleration(&self, s: &OrbitState) -> [f64; 3] {
        // Geocentric altitude.
        let r = s.r2().sqrt();
        let h = r - R_EARTH_KM;
        if h < 0.0 {
            return [0.0; 3];
        }
        let rho = self.density_kg_m3(h);

        let [rx, ry, _] = s.position_km();
        let [vx, vy, vz] = s.velocity_km_s();

        // v_rel = v − ω × r.   ω = (0, 0, ω_⊕).
        let omega_cross_r = [-OMEGA_EARTH_RAD_S * ry, OMEGA_EARTH_RAD_S * rx, 0.0];
        let v_rel_km_s = [
            vx - omega_cross_r[0],
            vy - omega_cross_r[1],
            vz - omega_cross_r[2],
        ];

        // Convert km/s → m/s for the dynamic-pressure expression, then back.
        // |v_rel| in m/s; v_rel in m/s; ρ in kg/m³; A/m in m²/kg.
        // a_m_s2 = −0.5 · Cd · A/m · ρ · |v| · v
        let v2_km2_s2 = v_rel_km_s[0].powi(2) + v_rel_km_s[1].powi(2) + v_rel_km_s[2].powi(2);
        let v_mag_m_s = v2_km2_s2.sqrt() * 1_000.0;
        let pre_m_s2 = -0.5 * self.cd * self.area_to_mass_m2_kg * rho * v_mag_m_s;
        // v_rel in m/s = v_rel_km_s * 1000.
        // We want km/s², so divide overall by 1000.
        let pre_km_s2 = pre_m_s2; // pre_m_s2 * (m/s) = m/s² accumulated below
        [
            pre_km_s2 * v_rel_km_s[0], // m/s² (since v_rel_km_s * 1000 cancels with /1000)
            pre_km_s2 * v_rel_km_s[1],
            pre_km_s2 * v_rel_km_s[2],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forces::two_body::TwoBody;
    use crate::forces::CompositeForce;
    use crate::integrators::rk4_propagate;
    use siderust::time::JulianDate;
    use siderust_pod_core::{Position, Velocity};

    #[test]
    fn density_decreases_with_altitude() {
        let d = ExponentialDrag::leo_500km(2.2, 0.02);
        assert!(d.density_kg_m3(500.0) > d.density_kg_m3(600.0));
        assert!(d.density_kg_m3(400.0) > d.density_kg_m3(500.0));
    }

    #[test]
    fn drag_decays_altitude_monotonically() {
        // Two-body + heavy drag should secularly reduce semi-major axis,
        // so the post-propagation radius should be smaller than the
        // initial radius after a few orbital revolutions.
        let mu: f64 = 398_600.441_8;
        let r0: f64 = R_EARTH_KM + 350.0; // very-low LEO
        let v0: f64 = (mu / r0).sqrt();
        let s0 = OrbitState::new(JulianDate::new(2_451_545.0), Position::new(r0, 0.0, 0.0), Velocity::new(0.0, v0, 0.0));
        // Exaggerated A/m to make decay visible in a short integration.
        let force = CompositeForce::empty()
            .push(Box::new(TwoBody::earth()))
            .push(Box::new(ExponentialDrag {
                cd: 2.2,
                area_to_mass_m2_kg: 5.0, // very draggy spacecraft
                rho0_kg_m3: 1.0e-11,     // amplified
                h0_km: 350.0,
                scale_height_km: 50.0,
            }));
        // 12 hours at 30 s.
        let s_end = rk4_propagate(&force, s0, 30.0, 1440);
        let [r_end_x, r_end_y, r_end_z] = s_end.position_km();
        let r_end = (r_end_x.powi(2) + r_end_y.powi(2) + r_end_z.powi(2)).sqrt();
        assert!(
            r_end < r0,
            "expected drag-driven decay; r0={r0:.3}, r_end={r_end:.3}",
        );
    }
}
