//! Local orbital frames built from a Cartesian inertial state.
//!
//! ## Frame types
//!
//! [`RTN`], [`VNC`], and [`LVLH`] are zero-sized marker types that implement
//! [`affn::frames::ReferenceFrame`], following the same pattern as the
//! astronomical frames in `affn` (GCRS, ICRS, TEME, …).  Unlike those global
//! frames, local orbital frames are not static: their orientation depends on
//! the spacecraft's current position and velocity.  The type system still
//! benefits from them: functions can enforce that a displacement vector has
//! been projected into a specific local frame.
//!
//! ## Conventions
//!
//! - **RTN (RIC)**: R = along position vector (radial), N = orbit normal
//!   (r×v), T = N×R (transverse, along-track–ish). The frame is right-handed
//!   and R is exactly along position.
//! - **VNC**: V = along velocity, N = orbit normal (r×v), C = V×N (co-normal).
//! - **LVLH**: Z = radial inward (−R), X = velocity projection onto the
//!   local horizontal, Y = Z×X.
//!
//! ## Usage
//!
//! ```rust
//! use siderust_pod_core::frames::{RTN, VNC, rotate_gcrs_to_rtn};
//! use siderust_pod_core::{OrbitState, Position, Velocity};
//! use siderust::time::JulianDate;
//!
//! let s = OrbitState::new(
//!     JulianDate::new(2_451_545.0),
//!     Position::new(7000.0, 0.0, 0.0),
//!     Velocity::new(0.0, 7.5, 0.0),
//! );
//! // Position difference is already a typed Displacement<GCRS, Kilometer>.
//! let delta = s.position - s.position;   // zero displacement, just for illustration
//! let _in_rtn = rotate_gcrs_to_rtn(&s, delta);  // Displacement<RTN, Kilometer>
//! ```

use affn::cartesian::{Direction, Displacement};
use affn::DeriveReferenceFrame;
use siderust::coordinates::frames::GCRS;

use crate::state::OrbitState;
use siderust::qtty::Kilometer;

/// A 3×3 rotation matrix (row-major) from the [`affn`] crate.
///
/// Re-exported so callers do not need a direct `affn` dependency.
pub use affn::Rotation3;

// =============================================================================
// Local orbital frame marker types
// =============================================================================

/// Radial / Transverse / Normal local orbital frame.
///
/// - **R**: along the position vector (radial, geocentre → satellite)
/// - **T**: transverse, = N×R (along-track–ish; *not* the velocity direction)
/// - **N**: orbit normal, = normalize(r×v)
///
/// The frame is right-handed.  Its orientation is defined at runtime from an
/// orbit state via [`rtn_from_state`] / [`rotate_gcrs_to_rtn`].
#[derive(Debug, Copy, Clone, DeriveReferenceFrame)]
pub struct RTN;

/// Velocity / Normal / Co-normal local orbital frame.
///
/// - **V**: along the velocity vector
/// - **N**: orbit normal, = normalize(r×v)
/// - **C**: co-normal, = V×N
///
/// Orientation is defined at runtime via [`vnc_from_state`] / [`rotate_gcrs_to_vnc`].
#[derive(Debug, Copy, Clone, DeriveReferenceFrame)]
pub struct VNC;

/// Local-Vertical / Local-Horizontal frame.
///
/// - **Z**: radial inward (= −R̂)
/// - **X**: velocity projected onto the local horizontal
/// - **Y**: = Z×X
///
/// Orientation is defined at runtime via [`lvlh_from_state`].
#[derive(Debug, Copy, Clone, DeriveReferenceFrame)]
pub struct LVLH;

// =============================================================================
// Basis construction helpers
// =============================================================================

/// Velocity unit direction extracted from an orbit state.
///
/// The velocity quantity has units `Per<Km, Second>`, which is not a
/// `LengthUnit`, so `normalize()` is not available on the vector directly.
/// We extract the raw scalar components and construct a `Direction`.
fn velocity_direction(s: &OrbitState) -> Direction<GCRS> {
    Direction::new(
        s.velocity.x().value(),
        s.velocity.y().value(),
        s.velocity.z().value(),
    )
}

// =============================================================================
// Rotation-matrix builders (return Rotation3 for covariance / STM callers)
// =============================================================================

/// Build the GCRS → RTN rotation matrix from an orbit state.
///
/// Returns `R_gcrs2rtn` such that `v_rtn = R · v_gcrs`.  Rows are the RTN
/// basis vectors expressed in GCRS: `[r̂, t̂, n̂]`.
pub fn rtn_from_state(s: &OrbitState) -> Rotation3 {
    let r_hat: Direction<GCRS> = s.position.direction_unchecked();
    let v_hat: Direction<GCRS> = velocity_direction(s);
    let n_hat = r_hat.cross(&v_hat).expect("position and velocity must not be parallel");
    let t_hat = n_hat.cross(&r_hat).expect("n and r are orthogonal by construction");
    Rotation3::from_matrix_unchecked([r_hat.as_array(), t_hat.as_array(), n_hat.as_array()])
}

