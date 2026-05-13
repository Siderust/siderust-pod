# siderust-pod-qc

Quality-control: residual analysis, orbit-compare, SLR validation, and HTML
report generation.

## Purpose

- Residual time-series statistics (mean, σ, RMS, percentiles)
- Orbit comparison (SP3-to-SP3 3D RMS, along/cross/radial decomposition)
- SLR validation (normal-point residuals, station-by-station breakdown)
- QC JSON-Schema manifest
- HTML report generation (Tera-templated; self-contained, no external CDN)
- Sky plots and RMS tables

## API entry points

```rust
use siderust_pod_qc::orbit_compare::OrbitComparison;
use siderust_pod_qc::report::HtmlReport;
```

## Feature flags

| Flag | Effect |
|---|---|
| `html-report` | Enables HTML report generation via `tera` (default on) |

## Example

```rust
// Example will be added in Phase 8.
// See docs/validation/acceptance-tests.md E2E-10.
```

## See also

- [`docs/validation/tolerances.md`](../../docs/validation/tolerances.md)
- [`docs/validation/acceptance-tests.md`](../../docs/validation/acceptance-tests.md)
