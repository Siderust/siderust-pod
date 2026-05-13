# Changelog

All notable changes to `siderust-dynamics` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

* Crate scaffolding for the reusable astrodynamics layer with reserved
  module skeletons (`variational`, `thrust`, `forces`).
* README documenting ownership boundaries with `siderust`, `siderust-pod-*`,
  and the future mission-design crates.
* `thrust` module — minimal but unit-safe finite-burn / thrust-arc model:
  - `ThrustArc::with_normalised_direction` validates direction, Isp,
    and interval at construction time;
  - `thrust_acceleration(arc, mass_kg, epoch)` returns
    `[ax, ay, az]` (m/s²) inside the active interval and `[0; 3]` outside;
  - `mass_flow_rate(thrust, isp_seconds)` computes `ṁ = F / (Isp · g₀)`;
  - `G0_M_PER_S2 = 9.80665` exposed for downstream pipelines;
  - `ManeuverError` covers invalid direction / Isp / mass / interval.
  - 7 unit tests including direction normalisation, gating, mass-flow
    arithmetic, and the rejection paths.
