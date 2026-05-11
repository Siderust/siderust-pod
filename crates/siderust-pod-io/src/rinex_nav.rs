//! # RINEX navigation message reader
//!
//! ## Scientific scope
//!
//! Broadcast navigation files encode the coarse orbital and clock
//! information transmitted by GNSS spacecraft. This module reads the GPS-
//! focused subset needed to reconstruct broadcast ephemerides for
//! observation modelling and comparative POD workflows.
//!
//! The scientific regime is therefore limited to the message families and
//! fields currently parsed from RINEX 3 NAV. Unsupported systems and
//! optional record features are intentionally left for follow-up work.
//!
//! ## Technical scope
//!
//! The main entry points are `read_rinex_nav` and `parse_rinex_nav`,
//! returning `RinexNavFile` collections of `GpsNavRecord` values. Parsed
//! fields stay close to the broadcast representation so downstream code can
//! choose how to convert them into propagated states.
//!
//! This module does not itself evaluate the broadcast model or perform
//! orbit determination.
//!
//! ## References
//!
//! - International GNSS Service / RTCM. (2020). RINEX: The Receiver
//!   Independent Exchange Format, Version 3.05.
//! - IS-GPS-200. (current revision). Navstar GPS Space Segment / Navigation
//!   User Interfaces.
use crate::PodIoError;
use qtty::angular::Radians;
use qtty::angular_rate::AngularRate;
use qtty::length::Meters;
use qtty::time::Seconds;
use qtty::unit::{Radian, Second};
use std::fs;
use std::path::Path;

/// One GPS broadcast navigation record (subset).
#[derive(Debug, Clone)]
pub struct GpsNavRecord {
    /// PRN identifier (e.g. `G01` → 1).
    pub prn: u8,
    /// TOC year (4-digit).
    pub year: i32,
    /// TOC month.
    pub month: u32,
    /// TOC day.
    pub day: u32,
    /// TOC hour.
    pub hour: u32,
    /// TOC minute.
    pub minute: u32,
    /// TOC seconds.
    pub second: Seconds,
    /// SV clock bias.
    pub af0: Seconds,
    /// SV clock drift (s/s — dimensionless rate; kept as scalar).
    pub af1: f64,
    /// SV clock drift rate (s/s² — kept as scalar).
    pub af2: f64,
    /// IODE (dimensionless index).
    pub iode: f64,
    /// Crs amplitude correction.
    pub crs: Meters,
    /// Δn — mean motion correction.
    pub delta_n: AngularRate<Radian, Second>,
    /// M0 — mean anomaly at reference time.
    pub m0: Radians,
    /// Cuc — argument-of-latitude correction (cosine).
    pub cuc: Radians,
    /// Eccentricity (dimensionless).
    pub e: f64,
    /// Cus — argument-of-latitude correction (sine).
    pub cus: Radians,
    /// √a (m^½ — composite dimension; kept as scalar).
    pub sqrt_a: f64,
    /// Toe — reference time of ephemeris.
    pub toe: Seconds,
    /// Cic — inclination correction (cosine).
    pub cic: Radians,
    /// Ω0 — longitude of ascending node at weekly epoch.
    pub omega0: Radians,
    /// Cis — inclination correction (sine).
    pub cis: Radians,
    /// i0 — inclination angle at reference time.
    pub i0: Radians,
    /// Crc amplitude correction.
    pub crc: Meters,
    /// ω — argument of perigee.
    pub omega: Radians,
    /// Ω̇ — rate of right ascension.
    pub omega_dot: AngularRate<Radian, Second>,
    /// IDOT — rate of inclination angle.
    pub idot: AngularRate<Radian, Second>,
}

impl Default for GpsNavRecord {
    fn default() -> Self {
        Self {
            prn: 0,
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: Seconds::new(0.0),
            af0: Seconds::new(0.0),
            af1: 0.0,
            af2: 0.0,
            iode: 0.0,
            crs: Meters::new(0.0),
            delta_n: AngularRate::new(0.0),
            m0: Radians::new(0.0),
            cuc: Radians::new(0.0),
            e: 0.0,
            cus: Radians::new(0.0),
            sqrt_a: 0.0,
            toe: Seconds::new(0.0),
            cic: Radians::new(0.0),
            omega0: Radians::new(0.0),
            cis: Radians::new(0.0),
            i0: Radians::new(0.0),
            crc: Meters::new(0.0),
            omega: Radians::new(0.0),
            omega_dot: AngularRate::new(0.0),
            idot: AngularRate::new(0.0),
        }
    }
}

/// Parsed RINEX NAV file.
#[derive(Debug, Clone, Default)]
pub struct RinexNavFile {
    /// All GPS broadcast records found in the file.
    pub gps: Vec<GpsNavRecord>,
}

/// Read a RINEX NAV file from disk.
pub fn read_rinex_nav<P: AsRef<Path>>(path: P) -> Result<RinexNavFile, PodIoError> {
    let text = fs::read_to_string(path)?;
    parse_rinex_nav(&text)
}

