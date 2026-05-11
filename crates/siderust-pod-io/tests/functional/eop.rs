//! Functional tests for the IERS C04 Earth-orientation parameter format.
//!
//! Fixture: `test-data/tiny/sample.eop` — 7 daily records (MJD 60310–60316).

use super::common::{assert_approx, assert_strictly_increasing, tiny};
use qtty::Day;
use siderust_pod_io::eop::{interpolate, read_eop_c04};
use tempoch::{ModifiedJulianDate, UTC};

// ── helpers ──────────────────────────────────────────────────────────────────

fn load() -> Vec<siderust_pod_io::eop::EopRecord> {
    let content = std::fs::read_to_string(tiny("sample.eop")).expect("read sample.eop");
    read_eop_c04(content.as_bytes()).expect("parse sample.eop")
}

fn mjd(v: f64) -> ModifiedJulianDate<UTC> {
    ModifiedJulianDate::<UTC>::try_new(Day::new(v)).expect("valid MJD")
}

// ── parse tests ──────────────────────────────────────────────────────────────

#[test]
fn eop_record_count() {
    assert_eq!(load().len(), 7, "expected 7 EOP records");
}

#[test]
fn eop_first_mjd() {
    assert_approx(load()[0].mjd.raw().value(), 60_310.0, 1e-9, "first MJD");
}

#[test]
fn eop_last_mjd() {
    let r = load();
    assert_approx(r[r.len() - 1].mjd.raw().value(), 60_316.0, 1e-9, "last MJD");
}

#[test]
fn eop_first_record_fields() {
    let r0 = load()[0];
    assert_approx(r0.x.value(), 0.123_456, 1e-9, "record[0].x [arcsec]");
    assert_approx(r0.y.value(), 0.234_567, 1e-9, "record[0].y [arcsec]");
    assert_approx(r0.ut1_utc.value(), 0.012_345, 1e-9, "record[0].ut1_utc [s]");
    assert_approx(r0.lod.value(), 0.001_234, 1e-9, "record[0].lod [s]");
    assert_approx(r0.dpsi.value(), 0.000_123, 1e-9, "record[0].dpsi [arcsec]");
    assert_approx(r0.deps.value(), 0.000_234, 1e-9, "record[0].deps [arcsec]");
}

#[test]
fn eop_chronological_order() {
    let mjds: Vec<f64> = load().iter().map(|r| r.mjd.raw().value()).collect();
    assert_strictly_increasing(&mjds, "EOP MJD sequence");
}

// ── interpolation tests ──────────────────────────────────────────────────────

#[test]
fn eop_interpolate_exact_node() {
    // Requesting an exact tabulated MJD should return that record's values.
    let recs = load();
    let result = interpolate(&recs, mjd(60_310.0)).expect("interpolate should succeed");
    assert_approx(result.x.value(), 0.123_456, 1e-9, "exact node x");
    assert_approx(
        result.ut1_utc.value(),
        0.012_345,
        1e-9,
        "exact node ut1_utc",
    );
}

#[test]
fn eop_interpolate_midpoint() {
    // Linear interpolation at the midpoint between day 0 and day 1.
    let recs = load();
    let result = interpolate(&recs, mjd(60_310.5)).expect("midpoint interpolate");
    let expected_x = (0.123_456 + 0.124_000) / 2.0;
    assert_approx(result.x.value(), expected_x, 1e-9, "midpoint x");
    let expected_ut1 = (0.012_345 + 0.012_200) / 2.0;
    assert_approx(
        result.ut1_utc.value(),
        expected_ut1,
        1e-9,
        "midpoint ut1_utc",
    );
}

#[test]
fn eop_interpolate_clamps_before_first() {
    let recs = load();
    let result = interpolate(&recs, mjd(60_000.0)).expect("clamp before first");
    assert_approx(
        result.x.value(),
        recs[0].x.value(),
        1e-12,
        "clamped to first record",
    );
}

#[test]
fn eop_interpolate_clamps_after_last() {
    let recs = load();
    let last = recs[recs.len() - 1];
    let result = interpolate(&recs, mjd(70_000.0)).expect("clamp after last");
    assert_approx(result.x.value(), last.x.value(), 1e-12, "clamped to last");
}

#[test]
fn eop_interpolate_empty_returns_none() {
    assert!(
        interpolate(&[], mjd(60_310.0)).is_none(),
        "empty table must return None"
    );
}

// ── malformed-input tests ────────────────────────────────────────────────────

#[test]
fn eop_lines_with_too_few_fields_are_skipped() {
    // Lines with fewer than 10 fields are silently skipped (graceful degradation).
    let txt = "\
# comment\n\
2024  1  1  60310\n\
2024  1  1  60310    0.1    0.2    0.3    0.4    0.5    0.6\n";
    let r = read_eop_c04(txt.as_bytes()).expect("parse should succeed");
    assert_eq!(r.len(), 1, "only the complete line should be parsed");
}

#[test]
fn eop_comment_lines_skipped() {
    let txt = "# full comment line\n2024  1  1  60310    0.1    0.2    0.3    0.4    0.5    0.6\n";
    let r = read_eop_c04(txt.as_bytes()).expect("parse");
    assert_eq!(r.len(), 1);
}

#[test]
fn eop_non_integer_year_field_skipped() {
    // A line whose first column doesn't parse as an integer is silently ignored.
    let txt = "XXXX  1  1  60310    0.1    0.2    0.3    0.4    0.5    0.6\n";
    let r = read_eop_c04(txt.as_bytes()).expect("non-integer year should not error");
    assert_eq!(r.len(), 0);
}

// ── heavy/ignored tests ──────────────────────────────────────────────────────

/// Place `eopc04.1962-now.1` (IERS C04 combined series from
/// https://hpiers.obspm.fr/iers/eop/eopc04/) at
/// `test-data/official/eopc04.eop` and remove `#[ignore]`.
#[test]
#[ignore]
fn eop_official_iers_c04() {
    let path = super::common::official("eopc04.eop");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("place IERS C04 file at {}", path.display()));
    let recs = read_eop_c04(content.as_bytes()).expect("parse official EOP");
    assert!(recs.len() > 1_000, "official C04 should have >1000 records");
    // Verify chronological ordering across the whole series.
    let mjds: Vec<f64> = recs.iter().map(|r| r.mjd.raw().value()).collect();
    assert_strictly_increasing(&mjds, "official EOP MJD sequence");
}
