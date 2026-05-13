# siderust-pod-products

Orbit and clock product writers, residual output, and run-manifest
serialisation.

## Purpose

- SP3-c/d writer (precise orbit product)
- CCSDS OEM writer (orbit ephemeris)
- Residual CSV and Parquet writers
- Run manifest serialisation (JSON)
- IGS-compatible file-naming convention helpers
- Covariance output formats

## API entry points

```rust
use siderust_pod_products::sp3::Sp3Writer;
use siderust_pod_products::residuals::ResidualCsvWriter;
```

## Feature flags

| Flag | Effect |
|---|---|
| `parquet` | Enables Parquet residual output via `arrow2` (default off) |

## Example

```rust
// Example will be added in Phase 8.
// See docs/validation/acceptance-tests.md E2E-10 (QC report generation).
```

## See also

- [`docs/formats/sp3.md`](../../docs/formats/sp3.md)
- [`docs/formats/ccsds-oem.md`](../../docs/formats/ccsds-oem.md)