/// Parse a RINEX NAV file from a string slice.
pub fn parse_rinex_nav(text: &str) -> Result<RinexNavFile, PodIoError> {
    let mut out = RinexNavFile::default();

    let mut lines = text.lines();
    // Skip header up to and including "END OF HEADER".
    for l in lines.by_ref() {
        if l.contains("END OF HEADER") {
            break;
        }
    }

    let collected: Vec<&str> = lines.collect();
    let mut i = 0;
    while i + 7 < collected.len() {
        let l0 = collected[i];
        if l0.len() < 4 || !(l0.starts_with('G') || l0.as_bytes()[0].is_ascii_digit()) {
            i += 1;
            continue;
        }
        let prn = if l0.starts_with('G') {
            l0[1..3].trim().parse::<u8>().unwrap_or(0)
        } else {
            l0[..3].trim().parse::<u8>().unwrap_or(0)
        };
        if prn == 0 {
            i += 1;
            continue;
        }

        // TOC: positions 4..23 → "YYYY MM DD HH MM SS"
        let rest = &l0[3..];
        let toc_tokens: Vec<&str> = rest.split_whitespace().collect();
        if toc_tokens.len() < 9 {
            i += 1;
            continue;
        }
        let mut rec = GpsNavRecord {
            prn,
            year: toc_tokens[0].parse().unwrap_or(0),
            month: toc_tokens[1].parse().unwrap_or(0),
            day: toc_tokens[2].parse().unwrap_or(0),
            hour: toc_tokens[3].parse().unwrap_or(0),
            minute: toc_tokens[4].parse().unwrap_or(0),
            second: Seconds::new(toc_tokens[5].parse().unwrap_or(0.0)),
            af0: Seconds::new(parse_d(toc_tokens[6])),
            af1: parse_d(toc_tokens[7]),
            af2: parse_d(toc_tokens[8]),
            ..Default::default()
        };

        // Lines 1..7 each carry up to 4 floats in fixed 19-char fields,
        // but split_whitespace works for our subset.
        let v1: Vec<f64> = collect_floats(collected[i + 1]);
        let v2: Vec<f64> = collect_floats(collected[i + 2]);
        let v3: Vec<f64> = collect_floats(collected[i + 3]);
        let v4: Vec<f64> = collect_floats(collected[i + 4]);
        let v5: Vec<f64> = collect_floats(collected[i + 5]);
        let v6: Vec<f64> = collect_floats(collected[i + 6]);
        let v7: Vec<f64> = collect_floats(collected[i + 7]);

        if v1.len() >= 4 {
            rec.iode = v1[0];
            rec.crs = Meters::new(v1[1]);
            rec.delta_n = AngularRate::new(v1[2]);
            rec.m0 = Radians::new(v1[3]);
        }
        if v2.len() >= 4 {
            rec.cuc = Radians::new(v2[0]);
            rec.e = v2[1];
            rec.cus = Radians::new(v2[2]);
            rec.sqrt_a = v2[3];
        }
        if v3.len() >= 4 {
            rec.toe = Seconds::new(v3[0]);
            rec.cic = Radians::new(v3[1]);
            rec.omega0 = Radians::new(v3[2]);
            rec.cis = Radians::new(v3[3]);
        }
        if v4.len() >= 4 {
            rec.i0 = Radians::new(v4[0]);
            rec.crc = Meters::new(v4[1]);
            rec.omega = Radians::new(v4[2]);
            rec.omega_dot = AngularRate::new(v4[3]);
        }
        if !v5.is_empty() {
            rec.idot = AngularRate::new(v5[0]);
        }
        let _ = v6;
        let _ = v7;

        out.gps.push(rec);
        i += 8;
    }

    Ok(out)
}

fn parse_d(s: &str) -> f64 {
    s.replace('D', "E").replace('d', "e").parse().unwrap_or(0.0)
}

fn collect_floats(line: &str) -> Vec<f64> {
    line.split_whitespace().map(parse_d).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_one_gps_record() {
        let txt = "\
     3.04           N: GNSS NAV DATA    M (Mixed)           RINEX VERSION / TYPE\n\
                                                            END OF HEADER\n\
G01 2024 01 01 00 00 00 1.234567E-04 5.678E-12 0.000E+00\n\
     1.000000E+00 2.000000E+01 3.456E-09 1.234567E+00\n\
     5.000E-07 1.000E-03 9.000E-07 5.153651E+03\n\
     5.184000E+05 1.000E-08 1.000E+00 2.000E-08\n\
     9.760000E-01 2.500E+02 -1.500E+00 -8.000E-09\n\
     1.000E-10\n\
     0.000E+00 0.000E+00 0.000E+00 0.000E+00\n\
     0.000E+00 0.000E+00 0.000E+00 0.000E+00\n";
        let f = parse_rinex_nav(txt).expect("parse");
        assert_eq!(f.gps.len(), 1);
        let r = &f.gps[0];
        assert_eq!(r.prn, 1);
        assert!((r.sqrt_a - 5_153.651).abs() < 1e-3);
        assert!((r.e - 1e-3).abs() < 1e-9);
        assert!((r.toe.value() - 518_400.0).abs() < 1e-3);
    }
}
