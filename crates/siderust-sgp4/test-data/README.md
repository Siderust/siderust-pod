# Test fixtures

This directory holds reference data used by the SGP4 regression tests.

## `vallado_sgp4ver_subset.toml`

Curated subset of the **SGP4-VER** validation set (Vallado & Crawford,
*"Revisiting Spacetrack Report #3"*, 2006). The SGP4-VER fixtures and the
`tcppver.out` reference outputs are public-domain material distributed by
[celestrak.org](https://celestrak.org) and Vallado's
[website](https://celestrak.org/software/vallado-sw.php).

The TOML repackaging used here was lifted verbatim from the test fixtures
of the pure-Rust [`sgp4`](https://crates.io/crates/sgp4) crate
(MIT-licensed, Copyright © 2020 International Centre for Neuromorphic
Systems). Only six satellites and four epochs each were vendored — the
full SGP4-VER set is ~3 500 lines and is not required for the regression
tolerance we enforce (≤ 1e-6 km position, ≤ 1e-9 km/s velocity).

If the regression is ever extended (e.g. to exercise the Lyddane-bug
satellites or the decayed-orbit cases), pull additional `[[list]]`
entries from the upstream fixture and add a comment line documenting
which Vallado test case they correspond to.
