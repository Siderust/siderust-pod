//! Functional tests for the vgosDB stub.
use siderust_pod_io::vgosdb::read_vgosdb;
use siderust_pod_io::PodIoError;
use std::path::Path;

#[test]
fn vgosdb_returns_unsupported() {
    let err = read_vgosdb(Path::new("/nonexistent")).unwrap_err();
    match err {
        PodIoError::Unsupported(msg) => assert!(msg.contains("vgosDB")),
        other => panic!("expected Unsupported, got {other:?}"),
    }
}
