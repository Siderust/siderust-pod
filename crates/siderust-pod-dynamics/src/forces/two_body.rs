//! Two-body central-gravity force.

use super::ForceModel;
use siderust_pod_core::OrbitState;

/// Newtonian central-gravity acceleration `−μ r / |r|³`.
#[derive(Debug, Clone, Copy)]
pub struct TwoBody {
    /// Gravitational parameter `GM` in km³/s².
    pub gm: f64,
}

impl TwoBody {
    /// Earth two-body field with EGM2008 GM.
    pub fn earth() -> Self {
        Self { gm: 398_600.441_8 }
    }
}

impl ForceModel for TwoBody {
    #[inline]
    fn acceleration(&self, s: &OrbitState) -> [f64; 3] {
        let r2 = s.r2();
        let r = r2.sqrt();
        let k = -self.gm / (r2 * r);
        let [rx, ry, rz] = s.position_km();
        [k * rx, k * ry, k * rz]
    }
}
