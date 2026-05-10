# siderust-pod

Modular, type-safe Precise Orbit Determination (POD) workspace built on
top of the [siderust](../siderust) astronomy stack.

> **Status:** pre-1.0. The synthetic-arc end-to-end pipeline (M0–M7) is
> green on every commit; real GNSS ingestion (SP3, RINEX OBS, RINEX
> NAV, ANTEX → estimator) is scheduled for milestone M9. See
> `plan.md` §13 for the post-M7 audit and the M8–M12 remediation
> roadmap.

## Workspace layout

```
crates/
  siderust-pod-core/          domain primitives, providers, manifest, errors
  siderust-pod-dynamics/      forces (two-body, J2, third-body, SRP, drag), integrators, STM
  siderust-pod-io/            SP3 / RINEX OBS+NAV / ANTEX / EOP / CRD / CPF / OEM (MVP subsets)
  siderust-pod-observations/  GNSS code+carrier, SLR range, corrections
  siderust-pod-estimation/    weighted least-squares + EKF (faer-backed)
  siderust-pod-qc/            residual statistics, RTN/RIC compare, JSON/HTML reports
  siderust-pod-products/      SP3/OEM/residual writers, manifest packaging
  siderust-pod-service/       config loader, pipeline runner, artifact layout
  siderust-pod-cli/           thin CLI over the service crate
  siderust-pod-py/            PyO3 bindings (early)
  siderust-pod-rest/          axum-based REST surface (early; unauthenticated)
```

## Quick start

```bash
# Run the synthetic-arc MVP-1 pipeline:
cargo run -p siderust-pod-cli -- run examples/configs/leo_gnss_mvp1.yaml

# Inspect the resulting manifest:
cargo run -p siderust-pod-cli -- inspect-manifest \
    examples/configs/leo_gnss_mvp1.out/run.manifest.json
```

## Dependencies on upstream crates

`siderust-pod-*` depends on the following foundational crates which are
**read-only** to this workspace:

* [`qtty`](../qtty) — typed physical quantities and units.
* [`tempoch`](../tempoch) — astronomical time scales, EOP, leap seconds.
* [`affn`](../affn) — typed positions, vectors, frames, conics.
* [`cheby`](../cheby) — Chebyshev approximation and interpolation.
* [`siderust`](../siderust) — astronomy, ephemerides, observatories.

Modifications to those crates are out of scope for `siderust-pod` and
require a separate branch + ADR.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/check_dep_graph.sh
cargo deny check          # licence + advisory gating (CI)
```

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE). Commercial-licence enquiries:
see [SUPPORT.md](SUPPORT.md).

## Security

See [SECURITY.md](SECURITY.md).
