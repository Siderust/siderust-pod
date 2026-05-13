# test-data provenance

This directory holds test fixtures used by the `siderust-pod-io` integration
tests.  Every fixture is either synthesised by hand or a published sample
embedded here for offline testing.

## Directories

### `cpf/`

Hand-crafted CPF (Consolidated Prediction Format) fixtures in versions 1 and 2.
Used by `tests/functional/cpf.rs`.

### `crd/`

Hand-crafted CRD (Consolidated Ranging Data) fixtures.
Used by `tests/functional/crd.rs`.

### `tiny/`

Minimal single-record fixtures for quick smoke-tests across multiple formats.

### `official/`

Placeholder directory for optional real-world files that are **not** checked in
due to size or licence.  Tests that rely on files in this directory are marked
`#[ignore]` and must be downloaded separately.

### `lisa/`

Synthesised CCSDS OEM v2.0 orbit-ephemeris files for the three LISA
spacecraft, matching the format used by the ESA LISA orbit repository at
<https://github.com/esa/lisa-orbit-files>.

Files:

| File | Spacecraft | NAIF ID | Description |
|------|-----------|---------|-------------|
| `lisa_orbit_sample.oem1` | LISA-1 | −1001 | 10 epochs, constant-velocity linear motion |
| `lisa_orbit_sample.oem2` | LISA-2 | −1002 | 10 epochs, constant-velocity linear motion |
| `lisa_orbit_sample.oem3` | LISA-3 | −1003 | 10 epochs, constant-velocity linear motion |

**Format**: CCSDS OEM KVN v2.0 — `CENTER_NAME = SUN`, `REF_FRAME = EME2000`,
`TIME_SYSTEM = TDB`.  Each data row contains: epoch, X, Y, Z (km), VX, VY, VZ
(km/s), AX, AY, AZ (km/s²).

**Provenance**: Synthesised fixture.  Values represent constant-velocity (linear)
trajectories so that cubic Hermite interpolation produces exact results and
numerical tests are self-validating.  The epoch, frame, and metadata fields
replicate the conventions of the real ESA LISA CReMA 1.0 files.

**Reference**: Martens, W., Joffre, E. (2021). Trajectory Design for the ESA
LISA Mission. *Journal of the Astronautical Sciences*, 68, 402–443.
<https://doi.org/10.1007/s40295-021-00263-2>
