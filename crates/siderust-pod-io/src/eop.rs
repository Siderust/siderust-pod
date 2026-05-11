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
use qtty::angular::Arcseconds;
use qtty::time::Seconds;
use qtty::Day;
use std::io::{BufRead, BufReader, Read};
use tempoch::{ModifiedJulianDate, UTC};

/// A single Earth-orientation record from IERS C04.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EopRecord {
    /// Modified Julian Date (UTC).
    pub mjd: ModifiedJulianDate<UTC>,
    /// Polar motion x.
    pub x: Arcseconds,
    /// Polar motion y.
    pub y: Arcseconds,
    /// UT1 − UTC.
    pub ut1_utc: Seconds,
    /// Length-of-day excess.
    pub lod: Seconds,
    /// Nutation correction dψ.
    pub dpsi: Arcseconds,
    /// Nutation correction dε.
    pub deps: Arcseconds,
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
        let mjd_raw: f64 = match parts[3].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let mjd = match ModifiedJulianDate::<UTC>::try_new(Day::new(mjd_raw)) {
            Ok(m) => m,
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
            x: Arcseconds::new(x),
            y: Arcseconds::new(y),
            ut1_utc: Seconds::new(ut1),
            lod: Seconds::new(lod),
            dpsi: Arcseconds::new(dpsi),
            deps: Arcseconds::new(deps),
        });
    }
    Ok(out)
}

/// Linear interpolation of an EOP record at a target MJD.
///
/// Returns `None` if the table is empty; clamps to the endpoints when the
/// requested epoch lies outside the table range.
pub fn interpolate(records: &[EopRecord], mjd: ModifiedJulianDate<UTC>) -> Option<EopRecord> {
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
    let a_val = a.mjd.raw().value();
    let b_val = b.mjd.raw().value();
    let t_val = mjd.raw().value();
    let f = (t_val - a_val) / (b_val - a_val);
    Some(EopRecord {
        mjd,
        x: Arcseconds::new(a.x.value() + f * (b.x.value() - a.x.value())),
        y: Arcseconds::new(a.y.value() + f * (b.y.value() - a.y.value())),
        ut1_utc: Seconds::new(a.ut1_utc.value() + f * (b.ut1_utc.value() - a.ut1_utc.value())),
        lod: Seconds::new(a.lod.value() + f * (b.lod.value() - a.lod.value())),
        dpsi: Arcseconds::new(a.dpsi.value() + f * (b.dpsi.value() - a.dpsi.value())),
        deps: Arcseconds::new(a.deps.value() + f * (b.deps.value() - a.deps.value())),
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
        assert!((r[0].mjd.raw().value() - 60310.0).abs() < 1e-9);
    }

    #[test]
    fn interpolates_midpoint() {
        let r = read_eop_c04(SAMPLE.as_bytes()).unwrap();
        let mid = interpolate(&r, ModifiedJulianDate::<UTC>::try_new(Day::new(60310.5)).unwrap()).unwrap();
        let expect = (0.123456 + 0.124000) / 2.0;
        assert!((mid.x.value() - expect).abs() < 1e-9);
    }
}
