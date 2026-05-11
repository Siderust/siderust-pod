# `siderust-pod-io`

Parsers and writers for POD and geodesy interchange formats used by the `siderust-pod` workspace.

This crate is the format boundary. Other crates should consume the canonical typed records exposed here rather than re-parsing text products themselves.

## Status

Current implemented subsets:

- SP3 reader and writer
- RINEX 3 OBS reader
- RINEX 3 NAV reader
- ANTEX reader
- IERS C04 EOP reader
- CCSDS OEM writer
- CRD reader
- CPF reader

Most modules currently implement MVP subsets, not full standards. The exact supported subset is documented below per module.

## In/Out Entries

| Module | Direction | Entry | Purpose |
| --- | --- | --- | --- |
| `antex` | In | `read_antex<R: Read>(rdr: R) -> Result<AntexCatalog, PodIoError>` | Parse ANTEX antenna phase-centre offsets (PCO only). |
| `cpf` | In | `read_cpf<P: AsRef<Path>>(path: P) -> Result<CpfFile, PodIoError>` | Read a CPF file from disk into canonical position samples. |
| `cpf` | In | `parse_cpf(text: &str) -> Result<CpfFile, PodIoError>` | Parse CPF text already loaded in memory. |
| `crd` | In | `read_crd<P: AsRef<Path>>(path: P) -> Result<CrdFile, PodIoError>` | Read a CRD file from disk into canonical SLR range records. |
| `crd` | In | `parse_crd(text: &str) -> Result<CrdFile, PodIoError>` | Parse CRD text already loaded in memory. |
| `eop` | In | `read_eop_c04<R: Read>(rdr: R) -> Result<Vec<EopRecord>, PodIoError>` | Parse daily IERS C04 Earth-orientation records. |
| `eop` | In/Helper | `interpolate(records: &[EopRecord], mjd: f64) -> Option<EopRecord>` | Linearly interpolate EOP values at a target MJD. |
| `oem` | Out | `write_oem<W: Write>(w: &mut W, meta: &OemMetadata, states: &[OrbitState]) -> Result<(), PodIoError>` | Emit a minimal CCSDS OEM KVN ephemeris product. |
| `rinex_nav` | In | `read_rinex_nav<P: AsRef<Path>>(path: P) -> Result<RinexNavFile, PodIoError>` | Read a RINEX 3 NAV file from disk. |
| `rinex_nav` | In | `parse_rinex_nav(text: &str) -> Result<RinexNavFile, PodIoError>` | Parse RINEX 3 NAV text already loaded in memory. |
| `rinex_obs` | In | `read_rinex_obs<R: Read>(rdr: R) -> Result<RinexObs, PodIoError>` | Parse a RINEX 3 OBS file/stream into station epochs and observables. |
| `sp3` | In | `read_sp3<R: Read>(r: R) -> Result<Sp3Record, Sp3Error>` | Parse an SP3 precise orbit product. |
| `sp3` | Out | `write_sp3<W: Write>(w: &mut W, rec: &Sp3Record) -> Result<(), Sp3Error>` | Write an SP3 record back out. |

## Common Error Types

### `PodIoError`

Shared error type for most modules.

- `PodIoError::Io(std::io::Error)`
  Purpose: wraps lower-level filesystem or stream I/O failures.
- `PodIoError::Format(String)`
  Purpose: reports malformed content or unsupported content in the subset parser/writer.

### `sp3::Sp3Error`

Dedicated SP3 error type.

- `Sp3Error::Io(std::io::Error)`
  Purpose: wraps reader/writer I/O failures.
- `Sp3Error::Header { line, message }`
  Purpose: reports malformed SP3 header content with line context.
- `Sp3Error::Record { line, message }`
  Purpose: reports malformed epoch or position records with line context.

## Module APIs

### `antex`

Purpose: ingest antenna phase-centre offset information needed by GNSS observation modeling.

Supported subset:

- antenna blocks
- frequency blocks
- `NORTH / EAST / UP` PCO values
- PCV grids are skipped

Exported types:

- `AntexNeu`
  Purpose: local ANTEX north-east-up frame marker.
- `Pco = affn::cartesian::Displacement<AntexNeu, qtty::length::Millimeter>`
  Purpose: one phase-centre offset displacement for one antenna at one frequency.
  Components:
  - `x()`: north offset in millimetres.
  - `y()`: east offset in millimetres.
  - `z()`: up offset in millimetres.
