# siderust-pod-estimation

Numerical estimation kernels for POD: batch WLS, nonlinear Gauss-Newton,
EKF, and RTS smoother.

## Purpose

- Batch weighted least squares (WLS) for orbit determination
- Nonlinear Gauss-Newton iteration with convergence guard
- Robust WLS (Huber / biweight reweighting)
- Covariance propagation and parameter correlation analysis
- Extended Kalman filter (EKF) sequential update
- Rauch-Tung-Striebel (RTS) backward smoother
- Integer ambiguity resolution via LAMBDA / MLAMBDA (Phase 7)

## API entry points

```rust
use siderust_pod_estimation::batch::BatchLs;
use siderust_pod_estimation::sequential::Ekf;
```

## Feature flags

None currently.

## Example

```rust
// Example will be added when Phase 7 is complete.
// See docs/validation/acceptance-tests.md E2E-08 for an EKF scenario.
```

## See also

- [`docs/requirements/pod-functional-requirements.md`](../../docs/requirements/pod-functional-requirements.md)
- [`docs/validation/acceptance-tests.md`](../../docs/validation/acceptance-tests.md)
