// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Atmospheric drag force model.
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
//! * `ρ(h)` is the atmospheric mass density provided by a [`DensityProvider`];
//! * `h` is the geocentric altitude `|r| − R_⊕` (km);
//! * `v_rel = v − ω_⊕ × r` is the inertial velocity corrected for
//!   the co-rotating atmosphere (Earth angular velocity
//!   ω_⊕ = (0, 0, 7.292 115 × 10⁻⁵) rad/s).
//!
//! This is a *minimum-viable* atmosphere: density only depends on the
//! magnitude of `r`, no diurnal/latitudinal/solar-activity effects. A
//! richer model (NRLMSISE-00 / DTM2000) belongs in a follow-up.

use crate::forces::ForceModel;
use siderust::astro::dynamics::atmosphere::{DensityProvider, ExponentialAtmosphere};
use siderust::qtty::Kilometers;
use siderust_pod_core::OrbitState;

/// Earth equatorial radius, km.
pub const R_EARTH_KM: f64 = 6_378.137;

/// Earth angular speed, rad/s (sidereal).
pub const OMEGA_EARTH_RAD_S: f64 = 7.292_115e-5;

/// Drag force model parameterised over any [`DensityProvider`].
///
/// Separate the atmosphere model from the spacecraft geometry so each can
/// be replaced independently.
#[derive(Debug, Clone)]
pub struct DragForce<D: DensityProvider> {
    /// Drag coefficient (dimensionless).
    pub cd: f64,
    /// Area-to-mass ratio, m² / kg.
    pub area_to_mass_m2_kg: f64,
    /// Atmosphere density provider.
    pub atmosphere: D,
}

impl DragForce<ExponentialAtmosphere> {
    /// Build a drag model using the [`ExponentialAtmosphere::LEO_500KM`] profile.
    pub fn leo_500km(cd: f64, area_to_mass_m2_kg: f64) -> Self {
        Self {
            cd,
            area_to_mass_m2_kg,
            atmosphere: ExponentialAtmosphere::LEO_500KM,
        }
    }
}

impl<D: DensityProvider> ForceModel for DragForce<D> {
    fn acceleration(
        &self,
        s: &OrbitState,
    ) -> siderust::astro::dynamics::state::Acceleration<
        siderust::coordinates::frames::GCRS,
        siderust::astro::dynamics::state::AccelerationUnit,
    > {
        type AccVec = siderust::astro::dynamics::state::Acceleration<
            siderust::coordinates::frames::GCRS,
            siderust::astro::dynamics::state::AccelerationUnit,
        >;
        let r = s.position.distance().value();
        let h = r - R_EARTH_KM;
        if h < 0.0 {
            return AccVec::new(0.0, 0.0, 0.0);
        }
        let rho = self.atmosphere.density_kg_m3(Kilometers::new(h));

        let [rx, ry, _] = [s.position.x().value(), s.position.y().value(), s.position.z().value()];
        let [vx, vy, vz] = [s.velocity.x().value(), s.velocity.y().value(), s.velocity.z().value()];

        let omega_cross_r = [-OMEGA_EARTH_RAD_S * ry, OMEGA_EARTH_RAD_S * rx, 0.0];
        let v_rel_km_s = [
            vx - omega_cross_r[0],
            vy - omega_cross_r[1],
            vz - omega_cross_r[2],
        ];

        let v2_km2_s2 = v_rel_km_s[0].powi(2) + v_rel_km_s[1].powi(2) + v_rel_km_s[2].powi(2);
        let v_mag_m_s = v2_km2_s2.sqrt() * 1_000.0;
        let pre_m_s2 = -0.5 * self.cd * self.area_to_mass_m2_kg * rho * v_mag_m_s;
        let pre_km_s2 = pre_m_s2;
        AccVec::new(
            pre_km_s2 * v_rel_km_s[0],
            pre_km_s2 * v_rel_km_s[1],
            pre_km_s2 * v_rel_km_s[2],
        )
    }
}

/// Type alias: drag model with the built-in exponential atmosphere.
pub type ExponentialDrag = DragForce<ExponentialAtmosphere>;

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
        let d = DragForce::leo_500km(2.2, 0.02);
        assert!(d.atmosphere.density_kg_m3(Kilometers::new(500.0)) > d.atmosphere.density_kg_m3(Kilometers::new(600.0)));
        assert!(d.atmosphere.density_kg_m3(Kilometers::new(400.0)) > d.atmosphere.density_kg_m3(Kilometers::new(500.0)));
    }

    #[test]
    fn drag_decays_altitude_monotonically() {
        let mu: f64 = 398_600.441_8;
        let r0: f64 = R_EARTH_KM + 350.0;
        let v0: f64 = (mu / r0).sqrt();
        let s0 = OrbitState::new(JulianDate::new(2_451_545.0), Position::new(r0, 0.0, 0.0), Velocity::new(0.0, v0, 0.0));
        let force = CompositeForce::empty()
            .push(Box::new(TwoBody::earth()))
            .push(Box::new(DragForce {
                cd: 2.2,
                area_to_mass_m2_kg: 5.0,
                atmosphere: ExponentialAtmosphere {
                    rho0_kg_m3: 1.0e-11,
                    h0_km: 350.0,
                    scale_height_km: 50.0,
                },
            }));
        let s_end = rk4_propagate(&force, s0, 30.0, 1440);
        let r_end_x = s_end.position.x().value();
        let r_end_y = s_end.position.y().value();
        let r_end_z = s_end.position.z().value();
        let r_end = (r_end_x.powi(2) + r_end_y.powi(2) + r_end_z.powi(2)).sqrt();
        assert!(
            r_end < r0,
            "expected drag-driven decay; r0={r0:.3}, r_end={r_end:.3}",
        );
    }
}
