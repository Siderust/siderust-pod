//! Functional tests for the RINEX navigation message format.
//!
//! Fixture: `test-data/tiny/sample_nav.rnx` — RINEX 3.04 GPS NAV,
//! 2 satellite records (G01, G05).

use super::common::{assert_approx, tiny};
use siderust_pod_io::rinex_nav::parse_rinex_nav;

// ── helpers ──────────────────────────────────────────────────────────────────

fn load() -> siderust_pod_io::rinex_nav::RinexNavFile {
    let content = std::fs::read_to_string(tiny("sample_nav.rnx")).expect("read sample_nav.rnx");
    parse_rinex_nav(&content).expect("parse sample_nav.rnx")
}

// ── parse tests ──────────────────────────────────────────────────────────────

#[test]
fn rinex_nav_record_count() {
    assert_eq!(load().gps.len(), 2, "expected 2 GPS nav records");
}

#[test]
fn rinex_nav_g01_prn() {
    assert_eq!(load().gps[0].prn, 1, "G01 PRN should be 1");
}

#[test]
fn rinex_nav_g05_prn() {
    assert_eq!(load().gps[1].prn, 5, "G05 PRN should be 5");
}

#[test]
fn rinex_nav_g01_toc() {
    let r = &load().gps[0];
    assert_eq!(r.year, 2024, "G01 TOC year");
    assert_eq!(r.month, 1, "G01 TOC month");
    assert_eq!(r.day, 1, "G01 TOC day");
    assert_eq!(r.hour, 0, "G01 TOC hour");
    assert_eq!(r.minute, 0, "G01 TOC minute");
    assert_approx(r.second.value(), 0.0, 1e-9, "G01 TOC second");
}

#[test]
fn rinex_nav_g01_clock_fields() {
    let r = &load().gps[0];
    assert_approx(r.af0.value(), 1.234_567e-4, 1e-12, "G01 af0 [s]");
    assert_approx(r.af1, 5.678_901e-12, 1e-18, "G01 af1 [s/s]");
    assert_approx(r.af2, 0.0, 1e-18, "G01 af2 [s/s²]");
}

#[test]
fn rinex_nav_g01_orbital_elements() {
    let r = &load().gps[0];
    // sqrt_a ≈ 5153.651 m^½ → semi-major axis ≈ 26 560 km (realistic GPS MEO).
    assert_approx(r.sqrt_a, 5_153.651, 1e-3, "G01 sqrt_a [m^½]");
    assert_approx(r.e, 1.234_567e-3, 1e-9, "G01 eccentricity");
    // Inclination ~55.9°, within typical GPS range 55–57°.
    assert_approx(r.i0.value(), 0.976, 1e-3, "G01 i0 [rad]");
    assert_approx(r.toe.value(), 518_400.0, 1e-3, "G01 Toe [s]");
    assert_approx(r.m0.value(), 1.234_567, 1e-6, "G01 M0 [rad]");
}

#[test]
fn rinex_nav_g05_orbital_elements() {
    let r = &load().gps[1];
    assert_approx(r.sqrt_a, 5_153.712, 1e-3, "G05 sqrt_a [m^½]");
    assert_approx(r.e, 9.876_543e-3, 1e-9, "G05 eccentricity");
}

#[test]
fn rinex_nav_orbital_elements_plausible() {
    // Plausibility check: GPS semi-major axis 26 500 – 26 600 km.
    for rec in &load().gps {
        let a = rec.sqrt_a.powi(2) / 1_000.0; // km
        assert!(
            a > 26_000.0 && a < 27_000.0,
            "PRN G{:02}: semi-major axis {a:.0} km outside expected GPS range",
            rec.prn
        );
        assert!(
            rec.e >= 0.0 && rec.e < 0.1,
            "PRN G{:02}: eccentricity {} outside GPS range",
            rec.prn,
            rec.e
        );
    }
}

// ── malformed-input tests ────────────────────────────────────────────────────

#[test]
fn rinex_nav_missing_end_of_header_returns_empty() {
    // Parser consumes all lines looking for END OF HEADER; never finding it
    // means no records are parsed (graceful empty result, no error).
    let no_eoh = "G01 2024 01 01 00 00 00 1.0E-04 0.0E+00 0.0E+00\n";
    let f = parse_rinex_nav(no_eoh).expect("should succeed even without header marker");
    assert_eq!(
        f.gps.len(),
        0,
        "without END OF HEADER the data block is never reached"
    );
}

#[test]
fn rinex_nav_truncated_record_ignored() {
    // A record with fewer than 8 continuation lines should be skipped by the
    // `i + 7 < collected.len()` guard.
    let txt = "\
                                                            END OF HEADER\n\
G01 2024 01 01 00 00 00 1.0E-04 0.0E+00 0.0E+00\n\
     1.0E+00 2.0E+00 3.0E-09 4.0E+00\n";
    // Only 2 data lines after END OF HEADER (need 8); parser skips it.
    let f = parse_rinex_nav(txt).expect("truncated record should not error");
    assert_eq!(f.gps.len(), 0, "truncated record must not appear in output");
}

// ── heavy/ignored tests ──────────────────────────────────────────────────────

/// Place a RINEX 3 NAV file (e.g. BRDC combined nav from CDDIS
/// https://cddis.nasa.gov/archive/gnss/data/daily/) at
/// `test-data/official/brdc_nav.rnx` and remove `#[ignore]`.
#[test]
#[ignore]
fn rinex_nav_official_large_file() {
    let path = super::common::official("brdc_nav.rnx");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("place official RINEX NAV at {}", path.display()));
    let f = parse_rinex_nav(&content).expect("parse official RINEX NAV");
    assert!(
        !f.gps.is_empty(),
        "official RINEX NAV must have GPS records"
    );
}
