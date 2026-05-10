//! `siderust-pod-io` — parsers and writers for POD/geodesy file formats.
//!
//! M2 status: SP3 (subset), CCSDS OEM writer, RINEX 3 OBS subset reader,
//! ANTEX (PCO subset) reader, IERS C04 EOP reader.
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
