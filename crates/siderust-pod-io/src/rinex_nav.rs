//! RINEX 3 NAV (broadcast ephemeris) reader — MVP subset.
//!
//! Reads GPS broadcast navigation messages: the eight 4-element data lines
//! per record. Only the fields actually consumed by `pod-observations` are
//! parsed; everything else is preserved as the raw broadcast value.
//!
//! The intent is to be permissive: any line that doesn't fit the expected
//! shape is skipped rather than failing the whole file, so partial files
//! still produce usable ephemerides.

use crate::PodIoError;
use std::fs;
use std::path::Path;

/// One GPS broadcast navigation record (subset).
#[derive(Debug, Clone, Default)]
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
    pub second: f64,
    /// SV clock bias (s).
    pub af0: f64,
    /// SV clock drift (s/s).
    pub af1: f64,
    /// SV clock drift rate (s/s²).
    pub af2: f64,
    /// IODE.
    pub iode: f64,
    /// Crs (m).
    pub crs: f64,
    /// Δn (rad/s).
    pub delta_n: f64,
    /// M0 (rad).
    pub m0: f64,
    /// Cuc (rad).
    pub cuc: f64,
    /// Eccentricity.
    pub e: f64,
    /// Cus (rad).
    pub cus: f64,
    /// √a (m^½).
    pub sqrt_a: f64,
    /// Toe (s of GPS week).
    pub toe: f64,
    /// Cic (rad).
    pub cic: f64,
    /// Ω0 (rad).
    pub omega0: f64,
    /// Cis (rad).
    pub cis: f64,
    /// i0 (rad).
    pub i0: f64,
    /// Crc (m).
    pub crc: f64,
    /// ω (rad).
    pub omega: f64,
    /// Ω̇ (rad/s).
    pub omega_dot: f64,
    /// IDOT (rad/s).
    pub idot: f64,
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
            second: toc_tokens[5].parse().unwrap_or(0.0),
            af0: parse_d(toc_tokens[6]),
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
            rec.crs = v1[1];
            rec.delta_n = v1[2];
            rec.m0 = v1[3];
        }
        if v2.len() >= 4 {
            rec.cuc = v2[0];
            rec.e = v2[1];
            rec.cus = v2[2];
            rec.sqrt_a = v2[3];
        }
        if v3.len() >= 4 {
            rec.toe = v3[0];
            rec.cic = v3[1];
            rec.omega0 = v3[2];
            rec.cis = v3[3];
        }
        if v4.len() >= 4 {
            rec.i0 = v4[0];
            rec.crc = v4[1];
            rec.omega = v4[2];
            rec.omega_dot = v4[3];
        }
        if !v5.is_empty() {
            rec.idot = v5[0];
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
        assert!((r.toe - 518_400.0).abs() < 1e-3);
    }
}
