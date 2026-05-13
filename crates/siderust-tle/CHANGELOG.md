# Changelog

## [Unreleased]

### Added

* Working TLE / 3LE parser with column-accurate field extraction:
  - `parse_tle(line1, line2)` and `parse_3le(name, line1, line2)`,
  - canonical TLE checksum validation via `validate_tle_checksum`,
  - Alpha-5 NORAD-ID extension (`A0000` … `Z9999`, `I` and `O` excluded),
  - typed angles via `qtty_core::angular::Degrees`,
  - typed mean motion via `qtty_core::AngularRate<Turn, Day>`,
  - `tempoch::Time<UTC>` epochs derived from the (year, day-of-year)
    encoding,
  - assumed-decimal-with-exponent field parser (BSTAR, second derivative
    of mean motion),
  - explicit `TleError` taxonomy (length, checksum, leading char,
    classification, satellite-id mismatch, invalid epoch).
* Unit tests covering the public-domain ISS (ZARYA) reference TLE,
  checksum rejection, length rejection, satellite-id mismatch, Alpha-5
  decoding, year expansion, and assumed-decimal-with-exponent parsing.

### Notes

* OMM (KVN / XML / JSON) is intentionally deferred to a follow-up; this
  release only handles classic 2LE / 3LE ASCII.