/// Build the GCRS → VNC rotation matrix from an orbit state.
///
/// Returns `R_gcrs2vnc` such that `v_vnc = R · v_gcrs`.  Rows are the VNC
/// basis vectors expressed in GCRS: `[v̂, n̂, ĉ]`.
pub fn vnc_from_state(s: &OrbitState) -> Rotation3 {
    let v_hat: Direction<GCRS> = velocity_direction(s);
    let r_hat: Direction<GCRS> = s.position.direction_unchecked();
    let n_hat = r_hat.cross(&v_hat).expect("position and velocity must not be parallel");
    let c_hat = v_hat.cross(&n_hat).expect("v and n are orthogonal by construction");
    Rotation3::from_matrix_unchecked([v_hat.as_array(), n_hat.as_array(), c_hat.as_array()])
}

/// Build the GCRS → LVLH rotation matrix from an orbit state.
///
/// Returns `R_gcrs2lvlh` such that `v_lvlh = R · v_gcrs`.  Rows are the LVLH
/// basis vectors expressed in GCRS: `[x̂, ŷ, ẑ]` where ẑ = −r̂.
pub fn lvlh_from_state(s: &OrbitState) -> Rotation3 {
    let r_hat: Direction<GCRS> = s.position.direction_unchecked();
    let v_hat: Direction<GCRS> = velocity_direction(s);
    let z_hat = r_hat.negate(); // radial inward
    let x_hat = v_hat
        .cross(&r_hat)
        .and_then(|n| n.cross(&z_hat))
        .expect("velocity and position must not be parallel");
    let y_hat = z_hat.cross(&x_hat).expect("z and x are orthogonal by construction");
    Rotation3::from_matrix_unchecked([x_hat.as_array(), y_hat.as_array(), z_hat.as_array()])
}

// =============================================================================
// Typed transform helpers
// =============================================================================

/// Rotate a GCRS displacement into the RTN frame of `state`.
///
/// This is the typed counterpart of [`rtn_from_state`].  The rotation matrix
/// is the same; the difference is that the result carries the [`RTN`] frame
/// tag in its type instead of requiring callers to handle raw arrays.
pub fn rotate_gcrs_to_rtn(
    state: &OrbitState,
    v: Displacement<GCRS, Kilometer>,
) -> Displacement<RTN, Kilometer> {
    (rtn_from_state(state) * v).reinterpret_frame()
}

/// Rotate a GCRS displacement into the VNC frame of `state`.
pub fn rotate_gcrs_to_vnc(
    state: &OrbitState,
    v: Displacement<GCRS, Kilometer>,
) -> Displacement<VNC, Kilometer> {
    (vnc_from_state(state) * v).reinterpret_frame()
}

/// Rotate a GCRS displacement into the LVLH frame of `state`.
pub fn rotate_gcrs_to_lvlh(
    state: &OrbitState,
    v: Displacement<GCRS, Kilometer>,
) -> Displacement<LVLH, Kilometer> {
    (lvlh_from_state(state) * v).reinterpret_frame()
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::time::JulianDate;
    use crate::state::Velocity;

    fn circular_orbit() -> OrbitState {
        OrbitState::new(
            JulianDate::new(2_451_545.0),
            crate::state::Position::new(7000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        )
    }

    #[test]
    fn rtn_basis_orthonormal_for_circular_orbit() {
        let s = circular_orbit();
        let r = rtn_from_state(&s);
        let m = r.as_matrix();
        // First row should be +x (radial along position).
        assert!((m[0][0] - 1.0).abs() < 1e-12);
        // Third row should be +z (orbit normal).
        assert!((m[2][2] - 1.0).abs() < 1e-12);
        // Second row should be +y (transverse).
        assert!((m[1][1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn vnc_basis_aligned_with_velocity_for_circular_orbit() {
        let s = circular_orbit();
        let r = vnc_from_state(&s);
        let m = r.as_matrix();
        assert!((m[0][1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn rotate_gcrs_to_rtn_typed_result() {
        let s = circular_orbit();
        // A pure radial displacement of 1 km along +x in GCRS.
        let d = Displacement::<GCRS, Kilometer>::new(1.0, 0.0, 0.0);
        let d_rtn = rotate_gcrs_to_rtn(&s, d);
        // In RTN the radial component is R; should be 1 km.
        assert!((d_rtn.x().value() - 1.0).abs() < 1e-12);
        assert!(d_rtn.y().value().abs() < 1e-12);
        assert!(d_rtn.z().value().abs() < 1e-12);
    }
}
