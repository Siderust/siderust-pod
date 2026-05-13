//! Functional tests for the CPF orbit prediction format.
//!
//! Fixtures:
//! - `test-data/tiny/sample.cpf` — LAGEOS-1 prediction,
//!   4 position records at 60-second spacing on MJD 60310.
//! - `test-data/cpf/lageos1_v2.cpf` — CPF v2 fixture, 5 positions, 60 s step.
//! - `test-data/cpf/lageos1_v1.cpf` — CPF v1 fixture, 3 positions, 120 s step.

use super::common::{assert_approx, assert_strictly_increasing, cpf_fixture, tiny};
use siderust_pod_io::cpf::{parse_cpf, parse_cpf_with_mode, write_cpf};
use siderust_pod_io::{ParseMode, PodIoError};

// ── helpers ──────────────────────────────────────────────────────────────────

fn load() -> siderust_pod_io::cpf::CpfFile {
    let content = std::fs::read_to_string(tiny("sample.cpf")).expect("read sample.cpf");
    parse_cpf(&content).expect("parse sample.cpf")
}

fn load_v2() -> siderust_pod_io::cpf::CpfFile {
    let content =
        std::fs::read_to_string(cpf_fixture("lageos1_v2.cpf")).expect("read lageos1_v2.cpf");
    parse_cpf(&content).expect("parse lageos1_v2.cpf")
}

fn load_v1() -> siderust_pod_io::cpf::CpfFile {
    let content =
        std::fs::read_to_string(cpf_fixture("lageos1_v1.cpf")).expect("read lageos1_v1.cpf");
    parse_cpf(&content).expect("parse lageos1_v1.cpf")
}

// ── header tests ─────────────────────────────────────────────────────────────

#[test]
fn cpf_version() {
    assert_eq!(load().version, "2", "expected CPF version 2");
}

#[test]
fn cpf_source_agency() {
    assert_eq!(load().source, "CNES");
}

#[test]
fn cpf_target_name() {
    assert_eq!(load().target_name, "lageos1");
}

#[test]
fn cpf_reference_frame() {
    assert_eq!(load().reference_frame, "ITRF2014");
}

// ── position record tests ──────────────────────────────────────────────────

#[test]
fn cpf_position_count() {
    assert_eq!(load().positions.len(), 4, "expected 4 position records");
}

#[test]
fn cpf_first_position_mjd() {
    assert_approx(
        load().positions[0].mjd.raw().value(),
        60_310.0,
        1e-9,
        "first position MJD",
    );
}

#[test]
fn cpf_first_position_sod() {
    assert_approx(
        load().positions[0].seconds_of_day.value(),
        0.0,
        1e-9,
        "first position SOD [s]",
    );
}

#[test]
fn cpf_first_position_xyz() {
    let p = &load().positions[0];
    assert_approx(p.position_m[0].value(), 6_000_000.0, 1e-3, "pos[0].x [m]");
    assert_approx(p.position_m[1].value(), 9_000_000.0, 1e-3, "pos[0].y [m]");
    assert_approx(p.position_m[2].value(), 7_000_000.0, 1e-3, "pos[0].z [m]");
}

#[test]
fn cpf_epochs_strictly_increasing() {
    // SOD within the same MJD day should be strictly increasing.
    let sods: Vec<f64> = load()
        .positions
        .iter()
        .map(|p| p.seconds_of_day.value())
        .collect();
    assert_strictly_increasing(&sods, "CPF seconds-of-day");
}

#[test]
fn cpf_position_vectors_finite() {
    for (i, pos) in load().positions.iter().enumerate() {
        for (j, coord) in pos.position_m.iter().enumerate() {
            assert!(
                coord.value().is_finite(),
                "pos[{i}][{j}] = {} is not finite",
                coord.value()
            );
        }
    }
}

#[test]
fn cpf_position_norm_plausible_lageos() {
    // LAGEOS-1 orbits at ~5858 km altitude (geocentric ~12 270 km).
    // Our synthetic fixture uses ~12 884 km.
    for (i, pos) in load().positions.iter().enumerate() {
        let norm_km = pos
            .position_m
            .iter()
            .map(|c| c.value().powi(2))
            .sum::<f64>()
            .sqrt()
            / 1_000.0;
        assert!(
            norm_km > 10_000.0,
            "pos[{i}] norm {norm_km:.0} km below LAGEOS-class floor"
        );
        assert!(
            norm_km < 20_000.0,
            "pos[{i}] norm {norm_km:.0} km above LAGEOS-class ceiling"
        );
    }
}

// ── malformed-input tests ────────────────────────────────────────────────────

#[test]
fn cpf_malformed_missing_mjd() {
    // "10" record with direction token only → missing MJD error.
    let bad = "H1 CPF 2 TST 2024 01 01 00 1\nH2 tgt 0 0\n10 1\n";
    let err = parse_cpf(bad).expect_err("missing MJD should fail");
    let msg = format!("{err}");
    assert!(
        msg.to_ascii_uppercase().contains("MJD"),
        "expected MJD in error, got: {msg}"
    );
}

#[test]
fn cpf_malformed_missing_xyz() {
    // "10" record with valid MJD and SOD but no X coordinate.
    let bad = "H1 CPF 2 TST 2024 01 01 00 1\nH2 tgt 0 0\n10 1 60310 0.0 0\n";
    let err = parse_cpf(bad).expect_err("missing X should fail");
    assert!(
        matches!(err, PodIoError::Format(_) | PodIoError::Located { .. }),
        "expected a format or located error, got {err:?}"
    );
}

