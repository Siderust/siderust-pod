//! Functional tests for the CRD SLR observation format.
//!
//! Fixture: `test-data/tiny/sample.crd` — station GRAZ, target LAGEOS-1,
//! 4 normal-point range observations on 2024-01-01.

use super::common::{assert_approx, assert_strictly_increasing, tiny};
use siderust_pod_io::crd::parse_crd;
use siderust_pod_io::PodIoError;

// ── helpers ──────────────────────────────────────────────────────────────────

fn load() -> siderust_pod_io::crd::CrdFile {
    let content = std::fs::read_to_string(tiny("sample.crd")).expect("read sample.crd");
    parse_crd(&content).expect("parse sample.crd")
}

// ── header tests ─────────────────────────────────────────────────────────────

#[test]
fn crd_station_name() {
    assert_eq!(load().station_name, "GRAZ");
}

#[test]
fn crd_station_cdp_pad() {
    assert_eq!(load().station_cdp_pad, 7839);
}

#[test]
fn crd_satellite_name() {
    assert_eq!(load().satellite_name, "lageos1");
}

#[test]
fn crd_satellite_sic() {
    assert_eq!(load().satellite_sic, 1155);
}

#[test]
fn crd_satellite_norad() {
    assert_eq!(load().satellite_norad, "7603901");
}

#[test]
fn crd_session_date() {
    let f = load();
    assert_eq!(f.year, 2024, "session year");
    assert_eq!(f.month, 1, "session month");
    assert_eq!(f.day, 1, "session day");
}

// ── observation tests ─────────────────────────────────────────────────────────

#[test]
fn crd_range_count() {
    assert_eq!(load().ranges.len(), 4, "expected 4 range observations");
}

#[test]
fn crd_range_record_type() {
    for rng in &load().ranges {
        assert_eq!(
            rng.record_type, 11,
            "all fixture ranges should be normal-point (11)"
        );
    }
}

#[test]
fn crd_first_range_values() {
    let f = load();
    let r0 = &f.ranges[0];
    assert_approx(
        r0.seconds_of_day.value(),
        28_800.0,
        1e-9,
        "range[0].seconds_of_day [s]",
    );
    // LAGEOS-1 two-way time-of-flight ~51 ms (slant range ~7700 km).
    assert_approx(
        r0.time_of_flight.value(),
        0.051_234_567_890,
        1e-12,
        "range[0].time_of_flight [s]",
    );
}

#[test]
fn crd_observation_times_strictly_increasing() {
    let sods: Vec<f64> = load()
        .ranges
        .iter()
        .map(|r| r.seconds_of_day.value())
        .collect();
    assert_strictly_increasing(&sods, "CRD seconds-of-day");
}

#[test]
fn crd_time_of_flight_positive_finite() {
    for (i, rng) in load().ranges.iter().enumerate() {
        let tof = rng.time_of_flight.value();
        assert!(
            tof.is_finite() && tof > 0.0,
            "range[{i}]: time_of_flight={tof} not positive-finite"
        );
    }
}

#[test]
fn crd_system_config_id_recorded() {
    // All records in the fixture use the "std" config from the C0 record.
    for rng in &load().ranges {
        assert_eq!(rng.system_config_id, "std", "unexpected system_config_id");
    }
}

// ── malformed-input tests ────────────────────────────────────────────────────

#[test]
fn crd_malformed_range_missing_sod() {
    // "11" record with no following tokens → missing SOD error.
    let bad = "H2 TEST 1234 1 01 0\nH3 sat 0 0\nH4 0 2024 1 1\n11\n";
    let err = parse_crd(bad).expect_err("missing SOD should fail");
    match err {
        PodIoError::Format(msg) => assert!(
            msg.contains("SOD"),
            "expected SOD in error message, got: {msg}"
        ),
        other => panic!("expected PodIoError::Format, got {other:?}"),
    }
}

#[test]
fn crd_malformed_range_missing_tof() {
    // "11" record with SOD but no TOF → missing TOF error.
    let bad = "H2 TEST 1234 1 01 0\nH3 sat 0 0\nH4 0 2024 1 1\n11 12345.0\n";
    let err = parse_crd(bad).expect_err("missing TOF should fail");
    match err {
        PodIoError::Format(msg) => assert!(
            msg.contains("TOF"),
            "expected TOF in error message, got: {msg}"
        ),
        other => panic!("expected PodIoError::Format, got {other:?}"),
    }
}

#[test]
fn crd_malformed_non_numeric_sod() {
    // "11" record where SOD is not a number.
    let bad = "H2 TEST 1234 1 01 0\nH3 sat 0 0\nH4 0 2024 1 1\n11 NOTANUMBER 0.05\n";
    let err = parse_crd(bad).expect_err("non-numeric SOD should fail");
    assert!(matches!(err, PodIoError::Format(_)));
}

#[test]
fn crd_unknown_tags_silently_ignored() {
    // Tags not in the handled set (H2, H3, H4, C0, 10, 11) are skipped.
    let txt = "H1 CRD 2 2024 01 01 00\nXX unknown tag\nH2 GRAZ 7839 1 01 0\nH8\n";
    let f = parse_crd(txt).expect("unknown tag should not cause error");
    assert_eq!(f.station_name, "GRAZ");
}

// ── heavy/ignored tests ──────────────────────────────────────────────────────

/// Place an ILRS CRD file (from https://cddis.nasa.gov/archive/slr/data/npt_crd/)
/// at `test-data/official/ilrs.crd` and remove `#[ignore]`.
#[test]
#[ignore]
fn crd_official_ilrs_file() {
    let path = super::common::official("ilrs.crd");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("place ILRS CRD file at {}", path.display()));
    let f = parse_crd(&content).expect("parse official CRD");
    assert!(!f.station_name.is_empty(), "station name must not be empty");
    assert!(!f.ranges.is_empty(), "official CRD must have range records");
}
