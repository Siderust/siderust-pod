//! Third-body point-mass perturbations from Sun and Moon.
//!
//! Treats each perturbing body as a point mass at a known geocentric position
//! `d` (km, J2000 mean equator). The resulting acceleration on a satellite at
//! geocentric position `r` is the standard Battin form:
//!
//! ```text
//! a = μ_b · ( (d − r) / |d − r|³ − d / |d|³ )
//! ```
//!
//! The geocentric body positions are fetched from an
//! [`EphemerisProvider`]; ecliptic-of-J2000 positions are rotated by the
//! mean obliquity into the (mean equator of) J2000 frame, which we treat
//! as GCRF for MVP-1 modelling. A future revision will route through
//! [`siderust_pod_core::providers::FrameTransformProvider`] for the full
//! IAU-2006/2000A chain.

use crate::forces::ForceModel;
use siderust::time::JulianDate;
use siderust_pod_core::providers::EphemerisProvider;
use siderust_pod_core::OrbitState;
use std::sync::Arc;

/// Standard gravitational parameter of the Sun (km³/s²).
pub const MU_SUN_KM3_S2: f64 = 1.327_124_400_18e11;
/// Standard gravitational parameter of the Moon (km³/s²).
pub const MU_MOON_KM3_S2: f64 = 4.902_800_066e3;

/// Mean obliquity of the ecliptic at J2000 (radians).
const EPS_J2000_RAD: f64 = 0.409_092_804_222_329_5;

/// Astronomical unit, km.
const AU_KM: f64 = 149_597_870.7;

/// Third-body force from Sun + Moon, sourced from an [`EphemerisProvider`].
pub struct ThirdBodySunMoon {
    provider: Arc<dyn EphemerisProvider>,
}

impl ThirdBodySunMoon {
    /// Build from any boxed/arc provider implementation.
    pub fn new(provider: Arc<dyn EphemerisProvider>) -> Self {
        Self { provider }
    }

    fn sun_geocentric_km(&self, jd: JulianDate) -> [f64; 3] {
        // Sun position relative to Earth, expressed in ecliptic mean J2000, AU.
        let sun_b = self.provider.sun_barycentric(jd);
        let earth_b = self.provider.earth_barycentric(jd);
        let (sun_b, earth_b) = match (sun_b, earth_b) {
            (Ok(a), Ok(b)) => (a, b),
            _ => return [0.0; 3],
        };
        let s = position_xyz_au(&sun_b);
        let e = position_xyz_au(&earth_b);
        let d_ecl_au = [s[0] - e[0], s[1] - e[1], s[2] - e[2]];
        let d_eq_au = rotx_to_equator(d_ecl_au);
        [d_eq_au[0] * AU_KM, d_eq_au[1] * AU_KM, d_eq_au[2] * AU_KM]
    }

    fn moon_geocentric_km(&self, jd: JulianDate) -> [f64; 3] {
        let m_geo = match self.provider.moon_geocentric(jd) {
            Ok(p) => p,
            Err(_) => return [0.0; 3],
        };
        let m = position_xyz_km(&m_geo);
        rotx_to_equator(m)
    }
}

impl ForceModel for ThirdBodySunMoon {
    fn acceleration(&self, s: &OrbitState) -> [f64; 3] {
        let r = [s.position.x().value(), s.position.y().value(), s.position.z().value()];
        let mut a = [0.0; 3];
        for (mu, d) in [
            (MU_SUN_KM3_S2, self.sun_geocentric_km(s.epoch_tt)),
            (MU_MOON_KM3_S2, self.moon_geocentric_km(s.epoch_tt)),
        ] {
            let dr = [d[0] - r[0], d[1] - r[1], d[2] - r[2]];
            let dr_n = norm3(dr);
            let d_n = norm3(d);
            if dr_n == 0.0 || d_n == 0.0 {
                continue;
            }
            let dr3 = dr_n * dr_n * dr_n;
            let d3 = d_n * d_n * d_n;
            for i in 0..3 {
                a[i] += mu * (dr[i] / dr3 - d[i] / d3);
            }
        }
        a
    }
}

#[inline]
fn norm3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Rotate a vector around the X axis by `+ε` to convert ecliptic of J2000
/// into the mean equator of J2000.
fn rotx_to_equator(v: [f64; 3]) -> [f64; 3] {
    let (s, c) = EPS_J2000_RAD.sin_cos();
    [v[0], c * v[1] - s * v[2], s * v[1] + c * v[2]]
}

// --- helpers extracting f64 vectors from typed siderust positions ----------

fn position_xyz_au<C, F>(
    p: &siderust::coordinates::cartesian::Position<C, F, siderust::qtty::AstronomicalUnit>,
) -> [f64; 3]
where
    C: siderust::coordinates::centers::ReferenceCenter,
    F: siderust::coordinates::frames::ReferenceFrame,
{
    [p.x().value(), p.y().value(), p.z().value()]
}

fn position_xyz_km<C, F>(
    p: &siderust::coordinates::cartesian::Position<C, F, siderust::qtty::Kilometer>,
) -> [f64; 3]
where
    C: siderust::coordinates::centers::ReferenceCenter,
    F: siderust::coordinates::frames::ReferenceFrame,
{
    [p.x().value(), p.y().value(), p.z().value()]
}
