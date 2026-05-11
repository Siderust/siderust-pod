//! Functional tests for the RINEX observation format.
//!
//! Fixture: `test-data/tiny/sample_obs.rnx` — RINEX 3.04 mixed (GPS + GLONASS),
//! 2 epochs, G03 (3 obs types: C1C L1C S1C) and R08 (2 obs types: C1C L1C).

use super::common::{assert_approx, tiny};
use siderust_pod_io::rinex_obs::read_rinex_obs;

// ── helpers ──────────────────────────────────────────────────────────────────

fn load() -> siderust_pod_io::rinex_obs::RinexObs {
    let content = std::fs::read_to_string(tiny("sample_obs.rnx")).expect("read sample_obs.rnx");
    read_rinex_obs(content.as_bytes()).expect("parse sample_obs.rnx")
}

// ── header tests ─────────────────────────────────────────────────────────────

#[test]
fn rinex_obs_marker_name() {
    assert_eq!(load().marker, "FUNCTIONAL-TEST");
}

#[test]
fn rinex_obs_approx_xyz_present_and_correct() {
    let xyz = load()
        .approx_xyz_m
        .expect("APPROX POSITION XYZ should be parsed");
    assert_approx(xyz[0].value(), 3_837_086.1234, 1e-3, "approx_xyz[0] (X)");
    assert_approx(xyz[1].value(), -765_588.5678, 1e-3, "approx_xyz[1] (Y)");
    assert_approx(xyz[2].value(), 5_082_259.9012, 1e-3, "approx_xyz[2] (Z)");
}

#[test]
fn rinex_obs_interval() {
    let interval = load().interval_s.expect("INTERVAL should be parsed");
    assert_approx(interval.value(), 30.0, 1e-9, "interval [s]");
}

#[test]
fn rinex_obs_types_gps() {
    let r = load();
    let gps = r.obs_types.get(&'G').expect("GPS obs types should exist");
    assert_eq!(gps.len(), 3, "GPS: expected 3 obs types");
    assert_eq!(gps[0], "C1C");
    assert_eq!(gps[1], "L1C");
    assert_eq!(gps[2], "S1C");
}

#[test]
fn rinex_obs_types_glonass() {
    let r = load();
    let glo = r
        .obs_types
        .get(&'R')
        .expect("GLONASS obs types should exist");
    assert_eq!(glo.len(), 2, "GLONASS: expected 2 obs types");
    assert_eq!(glo[0], "C1C");
    assert_eq!(glo[1], "L1C");
}

// ── epoch & observation tests ─────────────────────────────────────────────────

#[test]
fn rinex_obs_epoch_count() {
    assert_eq!(load().epochs.len(), 2, "expected 2 observation epochs");
}

#[test]
fn rinex_obs_epoch0_satellite_count() {
    let r = load();
    assert_eq!(
        r.epochs[0].satellites.len(),
        2,
        "epoch 0: expected 2 satellites"
    );
}

#[test]
fn rinex_obs_epoch0_g03_c1c() {
    let r = load();
    let g03 = r.epochs[0]
        .satellites
        .get("G03")
        .expect("G03 should be in epoch 0");
    let c1c = g03.get("C1C").copied().expect("C1C obs for G03");
    assert_approx(c1c, 20_500_123.456, 1e-3, "G03.C1C [m]");
}

#[test]
fn rinex_obs_epoch0_g03_l1c() {
    let r = load();
    let g03 = r.epochs[0].satellites.get("G03").expect("G03");
    let l1c = g03.get("L1C").copied().expect("L1C obs for G03");
    assert_approx(l1c, 107_878_123.456, 1e-3, "G03.L1C [cycles]");
}

#[test]
fn rinex_obs_epoch0_g03_s1c() {
    let r = load();
    let g03 = r.epochs[0].satellites.get("G03").expect("G03");
    let s1c = g03.get("S1C").copied().expect("S1C obs for G03");
    assert_approx(s1c, 45.0, 1e-3, "G03.S1C [dBHz]");
}

#[test]
fn rinex_obs_epoch0_r08_c1c() {
    let r = load();
    let r08 = r.epochs[0]
        .satellites
        .get("R08")
        .expect("R08 should be in epoch 0");
    let c1c = r08.get("C1C").copied().expect("C1C obs for R08");
    assert_approx(c1c, 21_200_456.789, 1e-3, "R08.C1C [m]");
}

#[test]
fn rinex_obs_epoch1_g03_c1c_different_from_epoch0() {
    let r = load();
    let c1c_e0 = r.epochs[0].satellites["G03"]["C1C"];
    let c1c_e1 = r.epochs[1].satellites["G03"]["C1C"];
    assert!(
        (c1c_e0 - c1c_e1).abs() > 1.0,
        "C1C values should differ across epochs (satellite motion)"
    );
}

// ── malformed-input tests ────────────────────────────────────────────────────

#[test]
fn rinex_obs_malformed_missing_end_of_header() {
    // File ends before END OF HEADER → unexpected EOF error.
    let bad = "     3.04           OBSERVATION DATA    M\nMARKER NAME\nTEST\n";
    let err = read_rinex_obs(bad.as_bytes()).expect_err("missing END OF HEADER should fail");
    match err {
        siderust_pod_io::PodIoError::Format(msg) => {
            assert!(msg.contains("EOF"), "expected EOF in error: {msg}");
        }
        other => panic!("expected Format error, got {other:?}"),
    }
}

#[test]
fn rinex_obs_malformed_invalid_epoch_month() {
    // Month 13 is invalid → epoch date parse error.
    let bad = "\
     3.04           OBSERVATION DATA    M\n\
                                                            END OF HEADER\n\
> 2024 13 01 00 00  0.0000000  0  1\n\
G01  20000000.000\n";
    let err = read_rinex_obs(bad.as_bytes()).expect_err("invalid month should fail");
    match err {
        siderust_pod_io::PodIoError::Format(_) => {}
        other => panic!("expected Format error, got {other:?}"),
    }
}

// ── heavy/ignored tests ──────────────────────────────────────────────────────

/// Place a RINEX 3 OBS file from a IGS station (e.g. from CDDIS
/// https://cddis.nasa.gov/archive/gnss/data/daily/) at
/// `test-data/official/station_obs.rnx` and remove `#[ignore]`.
#[test]
#[ignore]
fn rinex_obs_official_large_file() {
    let path = super::common::official("station_obs.rnx");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("place official RINEX OBS at {}", path.display()));
    let r = read_rinex_obs(content.as_bytes()).expect("parse official RINEX OBS");
    assert!(!r.epochs.is_empty(), "official file must have epochs");
    assert!(!r.marker.is_empty(), "marker name must not be empty");
}
