# NAIF DAF/SPK

## Summary

NAIF SPICE distributes planetary, satellite, and spacecraft trajectory
data in DAF (Double-precision Array File) container files. The SPK
sub-format ("SP-Kernel") layers a typed segment table on top of DAF and
defines 21 segment types covering Chebyshev, Lagrange, Hermite, and
discrete-state representations. The familiar `.bsp` extension denotes a
binary SPK kernel; there is no separate `.bsp` format.

## Authoritative specs

- DAF: <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/daf.html>
- SPK: <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/spk.html>

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  —    |

## Owning crate

`siderust::data::spk`

## Supported SPK segment types

| Type | Description                                       | Status |
|------|---------------------------------------------------|--------|
|  2   | Chebyshev polynomial — position only              | ✅     |
|  3   | Chebyshev polynomial — position and velocity      | ✅     |

These two types cover the JPL DE-series planetary ephemerides
(DE440, DE441) and most ITRF-to-GCRF station kernels in common use.

## Not supported

The following SPK types return `SpiceError::UnsupportedDataType { ty }`
when encountered. They are explicitly enumerated so that the dispatch
matches against them and produces a clear error rather than panicking on
an unknown data type:

`1, 5, 8, 9, 10, 12, 13, 14, 15, 17, 18, 19, 20, 21`.

Adding a new type requires:

1. Implementing the segment evaluator under `siderust_spice::spk::types`.
2. Wiring it into the `SpkSegment` dispatch enum.
3. Adding regression cases against a JPL-published reference for that type.

See [ADR-0007 — SPK type coverage](../adr/0007-spk-type-coverage.md).

## Deviations

- The reader **does not** load the entire kernel into memory; it memory-
  maps the file (read-only) and decodes segments lazily.
- Endianness is detected from the DAF file record; both `LTL-IEEE` and
  `BIG-IEEE` kernels are supported.
- Comment area (DAF "ID word" followed by reserved record area) is
  exposed as a `String` and used as part of the run manifest provenance
  block.
