//! # siderust-pod I/O
//!
//! ## Scientific scope
//!
//! This crate handles the interchange formats that feed and leave POD
//! workflows: precise or broadcast GNSS products, SLR products, Earth-
//! orientation series, and orbit product files. The scientific content is
//! defined by the external standards being parsed or written rather than by
//! new physics inside the crate.
//!
//! Current coverage is intentionally selective. Each parser or writer
//! supports the subset needed by the MVP POD pipeline and ignores
//! unsupported record families when that is safer than pretending to
//! implement the full standard.
//!
//! ## Technical scope
//!
//! The crate exports format-specific reader and writer modules plus the
//! shared `PodIoError`. Most readers map external text or binary records
//! into typed Rust structs and, where appropriate, into `siderust`
//! orbit/time types such as `OrbitState` and `JulianDate`.
//!
//! Higher-level estimation, validation, and product orchestration are
//! outside the scope of this crate and are handled by sibling service,
//! observations, and products modules.
//!
//! ## References
//!
//! - IERS Conventions Centre. (2010). IERS Conventions (2010). Verlag des
//!   Bundesamts fur Kartographie und Geodasie.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

pub mod antex;
pub mod cpf;
pub mod crd;
pub mod eop;
pub mod oem;
pub mod rinex_nav;
pub mod rinex_obs;
pub mod sp3;

/// IO-layer error type.
#[derive(Debug, Error)]
pub enum PodIoError {
    /// Underlying IO failure.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// Malformed or unsupported file content.
    #[error("format: {0}")]
    Format(String),
}
