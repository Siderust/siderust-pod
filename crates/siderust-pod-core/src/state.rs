//! Spacecraft and orbit state primitives.
//!
//! These types carry only the *minimum* information force models and
//! integrators need; richer state (including spacecraft mass, parameter
//! blocks, and provenance) lives in [`SpacecraftState`].
//!
//! ## Typed fields
//!
//! Position and velocity are stored directly as [`siderust`] typed values so
//! that frame and unit constraints are enforced at compile time:
//!
//! ```rust
//! use siderust_pod_core::OrbitState;
//! use siderust_pod_core::state::{Position, Velocity};
//! use siderust::time::JulianDate;
//! use siderust::coordinates::frames::GCRS;
//!
//! let pos = Position::<GCRS>::new(7000.0, 0.0, 0.0);
//! let vel = Velocity::<GCRS>::new(0.0, 7.5, 0.0);
//! let s = OrbitState::new(JulianDate::new(2_451_545.0), pos, vel);
//! assert!((s.position.x().value() - 7000.0).abs() < 1e-12);
//! ```
//!
//! The raw-array helpers [`OrbitState::to_array6`] and
//! [`OrbitState::from_array6`] are retained for the integrator inner loops
//! where arithmetic operates on plain `[f64; 6]` slices.
use siderust::coordinates::centers::Geocentric;
use siderust::coordinates::cartesian;
use siderust::coordinates::frames::GCRS;
use siderust::qtty::unit::{Kilometer, Per, Second};
use siderust::time::JulianDate;

/// Geocentric inertial position in GCRS, km.
pub type Position<S, U = Kilometer> = cartesian::Position<Geocentric, S, U>;

/// Velocity vector in GCRS frame, km/s.
pub type Velocity<S, U = Per<Kilometer, Second>> = cartesian::Velocity<S, U>;

/// Cartesian inertial position + velocity in km / (km/s) in GCRS.
///
/// Position and velocity are stored as typed [`siderust`] coordinate values,
/// giving compile-time frame and unit guarantees. The raw-array helpers
/// [`to_array6`](OrbitState::to_array6) / [`from_array6`](OrbitState::from_array6)
/// are provided for numeric inner loops (integrators, STM).
#[derive(Debug, Clone, Copy)]
pub struct OrbitState {
    /// Epoch (TT scale, Julian Date).
    pub epoch_tt: JulianDate,
    /// Position in GCRS, km.
    pub position: Position<GCRS, Kilometer>,
    /// Velocity in GCRS, km/s.
    pub velocity: Velocity<GCRS, Per<Kilometer, Second>>,
}

impl PartialEq for OrbitState {
    fn eq(&self, other: &Self) -> bool {
        self.epoch_tt == other.epoch_tt && self.to_array6() == other.to_array6()
    }
}

impl OrbitState {
    /// Construct from typed GCRS position and velocity.
    ///
    /// Call sites that only have raw `f64` components should wrap them first:
    /// ```
    /// use siderust_pod_core::state::{Position, Velocity};
    /// use siderust::coordinates::frames::GCRS;
    /// let pos = Position::<GCRS>::new(7000.0, 0.0, 0.0);
    /// let vel = Velocity::<GCRS>::new(0.0, 7.5, 0.0);
    /// ```
    #[inline]
    pub fn new(epoch_tt: JulianDate, position: Position<GCRS>, velocity: Velocity<GCRS>) -> Self {
        Self { epoch_tt, position, velocity }
    }

    /// 6-vector `[r, v]` packing used by integrators.
    #[inline]
    pub fn to_array6(&self) -> [f64; 6] {
        [
            self.position.x().value(),
            self.position.y().value(),
            self.position.z().value(),
            self.velocity.x().value(),
            self.velocity.y().value(),
            self.velocity.z().value(),
        ]
    }

    /// Construct from a 6-vector `[r, v]` at a given epoch.
    #[inline]
    pub fn from_array6(epoch_tt: JulianDate, x: [f64; 6]) -> Self {
        Self {
            epoch_tt,
            position: Position::<GCRS>::new(x[0], x[1], x[2]),
            velocity: Velocity::<GCRS>::new(x[3], x[4], x[5]),
        }
    }

    /// Position magnitude squared (km²).
    #[inline]
    pub fn r2(&self) -> f64 {
        let d = self.position.distance();
        d.value() * d.value()
    }
}

/// Spacecraft properties carried alongside the orbit state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacecraftProperties {
    /// Total mass, kg.
    pub mass_kg: f64,
    /// Cross-section for drag, m².
    pub drag_area_m2: f64,
    /// Drag coefficient (dimensionless).
    pub cd: f64,
    /// Cross-section for SRP, m².
    pub srp_area_m2: f64,
    /// SRP coefficient (dimensionless).
    pub cr: f64,
}

impl SpacecraftProperties {
    /// Reasonable demo defaults for a small LEO platform.
    pub fn demo_leo() -> Self {
        Self {
            mass_kg: 500.0,
            drag_area_m2: 2.0,
            cd: 2.2,
            srp_area_m2: 2.0,
            cr: 1.3,
        }
    }
}

/// Combined spacecraft state used by the propagator and the estimator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacecraftState {
    /// Orbit state.
    pub orbit: OrbitState,
    /// Spacecraft properties.
    pub properties: SpacecraftProperties,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_roundtrip_preserves_values() {
        let epoch = JulianDate::new(2_451_545.0);
        let pos = Position::<GCRS>::new(7000.0, 100.0, -200.0);
        let vel = Velocity::<GCRS>::new(0.5, 7.4, -0.1);

        let s = OrbitState::new(epoch, pos, vel);

        // Typed position fields must match input values exactly.
        assert!((s.position.x().value() - 7000.0).abs() < f64::EPSILON);
        assert!((s.position.y().value() - 100.0).abs() < f64::EPSILON);
        assert!((s.position.z().value() - (-200.0)).abs() < f64::EPSILON);
        assert!((s.velocity.x().value() - 0.5).abs() < f64::EPSILON);
        assert!((s.velocity.y().value() - 7.4).abs() < f64::EPSILON);
        assert!((s.velocity.z().value() - (-0.1)).abs() < f64::EPSILON);

        // to_array6 must reproduce the same values.
        let arr = s.to_array6();
        assert!((arr[0] - 7000.0).abs() < f64::EPSILON);
        assert!((arr[1] - 100.0).abs() < f64::EPSILON);
        assert!((arr[2] - (-200.0)).abs() < f64::EPSILON);
        assert!((arr[3] - 0.5).abs() < f64::EPSILON);
        assert!((arr[4] - 7.4).abs() < f64::EPSILON);
        assert!((arr[5] - (-0.1)).abs() < f64::EPSILON);
    }

    #[test]
    fn from_array6_consistent_with_new() {
        let epoch = JulianDate::new(2_451_545.0);
        let r = [6378.0, 0.0, 1000.0];
        let v = [0.0, 7.8, 0.0];

        let from_new = OrbitState::new(
            epoch,
            Position::<GCRS>::new(r[0], r[1], r[2]),
            Velocity::<GCRS>::new(v[0], v[1], v[2]),
        );
        let from_arr = OrbitState::from_array6(epoch, [r[0], r[1], r[2], v[0], v[1], v[2]]);

        assert_eq!(from_new, from_arr);
    }
}
