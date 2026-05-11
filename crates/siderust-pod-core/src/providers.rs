//! Provider traits bridging POD code to public `siderust` services.
//!
//! These traits are deliberately minimal so that POD code never reaches
//! into private `siderust` modules. Concrete implementations are typically
//! thin wrappers around the public APIs of `siderust::astro::*`.
//!
//! Concrete adapters live in `siderust-pod-dynamics` (force-model side)
//! and `siderust-pod-service` (run-context side).

use std::error::Error;

/// Marker trait for an ephemeris provider. Concrete state types are
/// resolved at the call site to avoid pinning POD code to a single
/// `siderust` representation. Implementations should typically wrap
/// `siderust::calculus::ephemeris` backends.
pub trait EphemerisProvider {
    type State;
    type Error: Error + Send + Sync + 'static;

    /// Return a state for `body_naif_id` at the given `epoch_seconds_tdb`.
    /// The semantics of `epoch_seconds_tdb` are TDB seconds since J2000.
    fn state(
        &self,
        body_naif_id: i32,
        epoch_seconds_tdb: f64,
    ) -> Result<Self::State, Self::Error>;
}

/// Marker trait for an Earth-orientation / EOP provider. Implementations
/// typically wrap `siderust::astro::eop` or `tempoch::eop`.
pub trait EarthOrientationProvider {
    type Error: Error + Send + Sync + 'static;

    /// UT1 - UTC offset (seconds) at the given UTC epoch (seconds since J2000 UTC).
    fn ut1_minus_utc(&self, epoch_seconds_utc: f64) -> Result<f64, Self::Error>;
    /// Polar motion x component (radians) at the given UTC epoch.
    fn polar_motion_x(&self, epoch_seconds_utc: f64) -> Result<f64, Self::Error>;
    /// Polar motion y component (radians) at the given UTC epoch.
    fn polar_motion_y(&self, epoch_seconds_utc: f64) -> Result<f64, Self::Error>;
}

/// Marker trait for a frame-transform service: produce the rotation that
/// takes a vector expressed in `from` to the same vector expressed in
/// `to` at `epoch`. Concrete frame ids are stringly-typed at this layer
/// because POD code must not leak `affn` frame markers across crate
/// boundaries; concrete adapters resolve them statically.
pub trait FrameTransformProvider {
    type Rotation;
    type Error: Error + Send + Sync + 'static;

    fn rotation(
        &self,
        from: &str,
        to: &str,
        epoch_seconds_tdb: f64,
    ) -> Result<Self::Rotation, Self::Error>;
}
