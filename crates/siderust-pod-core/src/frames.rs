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
//! All bases are returned as [`Rotation3`] matrices whose rows are the basis
//! vectors expressed in the inertial frame, so an inertial vector `v_i`
//! becomes `R * v_i` in the local frame.

use crate::state::OrbitState;

/// A 3×3 rotation matrix (row-major) from the [`affn`] crate.
///
/// Re-exported so callers do not need a direct `affn` dependency.
pub use affn::Rotation3;

/// Normalised cross product of two unit vectors, returned as `[f64; 3]`.
///
/// Both inputs must be non-zero and non-parallel.  Using raw arrays avoids the
/// `affn` version split that would otherwise cause a type mismatch between the
/// `Direction` exported by the local `affn` path dep and the `Direction` that
/// `siderust` (which depends on the crates.io `affn`) returns.
fn cross3_unit(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    let cx = a[1] * b[2] - a[2] * b[1];
    let cy = a[2] * b[0] - a[0] * b[2];
    let cz = a[0] * b[1] - a[1] * b[0];
    let n = (cx * cx + cy * cy + cz * cz).sqrt();
    [cx / n, cy / n, cz / n]
}

/// Normalise a `[f64; 3]` vector.
fn unit3(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / n, v[1] / n, v[2] / n]
}

/// Build the inertial → RTN rotation matrix from an orbit state.
///
/// Returns `R_i2rtn` such that `v_rtn = R * v_i`.
pub fn rtn_from_state(s: &OrbitState) -> Rotation3 {
    // r̂: radial unit direction from geocenter to satellite.
    let r_hat = s.position.direction_unchecked().as_array();
    // v̂: velocity unit direction (manual normalise — Per<Km,s> is not LengthUnit).
    let v_hat = unit3([
        s.velocity.x().value(),
        s.velocity.y().value(),
        s.velocity.z().value(),
    ]);
    // ĥ = normalize(r̂ × v̂): orbit-normal direction.
    let h_hat = cross3_unit(r_hat, v_hat);
    // t̂ = ĥ × r̂: transverse (along-track) direction.
    let t_hat = cross3_unit(h_hat, r_hat);
    Rotation3::from_matrix_unchecked([r_hat, t_hat, h_hat])
}

/// Build the inertial → VNC rotation matrix from an orbit state.
pub fn vnc_from_state(s: &OrbitState) -> Rotation3 {
    // v̂: velocity unit direction.
    let v_hat = unit3([
        s.velocity.x().value(),
        s.velocity.y().value(),
        s.velocity.z().value(),
    ]);
    // r̂: radial unit direction.
    let r_hat = s.position.direction_unchecked().as_array();
    // n̂ = normalize(r̂ × v̂): orbit normal.
    let n_hat = cross3_unit(r_hat, v_hat);
    // ĉ = v̂ × n̂: co-normal direction.
    let c_hat = cross3_unit(v_hat, n_hat);
    Rotation3::from_matrix_unchecked([v_hat, n_hat, c_hat])
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
        let m = r.as_matrix();
        // First row should be +x.
        assert!((m[0][0] - 1.0).abs() < 1e-12);
        // Third row should be +z (orbit normal).
        assert!((m[2][2] - 1.0).abs() < 1e-12);
        // Second row should be +y.
        assert!((m[1][1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn vnc_basis_aligned_with_velocity_for_circular_orbit() {
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            Position::new(7000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        );
        let r = vnc_from_state(&s);
        let m = r.as_matrix();
        assert!((m[0][1] - 1.0).abs() < 1e-12);
    }
}

