//! # IERS C04 Earth-orientation records
//!
//! ## Scientific scope
//!
//! Earth-orientation parameters link terrestrial and celestial reference
//! frames by supplying polar motion, UT1-UTC, and related series. This
//! module reads the daily IERS C04 product that underpins precise Earth
//! rotation in POD frame transforms.
//!
//! Its validity regime is the cadence and fields offered by the C04 series.
//! Higher-order interpolation policy and any conversion into richer EOP
//! dataset abstractions are delegated downstream.
//!
//! ## Technical scope
//!
//! The module exports `EopRecord`, `read_eop_c04`, and a simple
//! `interpolate` helper. Records expose Modified Julian Date tags and the
//! standard C04 scalar fields used by frame-transform providers.
//!
//! This code does not itself build a full transformation matrix or bind to
//! `siderust` frame APIs; it only prepares the underlying geodetic time-
//! series inputs.
//!
//! ## References
//!
//! - Bizouard, C., & Gambis, D. (2009). The combined solution C04 for Earth
//!   Orientation Parameters. IERS Technical Note / Observatoire de Paris
//!   release documentation.
//! - IERS Conventions Centre. (2010). IERS Conventions (2010). Verlag des
//!   Bundesamts fur Kartographie und Geodasie.
use crate::PodIoError;
use std::io::{BufRead, BufReader, Read};

/// A single Earth-orientation record from IERS C04.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EopRecord {
    /// Modified Julian Date (UTC).
    pub mjd: f64,
    /// Polar motion x, arcseconds.
    pub x_arcsec: f64,
    /// Polar motion y, arcseconds.
    pub y_arcsec: f64,
    /// UT1 − UTC, seconds.
    pub ut1_utc_s: f64,
    /// Length-of-day excess, seconds.
    pub lod_s: f64,
    /// Nutation correction dψ, arcseconds.
    pub dpsi_arcsec: f64,
    /// Nutation correction dε, arcseconds.
    pub deps_arcsec: f64,
}

/// Parse a C04-style EOP file.
pub fn read_eop_c04<R: Read>(rdr: R) -> Result<Vec<EopRecord>, PodIoError> {
    let br = BufReader::new(rdr);
    let mut out = Vec::new();
    for line in br.lines() {
        let line = line?;
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }
        // Heuristic: a valid data line has the first three fields parseable as integers
        // (year, month, day) and the fourth as a float (MJD).
        if parts[0].parse::<i32>().is_err()
            || parts[1].parse::<i32>().is_err()
            || parts[2].parse::<i32>().is_err()
        {
            continue;
        }
        let mjd: f64 = match parts[3].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let x = parts[4].parse().unwrap_or(0.0);
        let y = parts[5].parse().unwrap_or(0.0);
        let ut1 = parts[6].parse().unwrap_or(0.0);
        let lod = parts[7].parse().unwrap_or(0.0);
        let dpsi = parts[8].parse().unwrap_or(0.0);
        let deps = parts[9].parse().unwrap_or(0.0);
        out.push(EopRecord {
            mjd,
            x_arcsec: x,
            y_arcsec: y,
            ut1_utc_s: ut1,
            lod_s: lod,
            dpsi_arcsec: dpsi,
            deps_arcsec: deps,
        });
    }
    Ok(out)
}

/// Linear interpolation of an EOP record at a target MJD.
///
/// Returns `None` if the table is empty; clamps to the endpoints when the
/// requested epoch lies outside the table range.
pub fn interpolate(records: &[EopRecord], mjd: f64) -> Option<EopRecord> {
    if records.is_empty() {
        return None;
    }
    if mjd <= records[0].mjd {
        return Some(records[0]);
    }
    if mjd >= records[records.len() - 1].mjd {
        return Some(records[records.len() - 1]);
    }
    // Binary search for the bracketing pair.
    let mut lo = 0usize;
    let mut hi = records.len() - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if records[mid].mjd <= mjd {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let a = records[lo];
    let b = records[hi];
    let f = (mjd - a.mjd) / (b.mjd - a.mjd);
    Some(EopRecord {
        mjd,
        x_arcsec: a.x_arcsec + f * (b.x_arcsec - a.x_arcsec),
        y_arcsec: a.y_arcsec + f * (b.y_arcsec - a.y_arcsec),
        ut1_utc_s: a.ut1_utc_s + f * (b.ut1_utc_s - a.ut1_utc_s),
        lod_s: a.lod_s + f * (b.lod_s - a.lod_s),
        dpsi_arcsec: a.dpsi_arcsec + f * (b.dpsi_arcsec - a.dpsi_arcsec),
        deps_arcsec: a.deps_arcsec + f * (b.deps_arcsec - a.deps_arcsec),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# IERS C04 sample
2024  1  1  60310    0.123456    0.234567    0.012345    0.001234    0.0001    0.0002
2024  1  2  60311    0.124000    0.235000    0.012000    0.001200    0.0001    0.0002
";

    #[test]
    fn parses_two_records() {
        let r = read_eop_c04(SAMPLE.as_bytes()).unwrap();
        assert_eq!(r.len(), 2);
        assert!((r[0].mjd - 60310.0).abs() < 1e-9);
    }

    #[test]
    fn interpolates_midpoint() {
        let r = read_eop_c04(SAMPLE.as_bytes()).unwrap();
        let mid = interpolate(&r, 60310.5).unwrap();
        let expect = (0.123456 + 0.124000) / 2.0;
        assert!((mid.x_arcsec - expect).abs() < 1e-9);
    }
}
