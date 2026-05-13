# siderust-pod-core

Core POD domain primitives. All other crates in the `siderust-pod` workspace
build on types defined here.

## Purpose

- POD error taxonomy (`PodError`)
- Run-level provenance: `RunManifest`, `DatasetRef`
- Parameter typing for the estimator: `ParameterKind`, `Parameter`, `ParameterCovariance`
- Arc definitions: `ArcId`, `ArcDefinition`
- Provider traits: `EphemerisProvider`, `EarthOrientationProvider`,
  `FrameTransformProvider`, `GravityFieldProvider`, `AtmosphereDensityProvider`
- Frame marker re-exports for RTN / LVLH / VNC local orbital frames

Geometric primitives (`OrbitState`, `SpacecraftState`, `StateCovariance`,
STM) live in upstream `siderust` and are re-exported here for ergonomics.

## API entry points

```rust
use siderust_pod_core::{PodError, providers::EphemerisProvider};
use siderust_pod_core::parameter::{Parameter, ParameterKind};
use siderust_pod_core::manifest::RunManifest;
```

## Feature flags

None — all features are always-on.

## Example

```rust
use siderust_pod_core::manifest::RunManifest;

let manifest = RunManifest::new("my-run-001");
println!("run id: {}", manifest.run_id());
```

## See also

- [`docs/architecture/overview.md`](../../docs/architecture/overview.md)
- [`docs/architecture/error-model.md`](../../docs/architecture/error-model.md)
- [`docs/adr/0003-typed-public-apis.md`](../../docs/adr/0003-typed-public-apis.md)
