//! POD error taxonomy.
//!
//! `PodError` is the top-level error returned by service-level POD
//! operations. Library crates (`-io`, `-observations`, `-estimation`, ...)
//! define their own `thiserror` enums and convert into `PodError` at the
//! workspace boundary.

use thiserror::Error;

/// Top-level POD error.
#[derive(Debug, Error)]
pub enum PodError {
    /// Capability scoped to a future milestone is not yet implemented.
    #[error("not implemented: {0}")]
    NotImplemented(String),

    /// Invalid configuration or input outside the supported envelope.
    #[error("invalid input: {0}")]
    Invalid(String),

    /// Underlying I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// JSON serialisation/deserialisation error (only when `serde` feature is on).
    #[cfg(feature = "serde")]
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
