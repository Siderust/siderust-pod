//! # CPF orbit prediction reader
//!
//! ## Scientific scope
//!
//! The Consolidated Prediction Format is used in satellite laser ranging to
//! distribute tabulated orbit predictions. This module reads the subset
//! needed for POD-side SLR validation, namely the reference-frame metadata
//! and sampled position records used to reconstruct predicted trajectories.
//!
//! Velocity records and richer optional sections are currently out of
//! scope, so the parser is suitable for position-driven validation
//! workflows rather than full CPF authoring or exhaustive standard
//! coverage.
//!
//! ## Technical scope
//!
//! The public entry points are `read_cpf` and `parse_cpf`, which produce
//! `CpfFile` values containing header metadata and `CpfPosition` samples.
//! Samples are kept in the file-native scalar form expected by downstream
//! SLR utilities.
//!
//! The module does not propagate, interpolate, or compare predictions on
//! its own; those steps belong to service and QC code.
//!
//! ## References
//!
//! - International Laser Ranging Service. (2018). Consolidated Prediction
//!   Format (CPF) Specification.
//! - Pearlman, M. R., Degnan, J. J., & Bosworth, J. M. (2002). The
//!   International Laser Ranging Service. Advances in Space Research,
//!   30(2), 135-143.
use crate::PodIoError;
use std::fs;
use std::path::Path;

/// One CPF position sample (record type "10").
#[derive(Debug, Clone)]
pub struct CpfPosition {
    /// Modified Julian Day of sample.
    pub mjd: i32,
    /// Seconds of day (UTC).
    pub seconds_of_day: f64,
    /// Position in metres, ITRF (per CPF convention).
    pub r_m: [f64; 3],
}

/// Parsed CPF file.
#[derive(Debug, Clone, Default)]
pub struct CpfFile {
    /// Source agency (H1).
    pub source: String,
    /// CPF version (H1).
    pub version: String,
    /// Target name (H2).
    pub target_name: String,
    /// Reference frame string from H2 (e.g. "ITRF2014").
    pub reference_frame: String,
    /// Position records.
    pub positions: Vec<CpfPosition>,
}

/// Read a CPF file from disk.
pub fn read_cpf<P: AsRef<Path>>(path: P) -> Result<CpfFile, PodIoError> {
    let text = fs::read_to_string(path)?;
    parse_cpf(&text)
}

/// Parse a CPF file from a string slice.
pub fn parse_cpf(text: &str) -> Result<CpfFile, PodIoError> {
    let mut out = CpfFile::default();
    for raw in text.lines() {
        let line = raw.trim_end();
        if line.is_empty() {
            continue;
        }
        let mut tokens = line.split_whitespace();
        let tag = match tokens.next() {
            Some(t) => t,
            None => continue,
        };
        match tag {
            "H1" => {
                // H1 CPF version source year month day hour ephemeris_seq
                let _ = tokens.next();
                if let Some(s) = tokens.next() {
                    out.version = s.to_string();
                }
                if let Some(s) = tokens.next() {
                    out.source = s.to_string();
                }
            }
            "H2" => {
                // H2 cospar sic norad start_year ... reference_frame
                let mut last = String::new();
                let mut name_or_norad = String::new();
                if let Some(t) = tokens.next() {
                    name_or_norad = t.to_string();
                }
                for t in tokens {
                    last = t.to_string();
                }
                out.target_name = name_or_norad;
                out.reference_frame = last;
            }
            "10" => {
                // 10 dir mjd sod leap x y z
                let _dir = tokens.next();
                let mjd: i32 = tokens
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| PodIoError::Format("CPF 10: missing MJD".into()))?;
                let sod: f64 = tokens
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| PodIoError::Format("CPF 10: missing SOD".into()))?;
                let _leap = tokens.next();
                let x: f64 = tokens
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| PodIoError::Format("CPF 10: missing X".into()))?;
                let y: f64 = tokens
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| PodIoError::Format("CPF 10: missing Y".into()))?;
                let z: f64 = tokens
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| PodIoError::Format("CPF 10: missing Z".into()))?;
                out.positions.push(CpfPosition {
                    mjd,
                    seconds_of_day: sod,
                    r_m: [x, y, z],
                });
            }
            _ => {}
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_cpf() {
        let txt = "\
H1 CPF 2 HTS 2024 01 01 00 1\n\
H2 1234567 1155 7603901 2024 01 01 00 00 00 0 0 ITRF2014\n\
10 1 60310 0.000     0  7000000.0      0.0      0.0\n\
10 1 60310 60.000    0  7000100.0   1000.0    -50.0\n\
99\n";
        let f = parse_cpf(txt).expect("parse");
        assert_eq!(f.version, "2");
        assert_eq!(f.reference_frame, "ITRF2014");
        assert_eq!(f.positions.len(), 2);
        assert_eq!(f.positions[1].mjd, 60310);
        assert!((f.positions[1].r_m[0] - 7_000_100.0).abs() < 1e-9);
    }
}
