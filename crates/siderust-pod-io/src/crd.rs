//! # CRD SLR observation reader
//!
//! ## Scientific scope
//!
//! The Consolidated Laser Ranging Data format is the primary interchange
//! format for SLR observations. This module reads the station, satellite,
//! and session metadata together with full-rate and normal-point ranges
//! needed to test the POD SLR modelling path.
//!
//! It intentionally stays within a validation-oriented regime. Unsupported
//! CRD record families are skipped so the parser remains useful on real
//! files without claiming complete implementation of the ILRS standard.
//!
//! ## Technical scope
//!
//! The public APIs are `read_crd` and `parse_crd`, which return a `CrdFile`
//! containing metadata and `CrdRange` observations. The values remain close
//! to the on-disk representation so later stages can choose their own
//! modelling assumptions.
//!
//! Station motion, atmospheric delay, and any range residual analysis are
//! outside this module.
//!
//! ## References
//!
//! - International Laser Ranging Service. (2022). Consolidated Laser
//!   Ranging Data Format Specification.
//! - Pearlman, M. R., Noll, C. E., et al. (2019). The ILRS: Current status
//!   and future prospects. Journal of Geodesy, 93, 2161-2180.
use crate::PodIoError;
use qtty::time::Seconds;
use std::fs;
use std::path::Path;

/// One SLR range observation in CRD ("11" normal point) or ("10" full rate).
#[derive(Debug, Clone)]
pub struct CrdRange {
    /// Seconds of day (UTC) at the epoch of the range observation.
    pub seconds_of_day: Seconds,
    /// Two-way time-of-flight.
    pub time_of_flight: Seconds,
    /// System configuration ID (often "std" or numeric code).
    pub system_config_id: String,
    /// `10` for full-rate, `11` for normal-point.
    pub record_type: u8,
}

/// Parsed CRD file (header + observations).
#[derive(Debug, Clone, Default)]
pub struct CrdFile {
    /// Station name (H2 record).
    pub station_name: String,
    /// CDP/Pad number (H2).
    pub station_cdp_pad: i32,
    /// Satellite name (H3).
    pub satellite_name: String,
    /// SIC (Satellite Identification Code).
    pub satellite_sic: i32,
    /// COSPAR/NORAD-style ID, kept as string.
    pub satellite_norad: String,
    /// Year of session (H4).
    pub year: i32,
    /// Month of session (H4).
    pub month: u32,
    /// Day of session (H4).
    pub day: u32,
    /// Range observations.
    pub ranges: Vec<CrdRange>,
}

/// Read a CRD file from disk.
pub fn read_crd<P: AsRef<Path>>(path: P) -> Result<CrdFile, PodIoError> {
    let text = fs::read_to_string(path)?;
    parse_crd(&text)
}

/// Parse a CRD file from a string slice.
pub fn parse_crd(text: &str) -> Result<CrdFile, PodIoError> {
    let mut out = CrdFile::default();
    let mut current_sys = String::from("std");

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
            "H1" => { /* format header — ignored */ }
            "H2" => {
                // H2 station_name cdp_pad sys_no occ_seq tz
                if let Some(s) = tokens.next() {
                    out.station_name = s.to_string();
                }
                if let Some(c) = tokens.next() {
                    out.station_cdp_pad = c.parse().unwrap_or(0);
                }
            }
            "H3" => {
                // H3 satellite_name sic norad spc epoch_id
                if let Some(s) = tokens.next() {
                    out.satellite_name = s.to_string();
                }
                if let Some(s) = tokens.next() {
                    out.satellite_sic = s.parse().unwrap_or(0);
                }
                if let Some(s) = tokens.next() {
                    out.satellite_norad = s.to_string();
                }
            }
            "H4" => {
                // H4 type year month day hour min sec ...
                let _ = tokens.next();
                if let Some(s) = tokens.next() {
                    out.year = s.parse().unwrap_or(0);
                }
                if let Some(s) = tokens.next() {
                    out.month = s.parse().unwrap_or(0);
                }
                if let Some(s) = tokens.next() {
                    out.day = s.parse().unwrap_or(0);
                }
            }
            "C0" => {
                // System config — record an ID we can attach to ranges.
                let _ = tokens.next(); // detail type
                if let Some(s) = tokens.next() {
                    current_sys = s.to_string();
                }
            }
            "10" | "11" => {
                let kind: u8 = if tag == "11" { 11 } else { 10 };
                let sod: f64 = tokens
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| PodIoError::Format("CRD range: missing SOD".into()))?;
                let tof: f64 = tokens
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| PodIoError::Format("CRD range: missing TOF".into()))?;
                out.ranges.push(CrdRange {
                    seconds_of_day: Seconds::new(sod),
                    time_of_flight: Seconds::new(tof),
                    system_config_id: current_sys.clone(),
                    record_type: kind,
                });
            }
            _ => {
                // skip everything else (config, meteo, calibration, statistics)
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_crd() {
        let txt = "\
H1 CRD 2 2024 01 01 00\n\
H2 7090 7090 1 01 0\n\
H3 lageos1 1155 7603901 8820 1\n\
H4 0 2024 01 01 00 00 00 00 00 00 0 0 0 0 1 0\n\
C0 0 std\n\
11 12345.000 0.05123456789 std 1 1 1 1 0 0\n\
11 12375.000 0.05123446712 std 1 1 1 1 0 0\n\
H8\n";
        let f = parse_crd(txt).expect("parse");
        assert_eq!(f.station_name, "7090");
        assert_eq!(f.satellite_name, "lageos1");
        assert_eq!(f.year, 2024);
        assert_eq!(f.ranges.len(), 2);
        assert!((f.ranges[0].time_of_flight.value() - 0.05123456789).abs() < 1e-15);
        assert_eq!(f.ranges[0].record_type, 11);
    }
}
