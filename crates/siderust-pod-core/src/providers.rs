//! Provider traits for the Siderust POD product family.
//!
//! These traits decouple POD compute crates from concrete `siderust` types.
//! Each provider has a default implementation backed by public `siderust` items.
//! Downstream code may inject custom implementations for testing, deterministic
//! reproduction, or alternative backends.
//!
//! See `docs/architecture/providers.md` and `docs/adrs/ADR-0004-provider-traits.md`.

pub mod atmosphere_density;
pub mod earth_orientation;
pub mod ephemeris;
pub mod frame_transform;
pub mod gravity_field;

pub use atmosphere_density::{ConstantDensity, DensityProvider, ExponentialAtmosphere};
pub use earth_orientation::{EopError, EopProvider, EopValues};
pub use ephemeris::{BoxEphemeris, DynEphemeris, Vsop87Provider};
pub use frame_transform::FrameTransformProvider;
pub use gravity_field::{GravityConstants, GravityFieldProvider, TwoBodyEarth};
