//! # RINEX-DORIS reader (feature-gated stub)
//!
//! Full RINEX-DORIS parsing is behind the `doris` feature flag.
//! Without the feature, a public type and function are exported so callers
//! can pattern-match on the error without a panic.
//!
//! ## References
//!
//! - IGS/IDS RINEX-DORIS Format Description.

use super::PodIoError;
use std::io::Read;

/// A parsed RINEX-DORIS record (placeholder).
///
/// # Examples
///
/// ```
/// use siderust_pod::io::rinex_doris::RinexDorisRecord;
/// let r = RinexDorisRecord::default();
/// assert_eq!(r.epoch_mjd, 0.0);
/// ```
#[derive(Debug, Default)]
pub struct RinexDorisRecord {
    /// Header epoch (MJD).
    pub epoch_mjd: f64,
}

/// Read a RINEX-DORIS file.
///
/// Returns [`PodIoError::Unsupported`] unless the `doris` feature is enabled.
/// Even with the feature, this implementation returns `Unsupported` until a
/// full parser is provided.
///
/// # Errors
///
/// Always returns [`PodIoError::Unsupported`] in the current implementation.
///
/// # Examples
///
/// ```
/// use siderust_pod::io::rinex_doris::read_rinex_doris;
/// use siderust_pod::io::PodIoError;
/// let err = read_rinex_doris(&b""[..]).unwrap_err();
/// assert!(matches!(err, PodIoError::Unsupported(_)));
/// ```
#[cfg(not(feature = "doris"))]
pub fn read_rinex_doris<R: Read>(_reader: R) -> Result<RinexDorisRecord, PodIoError> {
    Err(PodIoError::Unsupported(
        "RINEX-DORIS parsing requires the `doris` feature".to_string(),
    ))
}

/// Read a RINEX-DORIS file (feature-enabled stub).
///
/// Returns [`PodIoError::Unsupported`] until a full parser is provided.
///
/// # Errors
///
/// Always returns [`PodIoError::Unsupported`] in the current implementation.
///
/// # Examples
///
/// ```
/// use siderust_pod::io::rinex_doris::read_rinex_doris;
/// use siderust_pod::io::PodIoError;
/// let err = read_rinex_doris(&b""[..]).unwrap_err();
/// assert!(matches!(err, PodIoError::Unsupported(_)));
/// ```
#[cfg(feature = "doris")]
pub fn read_rinex_doris<R: Read>(_reader: R) -> Result<RinexDorisRecord, PodIoError> {
    Err(PodIoError::Unsupported(
        "RINEX-DORIS not yet implemented even with `doris` feature".to_string(),
    ))
}
