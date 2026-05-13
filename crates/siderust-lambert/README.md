# siderust-lambert

Izzo 2014 Lambert solver (0-rev and N-rev) with typed `affn`/`qtty` API.

## Purpose

Solves the Lambert two-point boundary-value problem: given two position
vectors and a time-of-flight, find the departure and arrival velocities.

- 0-revolution and N-revolution (multi-rev) branches via Izzo 2014 with
  3rd-order Householder iteration.
- Typed entry-point operating on `affn::cartesian::Position` and
  `qtty::dynamics::GravitationalParameter`.
- Low-level numeric backend on `[f64; 3]` km / km/s for FFI and hot loops.

## API entry points

```rust
use siderust_lambert::{lambert, lambert_n_rev, NRevBranch};
```

## Feature flags

None.

## Example

```rust
use siderust_lambert::solve_lambert;

let r1 = [6578.0, 0.0, 0.0];
let r2 = [0.0, 42_164.0, 0.0];
let tof_s = 18_000.0;
let mu = 398_600.441_8;
let sol = solve_lambert(&r1, &r2, tof_s, mu, 0).unwrap();
println!("v1 = {:?}", sol.v1);
```

See also `examples/03_lambert_earth_to_mars.rs` for a typed end-to-end example.

## See also

- [`docs/formats/`](../../docs/formats/) — no file format; pure computation
- [`docs/validation/tolerances.md`](../../docs/validation/tolerances.md)

## License

AGPL-3.0-or-later.
