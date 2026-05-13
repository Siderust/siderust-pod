//! Provider traits bridging POD code to public `siderust` services.
//!
//! Per design §6.1 / §6.2 the POD layer interacts with five service
//! categories at the boundary of the [`siderust`] kernel:
//!
//! 1. [`EphemerisProvider`] — third-body / heliocentric ephemerides.
//! 2. [`EarthOrientationProvider`] — UT1-UTC, polar motion.
//! 3. [`FrameTransformProvider`] — inertial ↔ Earth-fixed rotations.
//! 4. [`GravityFieldProvider`] — fully-normalised geopotential
//!    coefficients (re-exported from [`siderust`] as the canonical
//!    interface).
//! 5. [`AtmosphereDensityProvider`] — neutral-atmosphere density for
//!    drag (re-exported from [`siderust`]'s `DensityProvider`).
//!
//! These traits are deliberately minimal so that POD code never reaches
//! into private `siderust` modules. Concrete implementations are typically
//! thin wrappers around the public APIs of `siderust::astro::*`. The
//! gravity- and atmosphere-side traits are *direct re-exports* from
//! `siderust` because the canonical typed contracts already live there;
//! mirroring them inside POD would create two parallel public surfaces
//! that could drift apart.
//!
//! The remaining three traits ([`EphemerisProvider`],
//! [`EarthOrientationProvider`], [`FrameTransformProvider`]) currently
//! accept epochs as J2000-anchored `f64` seconds for compatibility with
//! the in-flight `siderust-spice` adapter rewrite; the typed-`Time<S>`
//! migration is tracked under that crate's productization phase.
//!
//! # Default adapters
//!
//! Lightweight default adapters that wrap public `siderust` services are
//! provided in [`adapters`] for prototyping. Production POD pipelines
//! typically build a richer context-aware adapter inside
//! `siderust-pod-service`.

pub mod adapters;
pub mod atmosphere;
pub mod eop;
pub mod ephemeris;
pub mod frame_transform;
pub mod gravity;

pub use atmosphere::AtmosphereDensityProvider;
pub use eop::EarthOrientationProvider;
pub use ephemeris::EphemerisProvider;
pub use frame_transform::FrameTransformProvider;
pub use gravity::GravityFieldProvider;