#[test]
fn cpf_unknown_tags_silently_ignored() {
    let txt = "H1 CPF 2 CNES 2024 01 01 00 1\nXX garbage\nH2 sat 0 0\n99\n";
    let f = parse_cpf(txt).expect("unknown tag should not error");
    assert_eq!(f.source, "CNES");
}

#[test]
fn cpf_permissive_recovers_from_bad_record() {
    let txt =
        "H1 CPF 2 TST 2024 01 01 00 1\nH2 tgt 0 0\n10\n10 1 60310 0.0 0 6000000 9000000 7000000\n99\n";
    let f = parse_cpf_with_mode(txt, ParseMode::Permissive).expect("permissive parse");
    assert_eq!(f.positions.len(), 1, "bad record skipped, good one kept");
}

// ── CPF v2 fixture tests ──────────────────────────────────────────────────────

#[test]
fn cpf_v2_version() {
    assert_eq!(load_v2().version, "2");
}

#[test]
fn cpf_v2_position_count() {
    assert_eq!(
        load_v2().positions.len(),
        5,
        "expected 5 positions in v2 fixture"
    );
}

#[test]
fn cpf_v2_reference_frame() {
    let f = load_v2();
    assert!(
        f.reference_frame.contains("ITRF"),
        "expected ITRF frame, got '{}'",
        f.reference_frame
    );
}

#[test]
fn cpf_v2_sods_strictly_increasing() {
    let sods: Vec<f64> = load_v2()
        .positions
        .iter()
        .map(|p| p.seconds_of_day.value())
        .collect();
    assert_strictly_increasing(&sods, "v2 CPF seconds-of-day");
}

#[test]
fn cpf_v2_ephemeris_count() {
    let f = load_v2();
    assert_eq!(
        f.ephemeris.entries.len(),
        5,
        "ephemeris and positions should both have 5 entries"
    );
}

#[test]
fn cpf_v2_ephemeris_positions_km_range() {
    // LAGEOS-1 geocentric ~12 270 km; fixture values match
    for (i, entry) in load_v2().ephemeris.iter().enumerate() {
        let x = entry.position.x().value();
        let y = entry.position.y().value();
        let z = entry.position.z().value();
        let norm_km = (x * x + y * y + z * z).sqrt();
        assert!(
            norm_km > 10_000.0,
            "entry[{i}] norm {norm_km:.1} km below floor"
        );
        assert!(
            norm_km < 20_000.0,
            "entry[{i}] norm {norm_km:.1} km above ceiling"
        );
    }
}

#[test]
fn cpf_v2_epoch_day_2024_01_01() {
    use chrono::Datelike;
    // MJD 60310 = 2024-01-01
    let f = load_v2();
    let dt = f.ephemeris[0].epoch.try_to_chrono().unwrap();
    assert_eq!(dt.year(), 2024);
    assert_eq!(dt.month(), 1);
    assert_eq!(dt.day(), 1);
}

#[test]
fn cpf_v2_time_step_s() {
    assert_approx(load_v2().time_step_s, 60.0, 1e-9, "v2 time_step_s");
}

// ── CPF v1 fixture tests ──────────────────────────────────────────────────────

#[test]
fn cpf_v1_version() {
    assert_eq!(load_v1().version, "1");
}

#[test]
fn cpf_v1_position_count() {
    assert_eq!(
        load_v1().positions.len(),
        3,
        "expected 3 positions in v1 fixture"
    );
}

#[test]
fn cpf_v1_reference_frame_itrf() {
    let f = load_v1();
    assert!(
        f.reference_frame.contains("ITRF"),
        "expected ITRF frame, got '{}'",
        f.reference_frame
    );
}

// ── Round-trip test ───────────────────────────────────────────────────────────

#[test]
fn cpf_v2_round_trip_write_reparse() {
    let original = load_v2();

    // Serialize to a string buffer.
    let mut buf = Vec::<u8>::new();
    write_cpf(&original, &mut buf).expect("write_cpf should succeed");
    let serialised = String::from_utf8(buf).expect("output should be valid UTF-8");

    // Re-parse the serialised text.
    let reparsed = parse_cpf(&serialised).expect("re-parse of written CPF should succeed");

    assert_eq!(
        reparsed.positions.len(),
        original.positions.len(),
        "round-trip position count mismatch"
    );
    assert_eq!(
        reparsed.source, original.source,
        "round-trip source mismatch"
    );

    // Check each position numerically.
    for (i, (orig, rt)) in original
        .positions
        .iter()
        .zip(reparsed.positions.iter())
        .enumerate()
    {
        assert_approx(
            rt.seconds_of_day.value(),
            orig.seconds_of_day.value(),
            1e-3,
            &format!("round-trip pos[{i}].sod"),
        );
        for j in 0..3 {
            assert_approx(
                rt.position_m[j].value(),
                orig.position_m[j].value(),
                1.0, // 1 m precision (written as whole metres)
                &format!("round-trip pos[{i}][{j}]"),
            );
        }
    }
}

// ── heavy/ignored tests ──────────────────────────────────────────────────────

/// Place an ILRS CPF file (from https://cddis.nasa.gov/archive/slr/cpf_predicts/)
/// at `test-data/official/ilrs.cpf` and remove `#[ignore]`.
#[test]
#[ignore]
fn cpf_official_ilrs_file() {
    let path = super::common::official("ilrs.cpf");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("place ILRS CPF file at {}", path.display()));
    let f = parse_cpf(&content).expect("parse official CPF");
    assert!(!f.positions.is_empty(), "official CPF must have positions");
    let sods: Vec<f64> = f
        .positions
        .iter()
        .map(|p| p.seconds_of_day.value())
        .collect();
    assert_strictly_increasing(&sods, "official CPF SOD sequence");
}
