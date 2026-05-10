//! Ephemeris provider trait.
//!
//! Adapter over [`siderust::calculus::ephemeris::DynEphemeris`] expressing the
//! capabilities POD actually needs. The default implementation [`Vsop87Provider`]
//! wraps the built-in VSOP87/ELP2000 backend.

use siderust::calculus::ephemeris::{AuPerDay, DynEphemeris, Vsop87Ephemeris};
use siderust::coordinates::cartesian::{Position, Velocity};
use siderust::coordinates::centers::{Barycentric, Geocentric, Heliocentric};
use siderust::coordinates::frames::EclipticMeanJ2000;
use siderust::qtty::{AstronomicalUnit, Kilometer};
use siderust::time::JulianDate;

use crate::error::{PodError, Result};

/// Trait providing solar-system body positions and velocities at a given epoch.
///
/// Methods return typed positions in the J2000 ecliptic frame. Downstream code
/// is responsible for transforming to the inertial frame used in dynamics
/// (typically GCRF) via the [`crate::providers::FrameTransformProvider`].
pub trait EphemerisProvider: Send + Sync {
    /// Sun position in barycentric ecliptic coordinates.
    fn sun_barycentric(
        &self,
        jd: JulianDate,
    ) -> Result<Position<Barycentric, EclipticMeanJ2000, AstronomicalUnit>>;

    /// Earth position in barycentric ecliptic coordinates.
    fn earth_barycentric(
        &self,
        jd: JulianDate,
    ) -> Result<Position<Barycentric, EclipticMeanJ2000, AstronomicalUnit>>;

    /// Earth position in heliocentric ecliptic coordinates.
    fn earth_heliocentric(
        &self,
        jd: JulianDate,
    ) -> Result<Position<Heliocentric, EclipticMeanJ2000, AstronomicalUnit>>;

    /// Earth velocity in barycentric ecliptic coordinates.
    fn earth_barycentric_velocity(
        &self,
        jd: JulianDate,
    ) -> Result<Velocity<EclipticMeanJ2000, AuPerDay>>;

    /// Moon position in geocentric ecliptic coordinates.
    fn moon_geocentric(
        &self,
        jd: JulianDate,
    ) -> Result<Position<Geocentric, EclipticMeanJ2000, Kilometer>>;
}

/// Default [`EphemerisProvider`] backed by the VSOP87/ELP2000 backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct Vsop87Provider;

impl EphemerisProvider for Vsop87Provider {
    fn sun_barycentric(
        &self,
        jd: JulianDate,
    ) -> Result<Position<Barycentric, EclipticMeanJ2000, AstronomicalUnit>> {
        Vsop87Ephemeris.try_sun_barycentric(jd).map_err(map_eph)
    }
    fn earth_barycentric(
        &self,
        jd: JulianDate,
    ) -> Result<Position<Barycentric, EclipticMeanJ2000, AstronomicalUnit>> {
        Vsop87Ephemeris.try_earth_barycentric(jd).map_err(map_eph)
    }
    fn earth_heliocentric(
        &self,
        jd: JulianDate,
    ) -> Result<Position<Heliocentric, EclipticMeanJ2000, AstronomicalUnit>> {
        Vsop87Ephemeris.try_earth_heliocentric(jd).map_err(map_eph)
    }
    fn earth_barycentric_velocity(
        &self,
        jd: JulianDate,
    ) -> Result<Velocity<EclipticMeanJ2000, AuPerDay>> {
        Vsop87Ephemeris
            .try_earth_barycentric_velocity(jd)
            .map_err(map_eph)
    }
    fn moon_geocentric(
        &self,
        jd: JulianDate,
    ) -> Result<Position<Geocentric, EclipticMeanJ2000, Kilometer>> {
        Vsop87Ephemeris.try_moon_geocentric(jd).map_err(map_eph)
    }
}

fn map_eph<E: std::fmt::Display>(e: E) -> PodError {
    PodError::Provider {
        provider: "EphemerisProvider",
        message: e.to_string(),
    }
}
