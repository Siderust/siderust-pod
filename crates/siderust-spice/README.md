# siderust-spice

Reusable DAF/SPK reader and [`EphemerisProvider`] adapter built on top of
[`siderust::data::{daf, spk}`][siderust-data] and [`cheby`][cheby].

The crate exposes the upstream parser unchanged, plus three new layers:

| Layer            | Purpose                                                       |
|------------------|---------------------------------------------------------------|
| [`SpkSegment`]   | Decodes a single SPK summary into an evaluable Chebyshev block.|
| [`SpkKernel`]    | Owns kernel bytes; resolves multi-segment chains via BFS.     |
| [`SpiceEphemerisProvider`] | Typed [`EphemerisProvider`] facade returning [`Position<C, ICRS, Kilometer>`][position] + km/s velocity.|

## Supported SPK data types

| Type | Status        | Notes                                                  |
|------|---------------|--------------------------------------------------------|
|   2  | ✅ shipped    | Chebyshev position; velocity from analytic derivative. |
|   3  | ✅ shipped    | Chebyshev position **and** velocity (independent coefficients).|
|   1, 5, 8, 9, 10, 12, 13, 14, 15, 17, 18, 19, 20, 21 | ❌ rejected with [`SpiceError::UnsupportedDataType`] | Indexed by the kernel but not evaluable. |

Adding a new type only requires extending [`SpkSegment`] and the
`segment_for_summary()` dispatch — all chain resolution and provider
plumbing is type-agnostic.

## Quick start

```no_run
use siderust_spice::{SpkKernel, SpiceEphemerisProvider};
use siderust::coordinates::centers::Barycentric;
use siderust_pod_core::providers::EphemerisProvider;

let kernel = SpkKernel::open("de440.bsp")?;
let provider = SpiceEphemerisProvider::<Barycentric>::new(kernel, 0);
let state = provider.state(/* Earth */ 399, /* TDB s past J2000 */ 0.0)?;
println!("Earth pos = {:?}", state.position);
println!("Earth vel km/s = {:?}", state.velocity_km_s);
# Ok::<_, Box<dyn std::error::Error>>(())
```

## Optional `de440` feature & validation harness

The `de440` feature enables [`tests/de440_validation.rs`], a regression
test that compares `SpkKernel` output against committed CSPICE-generated
reference values to within 1 mm position / 1 mm/s velocity. The test
deliberately ships **without** the `.bsp` file and reads its kernel path
from the `SIDERUST_SPICE_DE_PATH` environment variable; if the variable
is unset, the file is missing, or the reference JSON has no cases, the
test prints a `SKIPPED` diagnostic and exits successfully. CI runners
that have a kernel on disk should set the variable to enable the gate.

## Determinism & safety

The crate is `#![forbid(unsafe_code)]` and contains no platform-specific
endianness assumptions: all word reads go through `f64::from_le_bytes`
in the upstream DAF parser. The provider is `Send + Sync`, with kernel
bytes wrapped in `Arc<SpkKernel>` so several providers can share a
single kernel cheaply.

## License

AGPL-3.0-or-later.

[siderust-data]: https://docs.rs/siderust
[cheby]: https://crates.io/crates/cheby
[`EphemerisProvider`]: siderust_pod_core::providers::EphemerisProvider
[`SpkSegment`]: crate::SpkSegment
[`SpkKernel`]: crate::SpkKernel
[`SpiceEphemerisProvider`]: crate::SpiceEphemerisProvider
[`SpiceError::UnsupportedDataType`]: crate::SpiceError::UnsupportedDataType
[position]: affn::cartesian::Position
