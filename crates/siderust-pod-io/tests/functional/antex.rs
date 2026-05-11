//! Functional tests for the ANTEX antenna phase-centre offset format.
//!
//! Fixture: `test-data/tiny/sample.atx` — 2 antennas (BLOCK IIA, TRM59800.00),
//! 2 GPS frequency blocks each (G01, G02).

use super::common::{assert_approx, tiny};
use siderust_pod_io::antex::read_antex;

// ── helpers ──────────────────────────────────────────────────────────────────

fn load() -> siderust_pod_io::antex::AntexCatalog {
    let content = std::fs::read_to_string(tiny("sample.atx")).expect("read sample.atx");
    read_antex(content.as_bytes()).expect("parse sample.atx")
}

// ── parse tests ──────────────────────────────────────────────────────────────

#[test]
fn antex_antenna_count() {
    assert_eq!(load().len(), 2, "expected 2 antenna blocks");
}

#[test]
fn antex_block_iia_present() {
    assert!(
        load().contains_key("BLOCK IIA           001"),
        "BLOCK IIA antenna not found in catalog"
    );
}

#[test]
fn antex_trm_receiver_present() {
    assert!(
        load().contains_key("TRM59800.00     NONE"),
        "TRM59800.00 antenna not found in catalog"
    );
}

#[test]
fn antex_block_iia_g01_pco_values() {
    let cat = load();
    let ant = cat.get("BLOCK IIA           001").expect("BLOCK IIA");
    let pco = ant.get("G01").expect("G01 frequency block in BLOCK IIA");
    // Values are in millimetres as stored in the file; ANTEX NEU maps to x/y/z.
    assert_approx(pco.x().value(), 279.00, 1e-6, "BLOCK IIA G01 north [mm]");
    assert_approx(pco.y().value(), -2.00, 1e-6, "BLOCK IIA G01 east [mm]");
    assert_approx(pco.z().value(), 1_023.40, 1e-6, "BLOCK IIA G01 up [mm]");
}

#[test]
fn antex_block_iia_g02_pco_values() {
    let cat = load();
    let ant = cat.get("BLOCK IIA           001").expect("BLOCK IIA");
    let pco = ant.get("G02").expect("G02 frequency block in BLOCK IIA");
    assert_approx(pco.x().value(), 279.00, 1e-6, "BLOCK IIA G02 north [mm]");
    assert_approx(pco.y().value(), -2.00, 1e-6, "BLOCK IIA G02 east [mm]");
    assert_approx(pco.z().value(), 856.10, 1e-6, "BLOCK IIA G02 up [mm]");
}

#[test]
fn antex_trm_receiver_g01_pco_values() {
    let cat = load();
    let ant = cat.get("TRM59800.00     NONE").expect("TRM59800.00");
    let pco = ant.get("G01").expect("G01 block in TRM59800.00");
    assert_approx(pco.x().value(), 0.00, 1e-6, "TRM G01 north [mm]");
    assert_approx(pco.y().value(), 0.00, 1e-6, "TRM G01 east [mm]");
    assert_approx(pco.z().value(), 68.90, 1e-6, "TRM G01 up [mm]");
}

#[test]
fn antex_frequency_count_per_antenna() {
    let cat = load();
    for (name, freqs) in &cat {
        assert_eq!(
            freqs.len(),
            2,
            "antenna '{name}' should have exactly 2 frequency blocks"
        );
    }
}

#[test]
fn antex_unit_values_are_millimetres() {
    // Confirm that PCO values are stored as-read (mm), not silently converted
    // to metres. The BLOCK IIA G01 up-offset is 1023.4 mm ≈ 1.0 m; if the
    // crate had converted to metres the value would be ~1.0, not ~1023.4.
    let cat = load();
    let pco = cat["BLOCK IIA           001"]["G01"];
    assert!(
        pco.z().value() > 100.0,
        "up PCO {:.3} mm looks like it was converted to metres",
        pco.z().value()
    );
}

// ── malformed-input tests ────────────────────────────────────────────────────

#[test]
fn antex_incomplete_antenna_block_dropped() {
    // START OF ANTENNA without END OF ANTENNA → silently dropped.
    let bad = "\
                                                            START OF ANTENNA\n\
TEST-ANT           SN001                                    TYPE / SERIAL NO\n\
   G01                                                      START OF FREQUENCY\n\
        0.00     0.00    50.00                              NORTH / EAST / UP\n\
                                                            END OF FREQUENCY\n\
";
    // No END OF ANTENNA → catalog is empty.
    let cat = read_antex(bad.as_bytes()).expect("incomplete block should not error");
    assert_eq!(
        cat.len(),
        0,
        "incomplete antenna block (no END OF ANTENNA) must not appear in catalog"
    );
}

#[test]
fn antex_empty_input_produces_empty_catalog() {
    let cat = read_antex("".as_bytes()).expect("empty input should succeed");
    assert_eq!(cat.len(), 0);
}

// ── heavy/ignored tests ──────────────────────────────────────────────────────

/// Place `igs20.atx` (from https://files.igs.org/pub/station/general/igs20.atx)
/// at `test-data/official/igs20.atx` and remove `#[ignore]`.
#[test]
#[ignore]
fn antex_official_igs20() {
    let path = super::common::official("igs20.atx");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("place igs20.atx at {}", path.display()));
    let cat = read_antex(content.as_bytes()).expect("parse igs20.atx");
    assert!(
        cat.len() > 100,
        "igs20.atx should have many antenna entries"
    );
}
