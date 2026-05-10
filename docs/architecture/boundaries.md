# Crate boundaries

| Crate | Owns | Does NOT own |
|---|---|---|
| `siderust-pod-core` | Domain primitives (`OrbitState`, `SpacecraftState`, `RunManifest`, `ParameterKind`, errors), provider traits, POD-specific frame markers. | Numerical algorithms, file parsing, estimator math. |
| `siderust-pod-dynamics` | Force models, integrators, propagation, STM, RTN/LVLH/VNC frame helpers. | File I/O, observation models, estimator. |
| `siderust-pod-io` | Parsers/writers for SP3, RINEX, ANTEX, EOP, OEM, ... | Numerics, estimation. |
| `siderust-pod-observations` | `MeasurementModel` trait + GNSS / SLR / DORIS / VLBI implementations and corrections. | File parsing, estimator solver. |
| `siderust-pod-estimation` | Parameter blocks, design-matrix assembly, WLS / EKF, robust weighting, covariance extraction. | File parsing, observation modelling. |
| `siderust-pod-qc` | Residual statistics, orbit comparison, QC JSON, HTML reports. | Estimation algorithms, product writing. |
| `siderust-pod-products` | SP3 / OEM writers, residual product packaging, manifest packaging, naming, validation. | Numerics, service runtime. |
| `siderust-pod-service` | Job model, config loading/validation, pipeline runner, artifact layout, manifest finalisation. | Numerical algorithms. |
| `siderust-pod-cli` | `clap`-based commands; passthrough to `siderust-pod-service`. | Any numerical logic. |

The full set of allowed and forbidden dependency edges is in
`dependency-rules.md` and is enforced by `scripts/check_dep_graph.sh`
in CI.
