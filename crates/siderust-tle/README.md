# siderust-tle

Two-Line Element (TLE / 3LE) and OMM (KVN / XML / JSON) parser, builder,
and formatter.

## Purpose

- Full TLE and 3LE parser with checksum validation.
- `TleBuilder` for programmatic construction and mutation.
- OMM (Orbit Mean-elements Message) reader and writer in KVN, XML, and JSON
  variants, covering the CCSDS OMM 2.0 schema.
- Round-trip parity (parse → format → re-parse).

## API entry points

```rust
use siderust_tle::{Tle, TleBuilder, TleError};
use siderust_tle::omm::{Omm, kvn, xml, json};
```

## Feature flags

None.

## Example

```rust
use siderust_tle::Tle;

let line1 = "1 25544U 98067A   24001.50000000  .00002182  00000-0  40520-4 0  9990";
let line2 = "2 25544  51.6461  22.0316 0002056  31.8482  42.8673 15.48815683437671";
let tle = Tle::parse(line1, line2).expect("valid TLE");
println!("epoch: {:?}", tle.epoch());
```

## See also

- [`docs/formats/tle-3le.md`](../../docs/formats/tle-3le.md)
- [`docs/formats/omm.md`](../../docs/formats/omm.md)
- `crates/siderust-sgp4/` — propagates a `Tle` using SGP4/SDP4

## License

AGPL-3.0-or-later.
