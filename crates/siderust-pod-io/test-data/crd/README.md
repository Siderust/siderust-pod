# CRD Test Fixtures

Synthesised Consolidated Laser Ranging Data (CRD) fixtures for regression testing.
No real observation data is included. All numerical values are plausible but
artificially constructed for structural coverage.

## Authoritative Specification

- **ILRS CRD Format v2.01** (2022):
  <https://ilrs.gsfc.nasa.gov/docs/2022/ILRS_CRD_Format_v2.01.pdf>
- **ILRS CRD Format v1.00** (earlier revision, same field layout except a
  smaller H4 record and absent format-flag field on record-11).

## Fixtures

| File | CRD version | Station | Target | Records |
|------|-------------|---------|--------|---------|
| `lageos1_v2.crd` | 2 | GRAZ (7839) | LAGEOS-1 (SIC 1155) | 5 NPs, C0 config |
| `lageos1_v1.crd` | 1 | HERS (7840) | LAGEOS-1 (SIC 1155) | 3 NPs, minimal C0 |

## Field Reference (CRD v2 record 11)

```
11  sod  tof  sys_cfg_id  epoch_event  filter_flag  data_quality
    format_flag  num_raws  bin_rms_ps  bin_skew  bin_kurtosis
    bin_peak  return_rate  [detector_channel]
```

Two-way time-of-flight (TOF) values are in seconds. The one-way slant range
is derived as `range_m = c × TOF / 2` where `c = 299 792 458 m/s` (IAU 2012).

LAGEOS-1 semi-major axis ≈ 12 270 km → two-way TOF ≈ 0.0816 s.
