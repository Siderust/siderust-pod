# Changelog

## [Unreleased]

### Added

* Crate scaffolding for the DAF/SPK reader extension and
  `SpiceEphemerisProvider` adapter.
* `SpkSegment` decoder for SPK Type 2 (Chebyshev position; analytic
  derivative for velocity) and Type 3 (independent Chebyshev
  position+velocity coefficients).
* `SpkKernel` owning kernel bytes and resolving multi-segment chains
  (forward and reverse edges) via BFS, returning typed
  `SpiceError::OutOfCoverage` vs `SpiceError::NoChain` diagnostics.
* `SpiceEphemerisProvider<C: ReferenceCenter<Params=()>>` exposing the
  upstream `siderust_pod_core::providers::EphemerisProvider` trait,
  returning `Position<C, ICRS, Kilometer>` plus a `[f64; 3]` velocity in
  km/s.
* `naif::naif_id_for_name` and `well_known` constants for the planets,
  Sun, Moon, EMB, and SSB.
* Optional `de440` feature gating
  `tests/de440_validation.rs`, which validates `SpkKernel` against a
  committed CSPICE reference (skipping gracefully when no `.bsp` is
  available on disk or the reference set is empty).

### Unsupported (rejected with `SpiceError::UnsupportedDataType`)

* SPK Types 1, 5, 8, 9, 10, 12, 13, 14, 15, 17, 18, 19, 20, 21 — the
  segments are still indexed in `SpkKernel`, but evaluation returns the
  typed error so callers can detect them programmatically.

