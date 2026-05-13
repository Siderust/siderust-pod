//! Functional tests for the CRD SLR observation format.
//!
//! Fixtures:
//! - `test-data/tiny/sample.crd` — station GRAZ, target LAGEOS-1,
//!   4 normal-point range observations on 2024-01-01.
//! - `test-data/crd/lageos1_v2.crd` — CRD v2 fixture, 5 NPs, full fields.
//! - `test-data/crd/lageos1_v1.crd` — CRD v1 fixture, 3 NPs, minimal fields.

use super::common::{assert_approx, assert_strictly_increasing, crd_fixture, tiny};
use siderust_pod_io::crd::{parse_crd, parse_crd_with_mode};
use siderust_pod_io::{ParseMode, PodIoError};

// ── helpers ──────────────────────────────────────────────────────────────────

fn load() -> siderust_pod_io::crd::CrdFile {
    let content = std::fs::read_to_string(tiny("sample.crd")).expect("read sample.crd");
    parse_crd(&content).expect("parse sample.crd")
}

fn load_v2() -> siderust_pod_io::crd::CrdFile {
    let content =
        std::fs::read_to_string(crd_fixture("lageos1_v2.crd")).expect("read lageos1_v2.crd");
    parse_crd(&content).expect("parse lageos1_v2.crd")
}

