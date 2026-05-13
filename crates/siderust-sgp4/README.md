# siderust-sgp4

Reusable SGP4 / SDP4 propagator producing strongly-typed TEME state vectors
from `siderust_tle::Tle` records.

## Status

**Shipped.** The propagator wraps the MIT-licensed pure-Rust
[`sgp4`](https://crates.io/crates/sgp4) crate (v2.4) as a private backend
and exposes a typed Rust API through `Sgp4Propagator`. The wrapper itself
contains no orbital mechanics; the upstream crate is a faithful port of
Vallado, Crawford, Hujsak & Kelso (2006). See `CHANGELOG.md` for details.

## Public surface

* `Sgp4Propagator::from_tle` / `from_tle_with_model` — initialise from a
  typed TLE, optionally selecting a `GravityModel`.
* `Sgp4Propagator::propagate_at(JulianDate<UTC>)` and
  `Sgp4Propagator::propagate_minutes(f64)` — produce a `TemeState`.
* `TemeState` — typed `(epoch, position, velocity)` triple over
  `siderust::coordinates::cartesian::position::TEME<Kilometer>` and the
  matching `velocity::TEME<KilometerPerSecond>` aliases.
* `Sgp4Error` — propagation/initialisation error variants.

## Validation

Verified bit-exactly (≤ 1 µm position, ≤ 1 nm·s⁻¹ velocity per axis) against
Vallado's `tcppver.out` reference outputs on six SGP4-VER satellites
(NORAD 5, 4632, 8195, 14128, 22674, 23177) at four epochs each — see
`tests/vallado_sgp4ver.rs` and `test-data/vallado_sgp4ver_subset.toml`.

## Ownership

* Input: `siderust_tle::Tle` (typed Two-Line Element / OMM).
* Output: typed TEME `(position, velocity)` aliases re-exported from this
  crate; downstream frame conversions live in
  `siderust::coordinates::transform::providers::frames_teme`.
* Backend: `sgp4 = "2.4"` (MIT, neuromorphicsystems). Never appears in the
  public API.
* Must not depend on `siderust-pod-*`.

## License

AGPL-3.0-or-later. The vendored Vallado SGP4-VER reference subset under
`test-data/` is repackaged from the upstream `sgp4` crate's MIT-licensed
test fixtures; the underlying numerical reference is public-domain
USAF/NASA data.
