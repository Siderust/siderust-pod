# siderust-pod-service

Job model, 15-stage pipeline runner, config loading, and artifact
orchestration.

## Purpose

- YAML/TOML config loader with strong typing and validation
- 15-stage POD pipeline (arc setup → propagation → obs loading →
  preprocessing → filter/smoother → convergence check → product writing →
  QC → manifest)
- EKF sliding-window driver for real-time mode
- Job queue and status tracking
- Artifact provenance linking (inputs → outputs → manifest)

## API entry points

```rust
use siderust_pod_service::config::PodConfig;
use siderust_pod_service::pipeline::PodPipeline;
```

## Feature flags

None currently.

## Example

```bash
# Run via CLI:
siderust-pod run --config examples/configs/leo_gnss.yaml

# Or programmatically:
```

```rust
// Example will be added in Phase 8.
// See examples/09_ekf_replay.rs (Phase 11).
```

## See also

- [`crates/siderust-pod-cli/README.md`](../siderust-pod-cli/README.md)
- [`docs/validation/acceptance-tests.md`](../../docs/validation/acceptance-tests.md)
