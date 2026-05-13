# Changelog

## [Unreleased]

### Added

* Productized SGP4 / SDP4 propagator (`Sgp4Propagator`) consuming
  `siderust_tle::Tle` and producing strongly-typed TEME state vectors
  (`TemeState`, `TemePositionKm`, `TemeVelocityKmPerSec`,
  `KilometerPerSecond`).
* `GravityModel` selector with three variants:
  * `Wgs72` (default) — WGS-72 geopotential with the AFSPC sidereal-time
    convention; bit-exact against Vallado's `tcppver.out`.
  * `Wgs72Iau` — WGS-72 geopotential with the IAU sidereal-time formula.
  * `Wgs84` — WGS-84 geopotential with the IAU sidereal-time formula.
* `Sgp4Propagator::propagate_at(JulianDate<UTC>)` and
  `propagate_minutes(f64)` entry points returning typed `TemeState`.
* Expanded `Sgp4Error` from a single placeholder variant to four
  documented variants (`InvalidElements`, `InvalidEpoch`, `Propagation`,
  `TimeConversion`); marked `#[non_exhaustive]`.
* Vendored Vallado SGP4-VER reference subset under `test-data/`
  (6 satellites × 4 epochs) and a regression test
  (`tests/vallado_sgp4ver.rs`) asserting ≤ 1e-6 km position /
  ≤ 1e-9 km·s⁻¹ velocity per component.
* Workspace example `examples/04_sgp4_from_tle.rs`.

### Changed

* Backend implementation delegates to the MIT-licensed pure-Rust
  [`sgp4`](https://crates.io/crates/sgp4) crate (v2.4) — declared as a
  private dependency only, never exposed in the public API.
