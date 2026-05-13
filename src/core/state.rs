//! POD orbit-state and spacecraft-state re-exports.
//!
//! Per the design (`docs/design/siderust_pod_detailed_design_document.md`,
//! §6.1), `OrbitState` is the canonical typed state representation used
//! throughout the POD pipeline. Rather than duplicate it here, this module
//! re-exports the implementations that already live in
//! [`siderust::astro::dynamics::state`] so that POD code and `siderust`
//! consumers share the *same* `affn`-backed types — eliminating the class
//! of bugs that arises from having two structurally-identical state types
//! in the public surface.
//!
//! Any future POD-specific extension (extra metadata, propagation hints,
//! etc.) should be a wrapper around [`OrbitState`], not a redefinition.
//!
//! # Examples
//!
//! ```
//! use siderust_pod::core::state::{OrbitState, Position, Velocity};
//! use siderust::coordinates::frames::GCRS;
//! use siderust::time::JulianDate;
//!
//! let s = OrbitState::new_at_jd(
//!     JulianDate::new(2_451_545.0),
//!     Position::<GCRS>::new(7000.0, 0.0, 0.0),
//!     Velocity::<GCRS>::new(0.0, 7.5, 0.0),
//! );
//! assert!((s.position.x().value() - 7000.0).abs() < 1e-12);
//! ```

pub use siderust::astro::dynamics::state::{
    Acceleration, AccelerationUnit, Force, OrbitState, Position, StateDerivative, Velocity,
    VelocityUnit,
};
