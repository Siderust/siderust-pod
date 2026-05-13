//! Shared helpers for functional integration tests.
#![allow(dead_code)]

use std::path::PathBuf;

/// Returns the absolute path to a file inside `test-data/tiny/`.
pub(super) fn tiny(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-data")
        .join("tiny")
        .join(name)
}

/// Returns the absolute path to a file inside `test-data/official/`.
/// The file may not exist; it is only used by `#[ignore]`d tests.
pub(super) fn official(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-data")
        .join("official")
        .join(name)
}

/// Returns the absolute path to a file inside `test-data/crd/`.
pub(super) fn crd_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-data")
        .join("crd")
        .join(name)
}

/// Returns the absolute path to a file inside `test-data/cpf/`.
pub(super) fn cpf_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-data")
        .join("cpf")
        .join(name)
}

/// Returns the absolute path to a file inside `test-data/lisa/`.
pub(super) fn lisa_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-data")
        .join("lisa")
        .join(name)
}

/// Assert `actual ≈ expected` within `tol`, with a diagnostic message naming
/// the field being checked.
#[track_caller]
pub(super) fn assert_approx(actual: f64, expected: f64, tol: f64, field: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tol,
        "field={field}: expected {expected}, got {actual} (diff={diff:.3e} > tol={tol:.3e})"
    );
}

/// Assert that every consecutive pair in `values` is strictly increasing,
/// naming the sequence for diagnostics.
#[track_caller]
pub(super) fn assert_strictly_increasing(values: &[f64], label: &str) {
    for i in 1..values.len() {
        assert!(
            values[i] > values[i - 1],
            "{label}: not strictly increasing at index {i} ({} <= {})",
            values[i],
            values[i - 1]
        );
    }
}