- `AntennaPco = HashMap<String, Pco>`
  Purpose: per-antenna frequency map keyed by frequency identifier such as `G01`.
- `AntexCatalog = HashMap<String, AntennaPco>`
  Purpose: full ANTEX result keyed by antenna name.

### `cpf`

Purpose: ingest SLR prediction ephemerides from Consolidated Prediction Format files.

Supported subset:

- `H1` source/version metadata
- `H2` target and reference frame metadata
- `10` position records
- velocity and extension records are ignored

Exported types:

- `CpfPosition`
  Purpose: one predicted target position sample.
  Fields:
  - `mjd`: Modified Julian Day of the sample.
  - `seconds_of_day`: seconds of day in UTC.
  - `r_m`: Cartesian position in metres, following CPF/ITRF convention.
- `CpfFile`
  Purpose: canonical in-memory representation of the parsed CPF subset.
  Fields:
  - `source`: source agency from `H1`.
  - `version`: CPF version from `H1`.
  - `target_name`: target identifier from `H2`.
  - `reference_frame`: frame string from `H2`, for example `ITRF2014`.
  - `positions`: parsed position samples.

### `crd`

Purpose: ingest SLR measurement products from Consolidated Laser Ranging Data files.

Supported subset:

- `H2` station metadata
- `H3` satellite metadata
- `H4` session date
- `C0` system configuration identifier
- `10` full-rate range records
- `11` normal-point range records
- other CRD records are skipped

Exported types:

- `CrdRange`
  Purpose: one SLR observation used for validation or estimation input.
  Fields:
  - `seconds_of_day`: UTC second-of-day at observation epoch.
  - `time_of_flight_s`: measured two-way light time in seconds.
  - `system_config_id`: configuration identifier attached from the latest `C0`.
  - `record_type`: `10` for full-rate or `11` for normal-point.
- `CrdFile`
  Purpose: canonical CRD subset containing session metadata plus ranges.
  Fields:
  - `station_name`: station name from `H2`.
  - `station_cdp_pad`: station CDP/pad identifier.
  - `satellite_name`: target name from `H3`.
  - `satellite_sic`: Satellite Identification Code.
  - `satellite_norad`: COSPAR/NORAD-style string identifier.
  - `year`: session year from `H4`.
  - `month`: session month from `H4`.
  - `day`: session day from `H4`.
  - `ranges`: parsed full-rate or normal-point observations.

### `eop`

Purpose: ingest Earth orientation parameters used by downstream frame and time conversion logic.

Supported subset:

- daily IERS C04-style rows
- comment and blank-line skipping
- linear interpolation helper over parsed records

Exported types:

- `EopRecord`
  Purpose: one Earth-orientation sample at a given UTC MJD.
  Fields:
  - `mjd`: Modified Julian Date in UTC.
  - `x_arcsec`: polar motion x in arcseconds.
  - `y_arcsec`: polar motion y in arcseconds.
  - `ut1_utc_s`: UT1 minus UTC in seconds.
  - `lod_s`: excess length of day in seconds.
  - `dpsi_arcsec`: nutation correction `dψ` in arcseconds.
  - `deps_arcsec`: nutation correction `dε` in arcseconds.

### `oem`

Purpose: emit CCSDS Orbit Ephemeris Message files from propagated or estimated orbit states.

Supported subset:

- ASCII KVN output
- single segment
- metadata block
- data block with position and velocity
- no covariance

Exported types:

- `OemMetadata`
  Purpose: metadata required to describe a single emitted OEM segment.
  Fields:
  - `object_id`: free-form object identifier.
  - `object_name`: free-form object name.
  - `ref_frame`: inertial reference frame string such as `EME2000` or `GCRF`.
  - `time_system`: time system string such as `TT` or `UTC`.
  - `center_name`: center body name such as `EARTH`.

Input state type:

- `siderust::astro::dynamics::OrbitState`
  Purpose: writer input for epoch, position, and velocity samples. `write_oem` consumes a slice of these states and serializes them into OEM records.

### `rinex_nav`

Purpose: ingest broadcast GNSS navigation messages, currently for GPS records needed by `siderust-pod` consumers.

Supported subset:

