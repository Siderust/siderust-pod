# siderust-pod

[![CI](https://github.com/Siderust/siderust-pod/actions/workflows/ci.yml/badge.svg)](https://github.com/Siderust/siderust-pod/actions/workflows/ci.yml)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)

**Precise Orbit Determination and orbit-analysis tooling in Rust.**

`siderust-pod` is an engineering-oriented toolkit for orbit propagation, observation modelling, state estimation, orbit-product handling, and quality control. It builds on released crates from the wider [Siderust](https://github.com/Siderust) ecosystem and aims to keep physical units, reference frames, time scales, and estimation primitives explicit in the type system.

> **Status: engineering preview (pre-1.0).**
> The synthetic end-to-end POD path is usable for development and validation. Real-data workflows and the REST surface are still evolving. This project is not yet intended for flight-critical or safety-critical operational use.

## What is implemented

| Area | Current scope |
| --- | --- |
| Orbit dynamics | Two-body, J2, third-body gravity, solar-radiation pressure, atmospheric drag, numerical integration, STM support |
| Orbit mechanics | Lambert solver, TLE/3LE/OMM handling, SGP4/SDP4 propagation, SPICE ephemerides |
| Observations | GNSS code/carrier and SLR building blocks; optional LISA-oriented models |
| Estimation | Weighted least squares, Gauss-Newton and EKF components |
| Formats | SP3, RINEX, ANTEX, EOP, CRD, CPF and CCSDS OEM support at the currently implemented subsets |
| Products & QC | Residuals, orbit products, manifests, comparison/QC utilities |
| Interfaces | Rust library, command-line interface, experimental Axum REST API |

The project deliberately separates orbital physics, observations, estimation, I/O, quality control, and service orchestration so each layer can be tested and evolved independently.

## Repository structure

```text
src/
  core/          Domain primitives, parameters, covariance and providers
  dynamics/      Force models, integration and state transition machinery
  estimation/    Batch and sequential estimators
  io/            Space/geodesy format readers and writers
  observations/  Measurement models and corrections
  products/      Orbit and residual products
  qc/            Validation and quality-control tooling
  service/       Configuration-driven pipeline orchestration
  lambert/       Lambert solver
  sgp4/          SGP4 integration
  spice/         SPICE helpers/providers
  tle/           TLE/3LE/OMM handling
  bin/           CLI and experimental REST entry points
```

## Quick start

```bash
git clone https://github.com/Siderust/siderust-pod.git
cd siderust-pod
cargo test
```

Foundational astrodynamics, typed quantities, time scales, frames, and reusable numerical mechanics live in the released Siderust ecosystem crates. `siderust-pod` focuses on precise orbit determination: estimation, observations, orbit products, quality control, and service orchestration. Some primitives first explored during POD development have since moved upstream and are consumed here through their public APIs.

Validate and run the synthetic POD configuration:

```bash
cargo run --bin siderust-pod -- validate-config examples/configs/leo_gnss_mvp1.yaml
cargo run --bin siderust-pod -- run examples/configs/leo_gnss_mvp1.yaml
```

Two standalone orbit-mechanics examples are also included:

```bash
cargo run --example 03_lambert_earth_to_mars
cargo run --example 04_sgp4_from_tle
```

See [examples/README.md](examples/README.md) for details.

## Design principles

- **Type safety for physical quantities.** Units, epochs, frames and states should be difficult to mix accidentally.
- **Traceable numerical behaviour.** Scientific algorithms should have explicit assumptions and validation tests.
- **Separation of concerns.** Dynamics, observations, estimation, formats and orchestration remain independently testable.
- **No unsafe Rust.** The library forbids `unsafe_code`.
- **Standards-oriented interoperability.** Common astrodynamics and geodesy formats are treated as first-class interfaces.

## Validation

The CI pipeline checks formatting, Clippy, tests across feature combinations, documentation, dependency direction, and source hygiene.

Run the main checks locally with:

```bash
cargo fmt -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-fail-fast
cargo test --workspace --no-default-features --no-fail-fast
cargo test --workspace --all-features --no-fail-fast
cargo doc --workspace --no-deps
bash scripts/check_dep_graph.sh
bash scripts/check_no_todos.sh
```

## Contributing

Contributions, validation cases and format-compatibility reports are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

For security issues, see [SECURITY.md](SECURITY.md). For support and commercial-licensing enquiries, see [SUPPORT.md](SUPPORT.md).

## License

`siderust-pod` is available under **AGPL-3.0-or-later**. See [LICENSE](LICENSE).

Commercial licensing can be discussed separately; see [SUPPORT.md](SUPPORT.md).
