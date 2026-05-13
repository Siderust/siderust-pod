# CPF Test Fixtures

Synthesised Consolidated Prediction Format (CPF) fixtures for regression testing.
No real orbit predictions are included. All XYZ values are plausible but
artificially constructed for structural coverage.

## Authoritative Specification

- **ILRS CPF Format v2.00** (2018):
  <https://ilrs.gsfc.nasa.gov/docs/2018/cpf_2.0h-1.pdf>
- **ILRS CPF Format v1.01** (earlier revision, same record-10 layout but
  a shorter H2 header with fewer timing fields).

## Fixtures

| File | CPF version | Target | Step | Positions |
|------|-------------|--------|------|-----------|
| `lageos1_v2.cpf` | 2 | LAGEOS-1 (COSPAR 7603901) | 60 s | 5 |
| `lageos1_v1.cpf` | 1 | LAGEOS-1 (COSPAR 7603901) | 120 s | 3 |

## Field Reference (CPF record 10)

```
10  direction_flag  mjd_int  sod  leap_second_flag  x_m  y_m  z_m
```

- Positions are in metres, referenced to the ITRF (geocentric).
- MJD 60324 = 2024-01-15 UTC.
- The typed API (`CpfEphemerisEntry`) exposes positions in kilometres.

LAGEOS-1 semi-major axis ≈ 12 270 km → position norm ≈ 12 200–12 400 km.
