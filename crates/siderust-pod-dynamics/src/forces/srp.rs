//! Cannonball solar radiation pressure (SRP).
//!
//! Implements the classical "cannonball" SRP model
//!
//! ```text
//! a_srp = − Cr · P0 · (AU / |r_sun_sat|)² · (A/m) · r̂_sun_sat
//! ```
//!
//! where:
//!
//! * `r_sun_sat = r_sat − r_sun` is the spacecraft-to-Sun vector in km;
//! * `r̂_sun_sat` is the corresponding unit vector pointing *from the Sun
//!   toward the satellite* (so the resulting acceleration pushes the
//!   spacecraft *away* from the Sun, as a positive `Cr` should);
//! * `P0 = 4.560 × 10⁻⁶ N/m²` is the solar radiation pressure at 1 AU;
//! * `A/m` is the area-to-mass ratio in m² / kg;
//! * `Cr` is the radiation-pressure coefficient (typically 1.0–1.5).
//!
//! The Sun position is obtained from the supplied
//! [`EphemerisProvider`] in the same convention as
//! [`super::ThirdBodySunMoon`] (mean equator of J2000, treated as GCRF
//! for MVP-grade modelling).
//!
//! Eclipse modelling is *not* yet included — the Sun is always treated
//! as visible. A conical Earth-shadow model belongs in a follow-up.

use crate::forces::ForceModel;
use siderust::time::JulianDate;
use siderust_pod_core::providers::EphemerisProvider;
use siderust_pod_core::OrbitState;
use std::sync::Arc;

/// Solar radiation pressure at 1 AU, N/m².
pub const P0_N_M2: f64 = 4.560e-6;

/// Astronomical unit, km.
const AU_KM: f64 = 149_597_870.7;

/// Mean obliquity of the ecliptic at J2000, rad.
const EPS_J2000_RAD: f64 = 0.409_092_804_222_329_5;

/// Cannonball SRP force model.
pub struct CannonballSrp {
    provider: Arc<dyn EphemerisProvider>,
    /// Radiation-pressure coefficient (dimensionless).
    pub cr: f64,
    /// Area-to-mass ratio in m² / kg.
    pub area_to_mass_m2_kg: f64,
}

impl CannonballSrp {
    /// Build a cannonball SRP model.
    pub fn new(provider: Arc<dyn EphemerisProvider>, cr: f64, area_to_mass_m2_kg: f64) -> Self {
        Self {
            provider,
            cr,
            area_to_mass_m2_kg,
        }
    }

    fn sun_geocentric_km(&self, jd: JulianDate) -> Option<[f64; 3]> {
        let sun_b = self.provider.sun_barycentric(jd).ok()?;
        let earth_b = self.provider.earth_barycentric(jd).ok()?;
        let s = [sun_b.x().value(), sun_b.y().value(), sun_b.z().value()];
        let e = [
            earth_b.x().value(),
            earth_b.y().value(),
            earth_b.z().value(),
        ];
        let d_ecl_au = [s[0] - e[0], s[1] - e[1], s[2] - e[2]];
        let (sn, cs) = EPS_J2000_RAD.sin_cos();
        let d_eq_au = [
            d_ecl_au[0],
            cs * d_ecl_au[1] - sn * d_ecl_au[2],
            sn * d_ecl_au[1] + cs * d_ecl_au[2],
        ];
        Some([d_eq_au[0] * AU_KM, d_eq_au[1] * AU_KM, d_eq_au[2] * AU_KM])
    }
}

impl ForceModel for CannonballSrp {
    fn acceleration(&self, s: &OrbitState) -> [f64; 3] {
        let Some(sun) = self.sun_geocentric_km(s.epoch_tt) else {
            return [0.0; 3];
        };
        let [rx, ry, rz] = s.position_km();
        let r_sun_sat_km = [rx - sun[0], ry - sun[1], rz - sun[2]];
        let r2 = r_sun_sat_km[0] * r_sun_sat_km[0]
            + r_sun_sat_km[1] * r_sun_sat_km[1]
            + r_sun_sat_km[2] * r_sun_sat_km[2];
        let r = r2.sqrt();
        if r == 0.0 {
            return [0.0; 3];
        }
        // Convert N/m² · m²/kg = m/s²; we want km/s² so divide by 1000.
        let mag_km_s2 =
            self.cr * P0_N_M2 * (AU_KM * AU_KM / r2) * self.area_to_mass_m2_kg / 1_000.0;
        let inv_r = 1.0 / r;
        [
            mag_km_s2 * r_sun_sat_km[0] * inv_r,
            mag_km_s2 * r_sun_sat_km[1] * inv_r,
            mag_km_s2 * r_sun_sat_km[2] * inv_r,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust_pod_core::providers::ephemeris::Vsop87Provider;

    #[test]
    fn srp_acceleration_has_expected_order_of_magnitude_at_leo() {
        // Cr=1.5, A/m=0.02 m²/kg ⇒ |a_srp| ≈ 1.5 · 4.56e-6 · 0.02 / 1000
        //                               ≈ 1.4e-10 km/s²   (~1.4e-7 m/s²)
        let p = std::sync::Arc::new(Vsop87Provider);
        let srp = CannonballSrp::new(p, 1.5, 0.02);
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            [7000.0, 0.0, 0.0],
            [0.0, 7.5, 0.0],
        );
        let a = srp.acceleration(&s);
        let mag = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        assert!(
            (5e-11..5e-10).contains(&mag),
            "SRP magnitude out of expected band: {mag} km/s²",
        );
    }

    #[test]
    fn srp_zero_when_area_is_zero() {
        let p = std::sync::Arc::new(Vsop87Provider);
        let srp = CannonballSrp::new(p, 1.5, 0.0);
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            [7000.0, 0.0, 0.0],
            [0.0, 7.5, 0.0],
        );
        let a = srp.acceleration(&s);
        assert!(a.iter().all(|x| *x == 0.0));
    }
}
