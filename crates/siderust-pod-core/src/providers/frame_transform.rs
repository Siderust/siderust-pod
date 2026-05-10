//! Frame-transform provider trait.
//!
//! Encapsulates the full ITRF↔GCRF chain (precession, nutation, Earth rotation
//! angle, polar motion). Default implementations are wired in
//! `siderust-pod-dynamics::frames` once the EOP source is bound.

use crate::error::Result;
use siderust::time::JulianDate;

/// 3×3 rotation expressed as row-major column-stacked array.
pub type Matrix3 = [[f64; 3]; 3];

/// Provider for frame rotations between Earth-fixed and inertial frames.
///
/// The `at(jd_tt, jd_ut1)` API takes both TT and UT1 because the precession-
/// nutation rotation is parameterised by TT while ERA is parameterised by UT1.
pub trait FrameTransformProvider: Send + Sync {
    /// Rotation from ITRF to GCRF at the given TT/UT1 epoch.
    fn itrf_to_gcrf(&self, jd_tt: JulianDate, jd_ut1: JulianDate) -> Result<Matrix3>;

    /// Rotation from GCRF to ITRF at the given TT/UT1 epoch.
    fn gcrf_to_itrf(&self, jd_tt: JulianDate, jd_ut1: JulianDate) -> Result<Matrix3> {
        let m = self.itrf_to_gcrf(jd_tt, jd_ut1)?;
        Ok(transpose3(m))
    }
}

fn transpose3(m: Matrix3) -> Matrix3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}