- RINEX 3 NAV header skipping through `END OF HEADER`
- GPS broadcast ephemeris records
- permissive parsing of partially malformed files
- fields not consumed by current workflows are not modeled

Exported types:

- `GpsNavRecord`
  Purpose: one parsed GPS broadcast ephemeris record.
  Fields:
  - `prn`: GPS PRN as integer satellite id.
  - `year`, `month`, `day`, `hour`, `minute`, `second`: clock data reference epoch.
  - `af0`, `af1`, `af2`: satellite clock bias, drift, and drift rate.
  - `iode`: issue of data ephemeris.
  - `crs`, `delta_n`, `m0`: radial correction, mean motion delta, mean anomaly.
  - `cuc`, `e`, `cus`, `sqrt_a`: latitude correction terms, eccentricity, and square-root semi-major axis.
  - `toe`, `cic`, `omega0`, `cis`: ephemeris reference time and inclination/node correction terms.
  - `i0`, `crc`, `omega`, `omega_dot`, `idot`: inclination, radial correction, argument of perigee, node rate, and inclination rate.
- `RinexNavFile`
  Purpose: parsed NAV container, currently just the list of GPS records.
  Fields:
  - `gps`: all parsed `GpsNavRecord` entries.

### `rinex_obs`

Purpose: ingest observation files for GNSS measurement modeling and quality control.

Supported subset:

- RINEX 3 observation files
- one or more per-system observation type lists
- marker name
- approximate ITRF XYZ
- interval
- epoch blocks beginning with `>`
- per-satellite observables stored by observation code
- only the header fields currently needed by the crate are interpreted

Exported types:

- `ObsEpoch`
  Purpose: one observation epoch with all parsed satellite measurements.
  Fields:
  - `date`: `(year, month, day)`.
  - `time`: `(hour, minute, second)`.
  - `satellites`: nested map `satellite_id -> observation_code -> value`.
- `RinexObs`
  Purpose: parsed observation file with station metadata and all epochs.
  Fields:
  - `marker`: station/marker name.
  - `approx_xyz_m`: optional approximate station Cartesian coordinates in metres.
  - `interval_s`: optional declared sample interval in seconds.
  - `obs_types`: map `system -> ordered observation code list`.
  - `epochs`: parsed epoch records.

### `sp3`

Purpose: ingest and re-emit precise orbit products in SP3 format.

Supported subset:

- header lines `#`, `##`, `+`, `++`, `%c`, `%f`, `%i`, `/*`
- epoch lines `*`
- position lines `P`
- verbatim preservation of header lines for round-trip
- velocity `V`, `EP`, and `EV` records are ignored on read

Exported types:

- `Sp3Position`
  Purpose: one satellite position and clock sample at one epoch.
  Fields:
  - `sat_id`: 3-character satellite id such as `G01`.
  - `x_km`, `y_km`, `z_km`: Cartesian position in kilometres.
  - `clock_us`: satellite clock bias in microseconds.
- `Sp3Epoch`
  Purpose: one SP3 epoch containing all satellite positions for that epoch.
  Fields:
  - `year`, `month`, `day`, `hour`, `minute`, `second`: epoch timestamp components.
  - `positions`: all satellite `P` records at that epoch.
- `Sp3Record`
  Purpose: full parsed SP3 product ready for downstream use or round-trip writing.
  Fields:
  - `header`: verbatim header lines preserved from the source file.
  - `epochs`: parsed epoch records.

## Notes For Callers

- Prefer the `parse_*` entry points when the caller already has the file contents in memory.
- Prefer the `read_*` entry points when the format module owns file loading.
- `rinex_obs`, `antex`, `eop`, and `sp3` accept generic `Read` inputs, which makes them suitable for buffered streams, in-memory fixtures, and compressed-reader adapters.
- `cpf`, `crd`, and `rinex_nav` currently provide path-based readers plus string-based parsers.
- Units are format-native today in several records, for example `km` in SP3 and `mm` in ANTEX PCO. Consumers should normalize explicitly when bridging into typed internal models.

## Relationship To The Rest Of `siderust-pod`

- `siderust-pod-io` owns text/binary product parsing and writing.
- Other crates should work with the canonical record types from this crate rather than depending on file-format specifics.
- When a format subset grows, the intended direction is to extend these typed records instead of leaking ad hoc parser details upward.
