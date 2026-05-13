//! Functional tests for the SINEX station-coordinate reader.
use siderust_pod_io::sinex::read_sinex;
use siderust_pod_io::ParseMode;

const SAMPLE: &[u8] = include_bytes!("../../test-data/tiny/sample.snx");

#[test]
fn sinex_parse_station_count() {
    let sol = read_sinex(SAMPLE, ParseMode::Permissive).unwrap();
    assert_eq!(sol.stations.len(), 1);
}

#[test]
fn sinex_station_code() {
    let sol = read_sinex(SAMPLE, ParseMode::Permissive).unwrap();
    assert_eq!(sol.stations[0].code, "MATE");
}

#[test]
fn sinex_position_x() {
    let sol = read_sinex(SAMPLE, ParseMode::Permissive).unwrap();
    let pos = &sol.stations[0].position;
    assert!((pos.x().value() - 4_641_949.398_4).abs() < 1e-3);
}

#[test]
fn sinex_velocity_present() {
    let sol = read_sinex(SAMPLE, ParseMode::Permissive).unwrap();
    assert!(sol.stations[0].velocity_m_yr.is_some());
}

#[test]
fn sinex_strict_roundtrip() {
    let sol = read_sinex(SAMPLE, ParseMode::Strict).unwrap();
    assert_eq!(sol.stations.len(), 1);
}
