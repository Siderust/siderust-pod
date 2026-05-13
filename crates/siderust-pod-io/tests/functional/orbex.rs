//! Functional tests for the ORBEX orbit/clock reader.
use siderust_pod_io::orbex::read_orbex;
use siderust_pod_io::ParseMode;

const SAMPLE: &[u8] = include_bytes!("../../test-data/tiny/sample.orbex");

#[test]
fn orbex_parses_ok() {
    read_orbex(SAMPLE, ParseMode::Permissive).unwrap();
}

#[test]
fn orbex_orbit_entry_count() {
    let prod = read_orbex(SAMPLE, ParseMode::Permissive).unwrap();
    assert_eq!(prod.orbits.len(), 1);
}

#[test]
fn orbex_clock_entry_count() {
    let prod = read_orbex(SAMPLE, ParseMode::Permissive).unwrap();
    assert_eq!(prod.clocks.len(), 1);
}

#[test]
fn orbex_orbit_sat_id() {
    let prod = read_orbex(SAMPLE, ParseMode::Permissive).unwrap();
    assert_eq!(prod.orbits[0].sat, "G01");
}
