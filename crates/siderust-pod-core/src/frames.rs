//! Local orbital frames built from a Cartesian inertial state.
//!
//! - **RTN (RIC)**: radial / along-track-ish (transverse) / cross-track. The
//!   `T` axis is *not* the velocity direction; it is `N × R` so the frame is
//!   right-handed and `R` is exactly along position.
//! - **LVLH**: local-vertical / local-horizontal. Differs from RTN only in
//!   axis ordering and signs depending on convention. We use the
//!   "Z down (radial inward), X along velocity-projection, Y = Z × X"
//!   convention.
//! - **VNC**: velocity / normal / co-normal. `V` is along velocity, `N` is
//!   the orbit normal (= r×v), `C = V × N`.
//!
//! All bases are returned as 3×3 row-major rotation matrices `R` whose rows
//! are the basis vectors expressed in the inertial frame, so an inertial
//! vector `v_i` becomes `R · v_i` in the local frame.

use crate::state::OrbitState;

/// A 3×3 rotation matrix (row-major).
pub type Rotation3 = [[f64; 3]; 3];

#[inline]
fn norm(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = norm(v);
    if n == 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / n, v[1] / n, v[2] / n]
    }
}

#[inline]
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Build the inertial → RTN rotation matrix from an orbit state.
///
/// Returns `R_i2rtn` such that `v_rtn = R · v_i`.
pub fn rtn_from_state(s: &OrbitState) -> Rotation3 {
    let r = unit(s.position_km());
    let h = unit(cross(s.position_km(), s.velocity_km_s()));
    let t = cross(h, r);
    [r, t, h]
}

/// Build the inertial → VNC rotation matrix from an orbit state.
pub fn vnc_from_state(s: &OrbitState) -> Rotation3 {
    let v = unit(s.velocity_km_s());
    let n = unit(cross(s.position_km(), s.velocity_km_s()));
    let c = cross(v, n);
    [v, n, c]
}

/// Apply a rotation to a 3-vector.
#[inline]
pub fn apply3(r: &Rotation3, v: [f64; 3]) -> [f64; 3] {
    [
        r[0][0] * v[0] + r[0][1] * v[1] + r[0][2] * v[2],
        r[1][0] * v[0] + r[1][1] * v[1] + r[1][2] * v[2],
        r[2][0] * v[0] + r[2][1] * v[1] + r[2][2] * v[2],
    ]
}

/// Transpose a 3×3 rotation matrix.
#[inline]
pub fn transpose3(r: &Rotation3) -> Rotation3 {
    [
        [r[0][0], r[1][0], r[2][0]],
        [r[0][1], r[1][1], r[2][1]],
        [r[0][2], r[1][2], r[2][2]],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::time::JulianDate;
    use crate::state::{Position, Velocity};

    #[test]
    fn rtn_basis_orthonormal_for_circular_orbit() {
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            Position::new(7000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        );
        let r = rtn_from_state(&s);
        // First row should be +x.
        assert!((r[0][0] - 1.0).abs() < 1e-12);
        // Third row should be +z (orbit normal).
        assert!((r[2][2] - 1.0).abs() < 1e-12);
        // Second row should be +y.
        assert!((r[1][1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn vnc_basis_aligned_with_velocity_for_circular_orbit() {
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            Position::new(7000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        );
        let r = vnc_from_state(&s);
        assert!((r[0][1] - 1.0).abs() < 1e-12);
    }
}
