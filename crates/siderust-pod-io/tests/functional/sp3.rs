//! Functional tests for the SP3 precise orbit format.
//!
//! Fixture: `test-data/tiny/sample.sp3` — synthetic GPS MEO constellation,
//! 3 epochs (15-minute spacing), 3 satellites (G01, G02, G03).

use super::common::{assert_approx, assert_strictly_increasing, tiny};
use siderust_pod_io::sp3::{read_sp3, write_sp3, Sp3Error};

// ── helpers ─────────────────────────────────────────────────────────────────

fn load() -> siderust_pod_io::sp3::Sp3Record {
    let content = std::fs::read_to_string(tiny("sample.sp3")).expect("read sample.sp3");
    read_sp3(content.as_bytes()).expect("parse sample.sp3")
}

// ── parse tests ──────────────────────────────────────────────────────────────

#[test]
fn sp3_epoch_count() {
    assert_eq!(load().epochs.len(), 3, "expected 3 epochs in fixture");
}

#[test]
fn sp3_satellite_count_per_epoch() {
    let rec = load();
    for (i, epoch) in rec.epochs.iter().enumerate() {
        assert_eq!(
            epoch.positions.len(),
            3,
            "epoch {i}: expected 3 satellite records"
        );
    }
}

#[test]
fn sp3_satellite_ids() {
    let epoch = &load().epochs[0];
    let ids: Vec<&str> = epoch.positions.iter().map(|p| p.sat_id.as_str()).collect();
    assert!(ids.contains(&"G01"), "G01 missing from epoch 0");
    assert!(ids.contains(&"G02"), "G02 missing from epoch 0");
    assert!(ids.contains(&"G03"), "G03 missing from epoch 0");
}

#[test]
fn sp3_position_values_epoch0_g01() {
    let epoch = &load().epochs[0];
    let g01 = epoch
        .positions
        .iter()
        .find(|p| p.sat_id == "G01")
        .expect("G01 not found");
    assert_approx(g01.x.value(), 15_000.0, 1e-6, "G01.x [km]");
    assert_approx(g01.y.value(), 20_000.0, 1e-6, "G01.y [km]");
    assert_approx(g01.z.value(), -5_000.0, 1e-6, "G01.z [km]");
    assert_approx(g01.clock.value(), 0.000_123, 1e-9, "G01.clock [µs]");
}

#[test]
fn sp3_timestamps_strictly_increasing() {
    let rec = load();
    let ts: Vec<f64> = rec
        .epochs
        .iter()
        .map(|e| {
            e.time
                .try_to_chrono()
                .expect("epoch to chrono")
                .timestamp_nanos_opt()
                .expect("nanos") as f64
        })
        .collect();
    assert_strictly_increasing(&ts, "SP3 epoch timestamps");
}

#[test]
fn sp3_position_norm_plausible_gps_meo() {
    // GPS MEO satellites occupy ~20 200 – 26 600 km geocentric orbits.
    let rec = load();
    for epoch in &rec.epochs {
        for pos in &epoch.positions {
            let norm =
                (pos.x.value().powi(2) + pos.y.value().powi(2) + pos.z.value().powi(2)).sqrt();
            assert!(
                norm > 20_000.0,
                "sat={}: norm={norm:.0} km below GPS MEO floor",
                pos.sat_id
            );
            assert!(
                norm < 30_000.0,
                "sat={}: norm={norm:.0} km above GPS MEO ceiling",
                pos.sat_id
            );
        }
    }
}

// ── round-trip test ──────────────────────────────────────────────────────────

