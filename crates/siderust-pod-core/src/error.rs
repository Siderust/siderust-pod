//! Shared error taxonomy for the Siderust POD product family.

use thiserror::Error;

/// Top-level error type re-exported by all POD crates. Each crate has its own
/// concrete error enum; they convert into this umbrella type via `#[from]`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PodError {
    /// I/O failure (file system, parsing).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A configuration value was invalid.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// A force model evaluation failed.
    #[error("dynamics error: {0}")]
    Dynamics(String),

    /// A measurement model evaluation failed.
    #[error("observation error: {0}")]
    Observation(String),

    /// An estimator failed to converge or produced an invalid result.
    #[error("estimation error: {0}")]
    Estimation(String),

    /// A QC check failed.
    #[error("QC error: {0}")]
    Qc(String),

    /// A capability was requested that has not yet been implemented.
    ///
    /// Used in particular by the runner to refuse the real-input ingestion
    /// path until milestone M9 wires SP3/RINEX/ANTEX consumers into the
    /// estimator. Carries a free-form message naming the missing
    /// capability and the milestone it is scheduled for.
    #[error("not implemented: {0}")]
    NotImplemented(String),

    /// A file format violation.
    #[error("format error in {format}: {message}")]
    Format {
        /// Format identifier (e.g. `"SP3"`, `"RINEX-OBS"`).
        format: &'static str,
        /// Human-readable description of the problem.
        message: String,
    },

    /// A provider (ephemeris, EOP, frame transform) failed.
    #[error("provider error in {provider}: {message}")]
    Provider {
        /// Name of the provider (e.g. `"EphemerisProvider"`).
        provider: &'static str,
        /// Human-readable description of the failure.
        message: String,
    },
}

/// Convenience result alias for crate APIs.
pub type Result<T> = core::result::Result<T, PodError>;
