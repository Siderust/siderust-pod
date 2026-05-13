//! Functional tests for the RINEX-DORIS stub.
use siderust_pod_io::rinex_doris::read_rinex_doris;
use siderust_pod_io::PodIoError;

#[test]
fn rinex_doris_returns_unsupported() {
    let err = read_rinex_doris(&b""[..]).unwrap_err();
    match err {
        PodIoError::Unsupported(msg) => assert!(msg.contains("doris") || msg.contains("DORIS")),
        other => panic!("expected Unsupported, got {other:?}"),
    }
}
