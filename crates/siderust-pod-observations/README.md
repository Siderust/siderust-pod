# siderust-pod-observations

POD observation models, geometric corrections, and measurement residual
computation.

## Purpose

- GNSS pseudorange and carrier-phase observation models
  (ionosphere-free, geometry-free combinations)
- Ionospheric corrections (dual-frequency IF, GRAPHIC)
- Tropospheric corrections (Saastamoinen + mapping function)
- Phase-centre corrections (ANTEX PCO/PCV via `siderust-pod-io`)
- SLR normal-point observation model
- Inter-satellite range model (`InterSatRange` — used by the LISA POC)
- Residual formation and outlier detection

## API entry points

```rust
use siderust_pod_observations::gnss::IonoFreeCombination;
use siderust_pod_observations::slr::NormalPointResidual;
use siderust_pod_observations::intersat::InterSatRange;
```

## Feature flags

None currently.

## Example

```rust
// Example will be added in Phase 6.
// See docs/validation/acceptance-tests.md E2E-02 (GNSS POD)
// and E2E-12 (LISA POC / InterSatRange).
```

## See also

- [`docs/adr/0005-lisa-poc-scope.md`](../../docs/adr/0005-lisa-poc-scope.md)
- [`docs/validation/acceptance-tests.md`](../../docs/validation/acceptance-tests.md)
