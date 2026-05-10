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
use siderust::coordinates::centers::Geocentric;
use siderust::coordinates::cartesian;
use siderust::coordinates::frames::GCRS;
use siderust::qtty::{Kilograms, SquareMeters};
use siderust::qtty::unit::{Kilometer, Per, Second};
use siderust::time::JulianDate;

/// Geocentric inertial position in GCRS, km.
pub type Position<S, U = Kilometer> = cartesian::Position<Geocentric, S, U>;

/// Velocity vector in GCRS frame, km/s.
pub type Velocity<S, U = Per<Kilometer, Second>> = cartesian::Velocity<S, U>;

/// Cartesian inertial position + velocity in km / (km/s) in GCRS.
///
/// Position and velocity are stored as typed [`siderust`] coordinate values,
/// giving compile-time frame and unit guarantees.
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
        self.epoch_tt == other.epoch_tt
            && self.position.x() == other.position.x()
            && self.position.y() == other.position.y()
            && self.position.z() == other.position.z()
            && self.velocity.x() == other.velocity.x()
            && self.velocity.y() == other.velocity.y()
            && self.velocity.z() == other.velocity.z()
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

    /// Advance position and velocity by `dt_s` seconds along `deriv`.
    ///
    /// The epoch is **not** updated — the caller is responsible for advancing
    /// `epoch_tt` to the new time. This mirrors the mathematical step
    /// `x(t + h) ≈ x(t) + h · ẋ(t)`.
    #[inline]
    pub fn advance(&self, deriv: &StateDerivative, dt_s: f64) -> Self {
        Self {
            epoch_tt: self.epoch_tt,
            position: Position::<GCRS>::new(
                self.position.x().value() + dt_s * deriv.vel[0],
                self.position.y().value() + dt_s * deriv.vel[1],
                self.position.z().value() + dt_s * deriv.vel[2],
            ),
            velocity: Velocity::<GCRS>::new(
                self.velocity.x().value() + dt_s * deriv.acc[0],
                self.velocity.y().value() + dt_s * deriv.acc[1],
                self.velocity.z().value() + dt_s * deriv.acc[2],
            ),
        }
    }
}

/// Time derivative of an [`OrbitState`]: the 6-vector `[dr/dt, dv/dt]`.
///
/// `vel` is the position rate (= velocity, km/s) and `acc` is the velocity
/// rate (= inertial acceleration in km/s²). Keeping these as raw `[f64; 3]`
/// arrays allows force-model arithmetic without hitting the `affn` version
/// boundary; a fully-typed version is deferred until the version split is
/// resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateDerivative {
    /// Position rate (= velocity), km/s.
    pub vel: [f64; 3],
    /// Velocity rate (= acceleration), km/s².
    pub acc: [f64; 3],
}

impl StateDerivative {
    /// Weighted combination: `self + (w2·d2 + w3·d3 + w4·d4) / norm` used by RK4.
    #[inline]
    pub fn rk4_combine(
        k1: &Self,
        k2: &Self,
        k3: &Self,
        k4: &Self,
    ) -> Self {
        Self {
            vel: [
                (k1.vel[0] + 2.0 * k2.vel[0] + 2.0 * k3.vel[0] + k4.vel[0]) / 6.0,
                (k1.vel[1] + 2.0 * k2.vel[1] + 2.0 * k3.vel[1] + k4.vel[1]) / 6.0,
                (k1.vel[2] + 2.0 * k2.vel[2] + 2.0 * k3.vel[2] + k4.vel[2]) / 6.0,
            ],
            acc: [
                (k1.acc[0] + 2.0 * k2.acc[0] + 2.0 * k3.acc[0] + k4.acc[0]) / 6.0,
                (k1.acc[1] + 2.0 * k2.acc[1] + 2.0 * k3.acc[1] + k4.acc[1]) / 6.0,
                (k1.acc[2] + 2.0 * k2.acc[2] + 2.0 * k3.acc[2] + k4.acc[2]) / 6.0,
            ],
        }
    }

    /// Scale this derivative by `factor`.
    #[inline]
    pub fn scaled(&self, factor: f64) -> Self {
        Self {
            vel: [self.vel[0] * factor, self.vel[1] * factor, self.vel[2] * factor],
            acc: [self.acc[0] * factor, self.acc[1] * factor, self.acc[2] * factor],
        }
    }

    /// Element-wise addition.
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            vel: [self.vel[0] + other.vel[0], self.vel[1] + other.vel[1], self.vel[2] + other.vel[2]],
            acc: [self.acc[0] + other.acc[0], self.acc[1] + other.acc[1], self.acc[2] + other.acc[2]],
        }
    }
}

/// Spacecraft properties carried alongside the orbit state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacecraftProperties {
    /// Total mass.
    pub mass: Kilograms,
    /// Cross-section for drag.
    pub drag_area: SquareMeters,
    /// Drag coefficient (dimensionless).
    pub cd: f64,
    /// Cross-section for SRP.
    pub srp_area: SquareMeters,
    /// SRP coefficient (dimensionless).
    pub cr: f64,
}

impl SpacecraftProperties {
    /// Reasonable demo defaults for a small LEO platform.
    pub fn demo_leo() -> Self {
        Self {
            mass: Kilograms::new(500.0),
            drag_area: SquareMeters::new(2.0),
            cd: 2.2,
            srp_area: SquareMeters::new(2.0),
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
    }

    #[test]
    fn advance_applies_derivative_correctly() {
        let epoch = JulianDate::new(2_451_545.0);
        let pos = Position::<GCRS>::new(7000.0, 0.0, 0.0);
        let vel = Velocity::<GCRS>::new(0.0, 7.5, 0.0);
        let s = OrbitState::new(epoch, pos, vel);

        let deriv = StateDerivative {
            vel: [0.0, 7.5, 0.0],
            acc: [0.0, 0.0, -9.8e-3],
        };
        let dt = 10.0;
        let s2 = s.advance(&deriv, dt);

        assert!((s2.position.x().value() - 7000.0).abs() < 1e-10);
        assert!((s2.position.y().value() - 75.0).abs() < 1e-10);
        assert!((s2.velocity.z().value() - (-0.098)).abs() < 1e-10);
        // Epoch is unchanged by advance.
        assert_eq!(s2.epoch_tt, epoch);
    }

    #[test]
    fn spacecraft_properties_demo_leo() {
        let p = SpacecraftProperties::demo_leo();
        assert!((p.mass.value() - 500.0).abs() < f64::EPSILON);
        assert!((p.drag_area.value() - 2.0).abs() < f64::EPSILON);
        assert!((p.srp_area.value() - 2.0).abs() < f64::EPSILON);
    }
}
