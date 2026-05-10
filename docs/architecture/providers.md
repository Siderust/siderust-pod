# Provider traits

The five provider traits in `siderust-pod-core::providers` are the seam
between POD code and the upstream `siderust` astronomy crate.

- `EphemerisProvider`: planetary / lunar positions and velocities.
- `EarthOrientationProvider`: precession / nutation / polar motion.
- `FrameTransformProvider`: ITRF ↔ GCRF rotations.
- `GravityFieldProvider`: spherical-harmonics evaluation.
- `AtmosphereDensityProvider`: density at a given epoch and position.

Default implementations live in the same module, wrap the *public*
upstream APIs, and are zero-sized so they cost nothing to pass around.

Tests substitute deterministic mock implementations to avoid pulling in
ephemeris files or external state.
