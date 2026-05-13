# siderust-pod-dynamics

POD-aware composition layer for orbit dynamics. Sits above `siderust-dynamics`
and `siderust` to provide the estimator with a composable force registry,
extended STM, and process-noise builders.

## Purpose

- Composite `ForceModel` registry (two-body, J2/spherical-harmonic gravity,
  third-body, cannonball SRP, atmospheric drag, empirical accels)
- Typed integrator adapters (RK4 / DOP853) propagating `SpacecraftState`
- Extended state-transition matrix including estimable parameter rows
  (Cd_scale, Crp_scale, empirical-accel coefficients)
- Process-noise Q builders (Gauss-Markov for scale factors, white noise for
  empirical accels)

## API entry points

```rust
use siderust_pod_dynamics::{ForceRegistry, ProcessNoiseBuilder};
use siderust_pod_dynamics::integrator::DynamicsIntegrator;
```

## Feature flags

| Flag | Effect |
|---|---|
| `jgm3` | Enables JGM-3 spherical-harmonic gravity (default off) |
| `egm2008` | Enables EGM2008 spherical-harmonic gravity (default off) |

## Example

```rust
use siderust_pod_dynamics::ForceRegistry;

let registry = ForceRegistry::default(); // two-body only
// add additional models:
// registry.add("J2", siderust_pod_dynamics::J2Gravity::default());
```

## See also

- [`crates/siderust-dynamics/README.md`](../siderust-dynamics/README.md) — lower-level integrator primitives
- [`docs/architecture/overview.md`](../../docs/architecture/overview.md)
