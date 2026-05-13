# siderust-pod examples

POD-shaped runnable examples. The cloned `siderust` examples that lived
here previously have been removed (Phase 11 — purge); the new POD
example set is implemented in stages alongside Phases 2–11 of the
maturity plan.

Run any example with:

```bash
cargo run --example <name>
```

| Example | Phase | Status |
|---|---|---|
| `01_propagate_two_body` | P11 | pending |
| `02_propagate_full_dynamics` | P11 | pending |
| `03_lambert_earth_to_mars` | P2 (lambert) | shipped |
| `04_sgp4_from_tle` | P2 (sgp4) | shipped |
| `05_spice_ephemeris` | P2 (spice) | pending |
| `06_synthetic_gnss_arc` | P11 | pending |
| `07_real_gnss_pod` | P11 (uses E2E-02 fixture) | pending |
| `08_slr_validation` | P11 | pending |
| `09_ekf_replay` | P11 | pending |
| `10_orbit_overlap` | P11 | pending |
| `11_qc_html_report` | P11 | pending |
| `12_lisa_poc` | P11 (LISA POC) | pending |
| `13_rest_quickstart.sh` | P11 (REST) | pending |

Configuration files for the examples live under `examples/configs/`;
fixtures under `examples/fixtures/` (frozen inputs — see
`docs/validation/fixtures.md` for provenance).
