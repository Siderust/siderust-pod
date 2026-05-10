# ADR-0004 — Provider-trait pattern

## Status
Accepted.

## Context
POD code needs ephemerides, Earth orientation, frame transforms, gravity
fields, and atmosphere densities. Reaching directly into `siderust`
modules would couple POD's compile graph and semantics to upstream's
internal layout and cause churn whenever upstream refactors.

## Decision
`siderust-pod-core::providers` defines five small traits:

- `EphemerisProvider`
- `EarthOrientationProvider`
- `FrameTransformProvider`
- `GravityFieldProvider`
- `AtmosphereDensityProvider`

Default implementations wrap public `siderust` items (e.g. `Vsop87Provider`
forwards to `siderust::calculus::ephemeris::Vsop87Ephemeris`). All
downstream POD crates depend on the *traits*, never on `siderust` directly
for these capabilities.

## Consequences
- Upstream API churn is absorbed in `pod-core::providers`.
- Tests can substitute deterministic mock providers.
- A new astronomy backend can be added without touching the estimator.
