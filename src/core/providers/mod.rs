//! Provider traits bridging POD code to external services.
//!
//! These traits define the minimal interfaces that POD force models,
//! observation models, and the estimator use to query Earth orientation,
//! ephemerides, and frame transforms. Concrete implementations are supplied
//! by the caller (SPICE, EOP files, siderust services, etc.).
//!
//! Gravity and atmosphere traits live directly in
//! [`siderust::astro::dynamics::gravity`] and
//! [`siderust::astro::dynamics::atmosphere`]; import them from there.

pub mod eop;
pub mod ephemeris;
pub mod frame_transform;

pub use eop::EarthOrientationProvider;
pub use ephemeris::EphemerisProvider;
pub use frame_transform::FrameTransformProvider;
