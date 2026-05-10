//! Earth orientation provider trait.
//!
//! Wraps `tempoch::eop` data and exposes UT1-UTC, polar motion and length-of-day
//! at a given UTC epoch. The default implementation defers to the EOP dataset
//! configured at runtime via `tempoch`.

use crate::error::Result;
use qtty::time::Seconds;

/// Earth-orientation parameters at an instant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eop {
    /// UT1 - UTC, in seconds.
    pub ut1_minus_utc: Seconds,
    /// Polar motion x component, in radians.
    pub xp_rad: f64,
    /// Polar motion y component, in radians.
    pub yp_rad: f64,
    /// Length-of-day excess, in seconds.
    pub lod: Seconds,
}

/// Provider of Earth-orientation parameters for an arbitrary UTC epoch.
pub trait EarthOrientationProvider: Send + Sync {
    /// Return the EOP at the given Modified Julian Date (UTC).
    fn at_mjd_utc(&self, mjd_utc: f64) -> Result<Eop>;
}
