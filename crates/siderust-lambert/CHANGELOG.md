# Changelog

## [Unreleased]

### Added

* Crate scaffolding for the Lambert two-point boundary-value solver.
* Stable public API surface for the single-revolution solver: `LambertBranch`,
  `LambertDiagnostics`, `LambertSolution`, `LambertError`, `solve_lambert`,
  `solve_lambert_n_rev`. Input validation (positive `mu`, positive
  time-of-flight, non-zero positions, non-collinear positions) is wired up;
  the Householder iteration kernel is reserved for a follow-up release and
  currently returns `LambertError::NotYetImplemented` once preconditions
  pass. Multi-revolution branches return `LambertError::MultiRevNotImplemented`.
  Six unit tests cover the validation matrix.