fn load_v1() -> siderust_pod_io::crd::CrdFile {
    let content =
        std::fs::read_to_string(crd_fixture("lageos1_v1.crd")).expect("read lageos1_v1.crd");
    parse_crd(&content).expect("parse lageos1_v1.crd")
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
    use chrono::Datelike;
    let f = load();
    let d = f.session_date.unwrap().try_to_chrono().unwrap();
    assert_eq!(d.year(), 2024, "session year");
    assert_eq!(d.month(), 1, "session month");
    assert_eq!(d.day(), 1, "session day");
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

// ── NormalPoint tests (typed API) ─────────────────────────────────────────────

#[test]
fn crd_normal_point_count_matches_ranges() {
    let f = load();
    assert_eq!(
        f.normal_points.len(),
        f.ranges.len(),
        "NormalPoint count should equal range count for type-11 files"
    );
}

#[test]
fn crd_normal_point_range_m_positive() {
    for (i, np) in load().normal_points.iter().enumerate() {
        assert!(
            np.range_m.value() > 0.0,
            "np[{i}].range_m not positive: {}",
            np.range_m.value()
        );
    }
}

#[test]
fn crd_normal_point_range_m_lageos_plausible() {
    // LAGEOS-1 one-way slant range during pass: 7 000–12 300 km
    for (i, np) in load().normal_points.iter().enumerate() {
        let r_km = np.range_m.value() / 1_000.0;
        assert!(r_km > 6_000.0, "np[{i}] range {r_km:.0} km below floor");
        assert!(r_km < 13_000.0, "np[{i}] range {r_km:.0} km above ceiling");
    }
}

#[test]
fn crd_normal_point_epoch_computed() {
    let f = load();
    for np in &f.normal_points {
        assert!(np.epoch.is_some(), "all NPs should have computed epoch");
    }
}

#[test]
fn crd_normal_point_epoch_hour_8() {
    use chrono::Timelike;
    // First NP has SOD = 28 800 s = 08:00:00
    let f = load();
    let dt = f.normal_points[0].epoch.unwrap().try_to_chrono().unwrap();
    assert_eq!(dt.hour(), 8, "SOD 28800 should give hour 8");
    assert_eq!(dt.minute(), 0);
    assert_eq!(dt.second(), 0);
}

// ── Station/Target typed struct tests ────────────────────────────────────────

#[test]
fn crd_station_struct_matches_flat() {
    let f = load();
    assert_eq!(f.station.name, f.station_name);
    assert_eq!(f.station.cdp_pad, f.station_cdp_pad);
}

#[test]
fn crd_target_struct_matches_flat() {
    let f = load();
    assert_eq!(f.target.name, f.satellite_name);
    assert_eq!(f.target.sic, f.satellite_sic);
    assert_eq!(f.target.norad, f.satellite_norad);
}

// ── CRD v2 fixture tests ──────────────────────────────────────────────────────

#[test]
fn crd_v2_format_version() {
    assert_eq!(load_v2().format_version, "2");
}

#[test]
fn crd_v2_normal_point_count() {
    assert_eq!(
        load_v2().normal_points.len(),
        5,
        "expected 5 NPs in v2 fixture"
    );
}

#[test]
fn crd_v2_num_raws_parsed() {
    let f = load_v2();
    assert!(
        f.normal_points.iter().all(|np| np.num_raws.is_some()),
        "all v2 NPs should have num_raws"
    );
}

#[test]
fn crd_v2_bin_rms_parsed() {
    let f = load_v2();
    assert!(
        f.normal_points.iter().all(|np| np.bin_rms_m.is_some()),
        "all v2 NPs should have bin_rms_m"
    );
}

#[test]
fn crd_v2_sods_strictly_increasing() {
    let sods: Vec<f64> = load_v2()
        .normal_points
        .iter()
        .map(|np| np.seconds_of_day.value())
        .collect();
    assert_strictly_increasing(&sods, "v2 NP seconds-of-day");
}

#[test]
fn crd_v2_tof_lageos_range() {
    // LAGEOS-1 two-way TOF ≈ 0.0816 s at 12 270 km geocentric
    for (i, np) in load_v2().normal_points.iter().enumerate() {
        let tof = np.time_of_flight.value();
        assert!(tof > 0.080, "np[{i}] tof {tof:.6} s below LAGEOS floor");
        assert!(tof < 0.090, "np[{i}] tof {tof:.6} s above LAGEOS ceiling");
    }
}

// ── CRD v1 fixture tests ──────────────────────────────────────────────────────

#[test]
fn crd_v1_format_version() {
    assert_eq!(load_v1().format_version, "1");
}

#[test]
fn crd_v1_normal_point_count() {
    assert_eq!(
        load_v1().normal_points.len(),
        3,
        "expected 3 NPs in v1 fixture"
    );
}

#[test]
fn crd_v1_station_hers() {
    assert_eq!(load_v1().station_name, "HERS");
}

// ── malformed-input tests ────────────────────────────────────────────────────

#[test]
fn crd_malformed_range_missing_sod() {
    // "11" record with no following tokens → missing SOD error.
    let bad = "H2 TEST 1234 1 01 0\nH3 sat 0 0\nH4 0 2024 1 1\n11\n";
    let err = parse_crd(bad).expect_err("missing SOD should fail");
    let msg = format!("{err}");
    assert!(
        msg.to_ascii_lowercase().contains("sod")
            || msg.to_ascii_lowercase().contains("seconds-of-day"),
        "expected SOD in error message, got: {msg}"
    );
}

#[test]
fn crd_malformed_range_missing_tof() {
    // "11" record with SOD but no TOF → missing TOF error.
    let bad = "H2 TEST 1234 1 01 0\nH3 sat 0 0\nH4 0 2024 1 1\n11 12345.0\n";
    let err = parse_crd(bad).expect_err("missing TOF should fail");
    let msg = format!("{err}");
    assert!(
        msg.to_ascii_lowercase().contains("tof")
            || msg.to_ascii_lowercase().contains("time-of-flight"),
        "expected TOF in error message, got: {msg}"
    );
}

#[test]
fn crd_malformed_non_numeric_sod() {
    // "11" record where SOD is not a number → parse error.
    let bad = "H2 TEST 1234 1 01 0\nH3 sat 0 0\nH4 0 2024 1 1\n11 NOTANUMBER 0.05\n";
    let err = parse_crd(bad).expect_err("non-numeric SOD should fail");
    assert!(
        matches!(err, PodIoError::Format(_) | PodIoError::Located { .. }),
        "expected a format or located error, got {err:?}"
    );
}

#[test]
fn crd_unknown_tags_silently_ignored() {
    // Tags not in the handled set (H2, H3, H4, C0, 10, 11) are skipped.
    let txt = "H1 CRD 2 2024 01 01 00\nXX unknown tag\nH2 GRAZ 7839 1 01 0\nH8\n";
    let f = parse_crd(txt).expect("unknown tag should not cause error");
    assert_eq!(f.station_name, "GRAZ");
}

#[test]
fn crd_permissive_recovers_from_bad_record() {
    let txt = "H2 GRAZ 7839 1 1 0\n11\n11 28800.0 0.08 std 0 0 1 0\nH8\n";
    let f = parse_crd_with_mode(txt, ParseMode::Permissive).expect("permissive parse");
    assert_eq!(
        f.normal_points.len(),
        1,
        "bad record skipped, good one kept"
    );
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
