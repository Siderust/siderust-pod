//! Functional tests for the CCSDS OPM reader/writer.
use siderust_pod_io::opm::{read_opm, write_opm};

const SAMPLE: &[u8] = include_bytes!("../../test-data/tiny/sample.opm");

#[test]
fn opm_parses_ok() {
    read_opm(SAMPLE).unwrap();
}

#[test]
fn opm_object_name() {
    let msg = read_opm(SAMPLE).unwrap();
    assert_eq!(msg.metadata.object_name, "TEST-SAT");
}

#[test]
fn opm_position_x() {
    let msg = read_opm(SAMPLE).unwrap();
    assert!((msg.state.position_km[0] - 7000.0).abs() < 1e-3);
}

#[test]
fn opm_roundtrip() {
    let msg = read_opm(SAMPLE).unwrap();
    let mut buf = Vec::new();
    write_opm(&mut buf, &msg).unwrap();
    let msg2 = read_opm(&buf[..]).unwrap();
    assert_eq!(msg2.metadata.object_name, msg.metadata.object_name);
    assert!((msg2.state.position_km[0] - msg.state.position_km[0]).abs() < 1e-3);
}
