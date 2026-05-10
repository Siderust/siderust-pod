# Changelog

All notable changes to this workspace are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html) per
crate (independent versioning).

## [Unreleased]

### Added

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

* `plan.md` §13 records the post-M7 technical audit (six 🔴 critical,
  nine 🟠 high, ten 🟡 medium, five 🟢 low findings) and the M8–M12
  remediation roadmap.
