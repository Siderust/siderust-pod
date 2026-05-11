# Official Test Datasets

This directory holds large official datasets that are **not committed to git**.
Place downloaded files here and un-ignore the corresponding `#[ignore]` tests.

---

## SP3 — precise orbit & clock (IGS)

| Expected filename | `brdc_sp3.sp3` |
|---|---|
| **Source** | IGS / CDDIS — https://cddis.nasa.gov/archive/gnss/products/ |
| **Format** | SP3-d |
| **Example path** | `ftp://gdc.cddis.eosdis.nasa.gov/gnss/products/2300/igs23P2300.sp3.gz` |
| **Checksum** | `sha256: <fill in after download>` |

---

## RINEX Observation — GNSS raw observations (IGS)

| Expected filename | `obs.rnx` |
|---|---|
| **Source** | IGS / CDDIS — https://cddis.nasa.gov/archive/gnss/data/daily/ |
| **Format** | RINEX 3.04 (mixed GNSS preferred) |
| **Example path** | `ftp://gdc.cddis.eosdis.nasa.gov/gnss/data/daily/2024/001/24o/` |
| **Checksum** | `sha256: <fill in after download>` |

---

## RINEX Navigation — broadcast ephemeris (IGS)

| Expected filename | `brdc_nav.rnx` |
|---|---|
| **Source** | IGS / CDDIS — https://cddis.nasa.gov/archive/gnss/data/daily/ |
| **Format** | RINEX 3.04 mixed navigation |
| **Example path** | `ftp://gdc.cddis.eosdis.nasa.gov/gnss/data/daily/2024/001/24n/brdc0010.24n.gz` |
| **Checksum** | `sha256: <fill in after download>` |

---

## ANTEX — antenna phase-centre offsets/variations (IGS)

| Expected filename | `igs20.atx` |
|---|---|
| **Source** | IGS — https://files.igs.org/pub/station/general/igs20.atx |
| **Format** | ANTEX 1.4 |
| **Direct URL** | `https://files.igs.org/pub/station/general/igs20.atx` |
| **Checksum** | `sha256: <fill in after download>` |

---

## EOP — Earth-orientation parameters (IERS C04)

| Expected filename | `eopc04.eop` |
|---|---|
| **Source** | IERS — https://hpiers.obspm.fr/iers/eop/eopc04/ |
| **Format** | IERS C04 combined series |
| **Direct URL** | `https://hpiers.obspm.fr/iers/eop/eopc04/eopc04.1962-now` |
| **Checksum** | `sha256: <fill in after download>` |

---

## CRD — SLR normal-point observations (ILRS)

| Expected filename | `ilrs.crd` |
|---|---|
| **Source** | ILRS / CDDIS — https://cddis.nasa.gov/archive/slr/data/npt_crd/ |
| **Format** | CRD v2 |
| **Example path** | `https://cddis.nasa.gov/archive/slr/data/npt_crd/2024/01/` |
| **Checksum** | `sha256: <fill in after download>` |

---

## CPF — SLR orbit predictions (ILRS)

| Expected filename | `ilrs.cpf` |
|---|---|
| **Source** | ILRS / CDDIS — https://cddis.nasa.gov/archive/slr/cpf_predicts/ |
| **Format** | CPF v2 |
| **Example path** | `https://cddis.nasa.gov/archive/slr/cpf_predicts/2024/01/` |
| **Checksum** | `sha256: <fill in after download>` |

---

## OEM — CCSDS orbit ephemeris message

> **Note:** The current crate only _writes_ OEM; there is no reader yet.
> This placeholder documents where to put reference files once a reader exists.

| Expected filename | `sample.oem` |
|---|---|
| **Source** | CCSDS samples — https://sanaregistry.org/r/navwork/ or ESA navigation files |
| **Alternate** | LISA Pathfinder orbit files from ESA: `https://www.cosmos.esa.int/web/lisapathfinder/mission-information` |
| **Format** | CCSDS OEM KVN (502.0-B-2 or later) |
| **Checksum** | `sha256: <fill in after download>` |

---

## Verification

After downloading, compute checksums and update the table above:

```sh
sha256sum igs20.atx eopc04.eop brdc_nav.rnx obs.rnx brdc_sp3.sp3 ilrs.crd ilrs.cpf sample.oem
```

Then remove `#[ignore]` from the corresponding test in the relevant file under
`tests/functional/`.

## Important notes

- **No credentials required**: all links above are publicly accessible without
  authentication. CDDIS requires a free NASA Earthdata account for some
  services; use the anonymous FTP mirrors or the HTTPS portal.
- **No files are downloaded automatically during `cargo test`**. All ignored
  tests fail only with a descriptive panic if the file is absent.
- **Do not commit these files** to git; they are listed in `.gitignore`.
