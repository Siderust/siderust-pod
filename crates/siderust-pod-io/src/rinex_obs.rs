//! Minimal RINEX 3 OBS reader.
//!
//! MVP-1 subset: a single GNSS system block, dual-frequency code/carrier
//! observables identified by their RINEX 3 codes (e.g. `C1C`, `L1C`,
//! `C2W`, `L2W`). The parser handles the header keys we actually use
//! (RINEX VERSION / TYPE, MARKER NAME, APPROX POSITION XYZ, OBS TYPES
//! per system, INTERVAL, TIME OF FIRST OBS, END OF HEADER). Other lines
//! are skipped.

use crate::PodIoError;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

/// One observation epoch.
#[derive(Debug, Clone, PartialEq)]
pub struct ObsEpoch {
    /// Year, month, day.
    pub date: (i32, u32, u32),
    /// Hour, minute, second.
    pub time: (u32, u32, f64),
    /// Per-satellite observations: `sat -> {obs_code -> value}`.
    pub satellites: HashMap<String, HashMap<String, f64>>,
}

/// Parsed RINEX OBS file.
#[derive(Debug, Clone)]
pub struct RinexObs {
    /// Marker name (station name).
    pub marker: String,
    /// Approximate XYZ in metres (ITRF).
    pub approx_xyz_m: Option<[f64; 3]>,
    /// Sampling interval in seconds, if declared.
    pub interval_s: Option<f64>,
    /// Per-system observation type list.
    pub obs_types: HashMap<char, Vec<String>>,
    /// Epoch records.
    pub epochs: Vec<ObsEpoch>,
}

/// Parse a RINEX OBS file.
pub fn read_rinex_obs<R: Read>(rdr: R) -> Result<RinexObs, PodIoError> {
    let mut br = BufReader::new(rdr);
    let mut line = String::new();

    let mut marker = String::new();
    let mut approx_xyz_m: Option<[f64; 3]> = None;
    let mut interval_s: Option<f64> = None;
    let mut obs_types: HashMap<char, Vec<String>> = HashMap::new();

    // Header.
    loop {
        line.clear();
        if br.read_line(&mut line)? == 0 {
            return Err(PodIoError::Format("rinex_obs: unexpected EOF".into()));
        }
        let label = line.get(60..).unwrap_or("").trim();
        let body = line.get(..60).unwrap_or("");
        match label {
            "MARKER NAME" => marker = body.trim().to_string(),
            "APPROX POSITION XYZ" => {
                let parts: Vec<f64> = body
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() == 3 {
                    approx_xyz_m = Some([parts[0], parts[1], parts[2]]);
                }
            }
            "INTERVAL" => {
                interval_s = body.split_whitespace().next().and_then(|s| s.parse().ok());
            }
            "SYS / # / OBS TYPES" => {
                let bytes = body.as_bytes();
                if bytes.is_empty() {
                    continue;
                }
                let sys = bytes[0] as char;
                if !sys.is_ascii_alphabetic() {
                    continue;
                }
                let count: usize = body[1..6].trim().parse().unwrap_or(0);
                let mut types: Vec<String> = body[7..]
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                while types.len() < count {
                    let mut cont = String::new();
                    if br.read_line(&mut cont)? == 0 {
                        break;
                    }
                    let c_body = cont.get(..60).unwrap_or("");
                    types.extend(c_body.split_whitespace().map(|s| s.to_string()));
                }
                types.truncate(count);
                obs_types.insert(sys, types);
            }
            "END OF HEADER" => break,
            _ => {}
        }
    }

    // Epochs.
    let mut epochs = Vec::new();
    loop {
        line.clear();
        if br.read_line(&mut line)? == 0 {
            break;
        }
        let trimmed = line.trim_end();
        if !trimmed.starts_with('>') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }
        let date = (
            parts[1].parse().unwrap_or(0),
            parts[2].parse().unwrap_or(0),
            parts[3].parse().unwrap_or(0),
        );
        let time = (
            parts[4].parse().unwrap_or(0),
            parts[5].parse().unwrap_or(0),
            parts[6].parse().unwrap_or(0.0),
        );
        let n_sat: usize = parts[8].parse().unwrap_or(0);

        let mut sats = HashMap::new();
        for _ in 0..n_sat {
            line.clear();
            if br.read_line(&mut line)? == 0 {
                break;
            }
            let row = line.trim_end();
            if row.len() < 3 {
                continue;
            }
            let sat = row[..3].to_string();
            let sys = sat.chars().next().unwrap_or(' ');
            let types = match obs_types.get(&sys) {
                Some(t) => t.clone(),
                None => Vec::new(),
            };
            let mut vals: HashMap<String, f64> = HashMap::new();
            // RINEX 3: each observable occupies 16 columns (F14.3 + 2 status
            // chars). We slice that and parse loosely.
            let payload = &row[3..];
            for (i, ty) in types.iter().enumerate() {
                let start = i * 16;
                if start >= payload.len() {
                    break;
                }
                let end = (start + 14).min(payload.len());
                let raw = payload[start..end].trim();
                if raw.is_empty() {
                    continue;
                }
                if let Ok(v) = raw.parse::<f64>() {
                    vals.insert(ty.clone(), v);
                }
            }
            sats.insert(sat, vals);
        }
        epochs.push(ObsEpoch {
            date,
            time,
            satellites: sats,
        });
    }

    Ok(RinexObs {
        marker,
        approx_xyz_m,
        interval_s,
        obs_types,
        epochs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
     3.04           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE
TEST-MARK                                                   MARKER NAME
  1234567.8901  -2345678.9012   5678901.2345                APPROX POSITION XYZ
G    2 C1C L1C                                              SYS / # / OBS TYPES
    30                                                      INTERVAL
                                                            END OF HEADER
> 2024 01 01 00 00  0.0000000  0  1
G01  20123456.789         123456789.012
> 2024 01 01 00 00 30.0000000  0  1
G01  20124000.000         123457000.000
";

    #[test]
    fn parses_header_and_epochs() {
        let r = read_rinex_obs(SAMPLE.as_bytes()).unwrap();
        assert_eq!(r.marker, "TEST-MARK");
        assert_eq!(r.interval_s, Some(30.0));
        assert_eq!(r.obs_types[&'G'], vec!["C1C", "L1C"]);
        assert_eq!(r.epochs.len(), 2);
        let e0 = &r.epochs[0];
        assert!(e0.satellites.contains_key("G01"));
        let g01 = &e0.satellites["G01"];
        assert!((g01["C1C"] - 20_123_456.789).abs() < 1e-3);
    }
}
