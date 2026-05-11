# Changelog

All notable changes to this workspace are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html) per
crate (independent versioning).

## [Unreleased]

### Added

* Reserved sibling reusable crates under `rust/`:
  `siderust-dynamics` (variational/STM, force-model composition,
  thrust-arc physics), `siderust-sgp4` (SGP4 propagator producing typed
  TEME states), `siderust-tle` (TLE/3LE/OMM parser),
  `siderust-spice` (DAF/SPK reader extending `siderust::data::{daf,spk}`
  to Type 3/9/13), and `siderust-lambert` (0/N-revolution Lambert
  solver). Each crate ships with a README defining ownership boundaries
  and a `CHANGELOG.md`. Implementation lands in plan Phases 3–5; the
  scaffolds today only declare crate identity, dependencies on `qtty` /
  `tempoch` / `affn[astro]` / `cheby` / `siderust`, an `AGPL-3.0-or-later`
  license, and `#![forbid(unsafe_code)]`. None of these crates depend on
  `siderust-pod-*`.
* `siderust-pod-core`: new crate scaffolding the POD domain primitives
  referenced by the existing design plan (`error::PodError`,
  `dataset::DatasetRef`, `manifest::RunManifest` with deterministic JSON,
  `parameter::{ParameterKind, Parameter, ParameterOrdering}`,
  `covariance::ParameterCovariance` with symmetry/diagonal validators,
  `providers::{EphemerisProvider, EarthOrientationProvider,
  FrameTransformProvider}` traits). Closes the long-standing gap where
  the workspace plan and changelog referenced this crate but it did not
  exist on disk.
* `siderust-pod-dynamics`: new crate scaffolding the POD-specific
  dynamics composition layer (`force_config::ForceModelConfig`,
  `thrust_arc::ThrustArcConfig` emitting estimable parameters,
  `process_noise::ProcessNoiseConfig`). Numerical propagation and
  analytic STM remain in `siderust` (today) and the future reusable
  `siderust-dynamics` crate; this layer only owns POD-specific
  composition.
* `siderust-pod-dynamics`: cannonball solar-radiation-pressure force
  (`forces::CannonballSrp`) and exponential-density atmospheric drag
  (`forces::ExponentialDrag`). (audit fixes A-09, A-10)
* `siderust-pod-core`: `PodError::NotImplemented` variant for
  capabilities that are scoped to a future milestone.
* `siderust-pod-service`: integration test verifying the runner refuses
  real-input runs with a structured `Unsupported` error rather than
  silently propagating-and-ignoring the inputs (audit fix A-03).
* Top-level `SECURITY.md`, `SUPPORT.md`, `CHANGELOG.md`, and
  `deny.toml` for cargo-deny gating in CI.
* `crates/siderust-pod-dynamics/benches/propagation.rs`: criterion
  bench skeleton for two-body and two-body+J2 RK4 propagation
  (audit fix A-13 starter).

### Changed

* `siderust-pod-service::runner::run`: when real inputs are supplied
  the runner now returns
  `std::io::ErrorKind::Unsupported` carrying the
  `PodError::NotImplemented` message instead of silently invoking the
  legacy propagation-only stub. This closes audit finding C-01.

### Documentation

* `siderust-pod/docs/architecture/dependency-rules.md`: extended forbidden
  edges to include reusable foundational crates (`qtty`, `tempoch`,
  `affn`, `cheby`, `siderust`) which must not depend on any
  `siderust-pod-*` crate. Enforced by `scripts/check_dep_graph.sh`.
* `plan.md` §13 records the post-M7 technical audit (six 🔴 critical,
  nine 🟠 high, ten 🟡 medium, five 🟢 low findings) and the M8–M12
  remediation roadmap.
