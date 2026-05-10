//! Spacecraft and orbit state primitives.
//!
//! Re-exported from [`siderust::astro::dynamics::state`]. The canonical
//! definitions live in `siderust`; this module exists only as a
//! compatibility shim so existing POD callers keep working under
//! `siderust_pod_core::state`.

pub use siderust::astro::dynamics::state::{
    Acceleration, AccelerationUnit, OrbitState, Position, SpacecraftProperties, SpacecraftState,
    StateDerivative, Velocity, VelocityUnit,
};
