# TLE / 3LE / OMM

## Summary

Two-Line Element sets (TLE) — and their three-line "named" variant 3LE —
are the legacy text format for distributing mean orbital elements to be
used with the SGP4/SDP4 propagator. The CCSDS Orbit Mean-Elements Message
(OMM) is the modern KVN/XML/JSON encoding of the same information.

## Authoritative specs

- TLE / 3LE: <https://celestrak.org/SATCAT/elements/> and
  <https://celestrak.org/publications/AIAA/2006-6753/> (Vallado et al.,
  "Revisiting Spacetrack Report #3", AIAA 2006-6753).
- OMM: see [`omm.md`](./omm.md).

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  —    |

## Owning crate

`siderust::astro::satellite::tle`

## Consumed by

`siderust::astro::satellite::sgp4` reads `Tle` records and propagates them.

## Supported subset

- Classic TLE / 3LE text format.
- OMM in KVN, XML, and JSON serialisations.
- Checksum verification on TLE lines (modulo-10 sum); checksum failures
  are returned as `TleError::ChecksumMismatch { line, expected, actual }`.

## Not supported

- Custom CelesTrak extensions to OMM beyond the documented field set are
  carried as `extra_fields: BTreeMap<String, String>` but not interpreted.
- The "alpha-5" extended NORAD-ID encoding is supported on read; future
  alpha-numeric encodings will be added as they are standardised.
