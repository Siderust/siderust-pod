# siderust-dynamics

Reusable astrodynamics primitives that sit between `siderust` and the
product crates (`siderust-pod-*`, future `siderust-mission-design`).

## Ownership

| Concern                                        | Owner                     |
| ---------------------------------------------- | ------------------------- |
| Variational equations, analytic STM partials   | **siderust-dynamics**     |
| Force-model composition adapter                | **siderust-dynamics**     |
| Finite-burn / thrust-arc physical model        | **siderust-dynamics**     |
| Two-body / J2 / drag / SRP force kernels       | `siderust::astro::dynamics::forces` |
| Integrators (RK4 / DOPRI / DOP853)             | `siderust::astro::dynamics::integrators` |
| RTN / VNC / LVLH local orbital frames          | `siderust::astro::dynamics::frames`  |
| 6×6 `StateCovariance`                          | `siderust::astro::dynamics::covariance` |
| Position / velocity / Matrix6 geometry         | `affn`                    |
| Time scales / EOP / leap seconds               | `tempoch`                 |
| Typed quantities                               | `qtty`                    |
| Chebyshev evaluation                           | `cheby`                   |

## Non-goals

* No POD-specific workflows. Anything coupled to GNSS observation pipelines,
  RINEX/SP3/ANTEX parsing, estimator parameter wiring, manifests, QC, or
  service orchestration belongs in `siderust-pod-*`.
* No mission-design / global optimization. Reserved for future
  `siderust-mission-design`, `siderust-low-thrust`, `siderust-optimize`.
* Must not depend on any `siderust-pod-*` crate.

## Public API (Phase 2.3 — shipped)

- `Integrator` trait abstracting RK4 and DOP853 (delegates to upstream `siderust`).
- `ThrustArc` / `FiniteBurnArc` — thrust-arc helpers with NaN-safe guard.
- `DeltaVAccumulator` — low-thrust ΔV bookkeeping (cumulative per arc).
- STM validation harness: `validate_stm_finite_diff()` asserting < 1e-7 relative
  agreement for two-body and J2 cases.

## License

AGPL-3.0-or-later.
