//! Minimal ANTEX parser — phase-centre offsets only.
//!
//! Reads the PCO (north / east / up, mm) for each frequency block of every
//! antenna in the file. PCV grid values are skipped. Sufficient to apply
//! antenna offsets in MVP-1 GNSS observation modelling.

use crate::PodIoError;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

/// PCO entry for one antenna and one frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pco {
    /// North offset, millimetres.
    pub n_mm: f64,
    /// East offset, millimetres.
    pub e_mm: f64,
    /// Up offset, millimetres.
    pub u_mm: f64,
}

/// Per-antenna PCO data, keyed by frequency identifier (e.g. "G01" for GPS L1).
pub type AntennaPco = HashMap<String, Pco>;

/// Parsed ANTEX file: `antenna_name -> per-frequency PCO`.
pub type AntexCatalog = HashMap<String, AntennaPco>;

/// Parse an ANTEX file (subset).
pub fn read_antex<R: Read>(rdr: R) -> Result<AntexCatalog, PodIoError> {
    let br = BufReader::new(rdr);
    let mut catalog = AntexCatalog::new();
    let mut current_antenna: Option<String> = None;
    let mut current_pcos: AntennaPco = AntennaPco::new();
    let mut current_freq: Option<String> = None;

    for line in br.lines() {
        let line = line?;
        let label = line.get(60..).unwrap_or("").trim();
        let body = line.get(..60).unwrap_or("");
        match label {
            "START OF ANTENNA" => {
                current_antenna = None;
                current_pcos.clear();
                current_freq = None;
            }
            "TYPE / SERIAL NO" => {
                current_antenna = Some(body.trim().to_string());
            }
            "START OF FREQUENCY" => {
                current_freq = body.split_whitespace().next().map(|s| s.to_string());
            }
            "NORTH / EAST / UP" => {
                if let Some(freq) = &current_freq {
                    let parts: Vec<f64> = body
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if parts.len() == 3 {
                        current_pcos.insert(
                            freq.clone(),
                            Pco {
                                n_mm: parts[0],
                                e_mm: parts[1],
                                u_mm: parts[2],
                            },
                        );
                    }
                }
            }
            "END OF FREQUENCY" => {
                current_freq = None;
            }
            "END OF ANTENNA" => {
                if let Some(name) = current_antenna.take() {
                    catalog.insert(name, std::mem::take(&mut current_pcos));
                }
            }
            _ => {}
        }
    }
    Ok(catalog)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
                                                            START OF ANTENNA
TEST-ANTENNA   ABC                                          TYPE / SERIAL NO
   G01                                                      START OF FREQUENCY
      1.50      0.20     90.00                              NORTH / EAST / UP
                                                            END OF FREQUENCY
   G02                                                      START OF FREQUENCY
      0.50     -0.30     85.00                              NORTH / EAST / UP
                                                            END OF FREQUENCY
                                                            END OF ANTENNA
";

    #[test]
    fn parses_pco_blocks() {
        let cat = read_antex(SAMPLE.as_bytes()).unwrap();
        let ant = &cat["TEST-ANTENNA   ABC"];
        let g01 = ant["G01"];
        assert!((g01.u_mm - 90.0).abs() < 1e-9);
        let g02 = ant["G02"];
        assert!((g02.e_mm + 0.3).abs() < 1e-9);
    }
}
