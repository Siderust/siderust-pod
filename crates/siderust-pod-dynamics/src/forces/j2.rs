//! J2 (Earth oblateness) perturbation acceleration in the inertial frame.

use super::ForceModel;
use siderust_pod_core::OrbitState;

/// J2 perturbation given GM, equatorial radius and the dimensionless `J2`
/// coefficient (positive — i.e. `J2 = -sqrt(5) * C20_normalised` already done).
#[derive(Debug, Clone, Copy)]
pub struct J2 {
    /// `GM` in km³/s².
    pub gm: f64,
    /// Equatorial radius in km.
    pub req_km: f64,
    /// Unnormalised `J2` coefficient. Earth: `1.082_626_68e-3`.
    pub j2: f64,
}

impl J2 {
    /// Standard Earth values.
    pub fn earth() -> Self {
        Self {
            gm: 398_600.441_8,
            req_km: 6_378.137,
            j2: 1.082_626_68e-3,
        }
    }
}

impl ForceModel for J2 {
    #[inline]
    fn acceleration(&self, s: &OrbitState) -> [f64; 3] {
        let r2 = s.r2();
        let r = r2.sqrt();
        let [rx, ry, rz] = s.position_km();
        let z2_over_r2 = (rz * rz) / r2;
        let factor = 1.5 * self.j2 * self.gm * self.req_km * self.req_km / (r2 * r2 * r);
        let cx = 5.0 * z2_over_r2 - 1.0;
        let cz = 5.0 * z2_over_r2 - 3.0;
        [
            factor * rx * cx,
            factor * ry * cx,
            factor * rz * cz,
        ]
    }
}
