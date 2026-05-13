//! Functional tests for the OMM KVN/XML reader/writer.
use siderust_pod_io::omm::{read_omm_kvn, write_omm_kvn};

#[test]
fn omm_kvn_roundtrip() {
    use siderust_tle::{omm::Omm, parse_3le};
    let l0 = "ISS (ZARYA)";
    let l1 = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927";
    let l2 = "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
    let tle = parse_3le(l0, l1, l2).unwrap();
    let omm = Omm::from_tle(&tle);
    let mut buf = Vec::new();
    write_omm_kvn(&mut buf, &omm).unwrap();
    let omm2 = read_omm_kvn(&buf[..]).unwrap();
    assert_eq!(omm2.norad_id, omm.norad_id);
}

#[test]
fn omm_xml_roundtrip() {
    use siderust_tle::{omm::Omm, parse_3le};
    let l0 = "ISS (ZARYA)";
    let l1 = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927";
    let l2 = "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
    let tle = parse_3le(l0, l1, l2).unwrap();
    let omm = Omm::from_tle(&tle);
    let mut buf = Vec::new();
    siderust_pod_io::omm::write_omm_xml(&mut buf, &omm).unwrap();
    let omm2 = siderust_pod_io::omm::read_omm_xml(&buf[..]).unwrap();
    assert_eq!(omm2.norad_id, omm.norad_id);
}
