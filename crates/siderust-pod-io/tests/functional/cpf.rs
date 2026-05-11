//! Functional tests for the CPF orbit prediction format.
//!
//! Fixture: `test-data/tiny/sample.cpf` — LAGEOS-1 prediction,
//! 4 position records at 60-second spacing on MJD 60310.

use super::common::{assert_approx, assert_strictly_increasing, tiny};
use siderust_pod_io::cpf::parse_cpf;
use siderust_pod_io::PodIoError;

// ── helpers ──────────────────────────────────────────────────────────────────

fn load() -> siderust_pod_io::cpf::CpfFile {
    let content = std::fs::read_to_string(tiny("sample.cpf")).expect("read sample.cpf");
    parse_cpf(&content).expect("parse sample.cpf")
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
    match err {
        PodIoError::Format(msg) => {
            assert!(msg.contains("MJD"), "expected MJD in error, got: {msg}");
        }
        other => panic!("expected PodIoError::Format, got {other:?}"),
    }
}

#[test]
fn cpf_malformed_missing_xyz() {
    // "10" record with valid MJD and SOD but no X coordinate.
    let bad = "H1 CPF 2 TST 2024 01 01 00 1\nH2 tgt 0 0\n10 1 60310 0.0 0\n";
    let err = parse_cpf(bad).expect_err("missing X should fail");
    assert!(matches!(err, PodIoError::Format(_)));
}

#[test]
fn cpf_unknown_tags_silently_ignored() {
    let txt = "H1 CPF 2 CNES 2024 01 01 00 1\nXX garbage\nH2 sat 0 0\n99\n";
    let f = parse_cpf(txt).expect("unknown tag should not error");
    assert_eq!(f.source, "CNES");
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
