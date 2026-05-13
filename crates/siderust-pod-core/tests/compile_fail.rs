//! Compile-fail tests asserting that the typed `affn`/`qtty`/`tempoch`
//! invariants on which POD code relies are actually enforced by the type
//! system.
#![allow(unexpected_cfgs)]
//!
//! These tests use [`trybuild`] to assert that specific source snippets
//! **fail** to compile. The exact rustc error text is captured in
//! `tests/compile_fail/*.stderr` and is therefore tied to the Rust
//! version used when generating the snapshots; for that reason the test
//! runs only when the `local_trybuild` cfg flag is set, e.g.:
//!
//! ```sh
//! RUSTFLAGS='--cfg local_trybuild' cargo test -p siderust-pod-core
//! ```
//!
//! In CI it is typically gated to a known toolchain. To regenerate the
//! `.stderr` snapshots locally:
//!
//! ```sh
//! TRYBUILD=overwrite RUSTFLAGS='--cfg local_trybuild' \
//!     cargo test -p siderust-pod-core --test compile_fail
//! ```

#[allow(unexpected_cfgs)]
#[test]
#[cfg_attr(
    not(local_trybuild),
    ignore = "trybuild stderr is rustc-version dependent"
)]
fn typed_invariants_compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
