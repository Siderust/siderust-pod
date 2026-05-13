//! Functional tests for the CCSDS TDM reader/writer.
use siderust_pod_io::tdm::{read_tdm, write_tdm};

const SAMPLE: &[u8] = include_bytes!("../../test-data/tiny/sample.tdm");

#[test]
fn tdm_parses_ok() {
    read_tdm(SAMPLE).unwrap();
}

#[test]
fn tdm_observation_count() {
    let msg = read_tdm(SAMPLE).unwrap();
    assert_eq!(msg.observations.len(), 2);
}

#[test]
fn tdm_range_value() {
    use siderust_pod_io::tdm::ObservationType;
    let msg = read_tdm(SAMPLE).unwrap();
    let range = msg
        .observations
        .iter()
        .find(|o| o.obs_type == ObservationType::Range)
        .unwrap();
    assert!((range.value - 23000.0).abs() < 1e-3);
}

#[test]
fn tdm_roundtrip() {
    let msg = read_tdm(SAMPLE).unwrap();
    let mut buf = Vec::new();
    write_tdm(&mut buf, &msg).unwrap();
    let msg2 = read_tdm(&buf[..]).unwrap();
    assert_eq!(msg2.observations.len(), msg.observations.len());
}