#[test]
fn sp3_write_reparse_round_trip() {
    let rec = load();
    let mut buf = Vec::new();
    write_sp3(&mut buf, &rec).expect("write_sp3");
    let rec2 = read_sp3(buf.as_slice()).expect("re-parse sp3");

    assert_eq!(
        rec.epochs.len(),
        rec2.epochs.len(),
        "epoch count changed after round-trip"
    );
    for (i, (e1, e2)) in rec.epochs.iter().zip(rec2.epochs.iter()).enumerate() {
        assert_eq!(
            e1.positions.len(),
            e2.positions.len(),
            "epoch {i}: satellite count changed"
        );
        for (p1, p2) in e1.positions.iter().zip(e2.positions.iter()) {
            assert_eq!(p1.sat_id, p2.sat_id, "epoch {i}: sat_id mismatch");
            let label = format!("epoch{i}/{}", p1.sat_id);
            assert_approx(p1.x.value(), p2.x.value(), 1e-4, &format!("{label}.x"));
            assert_approx(p1.y.value(), p2.y.value(), 1e-4, &format!("{label}.y"));
            assert_approx(p1.z.value(), p2.z.value(), 1e-4, &format!("{label}.z"));
            assert_approx(
                p1.clock.value(),
                p2.clock.value(),
                1e-6,
                &format!("{label}.clock"),
            );
        }
    }
}

#[test]
fn sp3_round_trip_header_line_count_preserved() {
    let rec = load();
    let mut buf = Vec::new();
    write_sp3(&mut buf, &rec).expect("write_sp3");
    let rec2 = read_sp3(buf.as_slice()).expect("re-parse sp3");
    assert_eq!(
        rec.header.len(),
        rec2.header.len(),
        "header line count changed after round-trip"
    );
}

// ── malformed-input tests ────────────────────────────────────────────────────

#[test]
fn sp3_malformed_truncated_epoch_line() {
    // Epoch line with only 4 fields (needs 6: year month day hour minute second).
    let bad = "*  2024  1  1  0\nEOF\n";
    let err = read_sp3(bad.as_bytes()).expect_err("truncated epoch should fail");
    assert!(
        matches!(err, Sp3Error::Record { .. }),
        "expected Sp3Error::Record, got {err:?}"
    );
}

#[test]
fn sp3_malformed_p_record_too_few_fields() {
    // P record with only 2 numeric fields (needs 4: x, y, z, clock).
    let bad = "*  2024  1  1  0  0  0.0\nPG01  1000.0  2000.0\nEOF\n";
    let err = read_sp3(bad.as_bytes()).expect_err("P record too short should fail");
    assert!(matches!(err, Sp3Error::Record { .. }));
}

#[test]
fn sp3_malformed_non_numeric_position() {
    let bad = "*  2024  1  1  0  0  0.0\nPG01  NOTANUMBER  2000.0  3000.0  0.0\nEOF\n";
    let err = read_sp3(bad.as_bytes()).expect_err("non-numeric position should fail");
    assert!(matches!(err, Sp3Error::Record { .. }));
}

#[test]
fn sp3_malformed_invalid_month() {
    // Month 13 is invalid.
    let bad = "*  2024 13  1  0  0  0.0\nEOF\n";
    let err = read_sp3(bad.as_bytes()).expect_err("invalid month should fail");
    assert!(matches!(err, Sp3Error::Record { .. }));
}

// ── heavy/ignored tests (require official IGS data) ─────────────────────────

/// Place an IGS precise orbit product (e.g. `IGS0OPSFIN_20240010000_01D_15M_ORB.SP3`)
/// from https://cddis.nasa.gov/archive/gnss/products/ in `test-data/official/` and
/// remove the `#[ignore]` to run.
#[test]
#[ignore]
fn sp3_official_igs_large_file() {
    let path = official("igs_orbit.sp3");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("place official SP3 at {}", path.display()));
    let rec = read_sp3(content.as_bytes()).expect("parse official SP3");
    assert!(!rec.epochs.is_empty(), "official SP3 must have epochs");
    assert!(
        rec.epochs[0].positions.len() > 10,
        "official SP3 must have >10 satellites"
    );
}

fn official(name: &str) -> std::path::PathBuf {
    super::common::official(name)
}
