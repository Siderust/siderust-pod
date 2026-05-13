# siderust-pod-rest

HTTP REST API for `siderust-pod`, built on `axum`.

## Purpose

- OpenAPI-documented REST endpoints for submitting, monitoring, and retrieving
  POD jobs
- JWT authentication (configurable; disabled in development mode)
- Job queue backed by `siderust-pod-service`
- Structured JSON responses with typed error bodies
- Healthcheck and metrics endpoints

## Endpoints (planned — Phase 8)

| Method | Path | Description |
|---|---|---|
| `POST` | `/jobs` | Submit a new POD job |
| `GET` | `/jobs/{id}` | Get job status |
| `DELETE` | `/jobs/{id}` | Cancel a job |
| `GET` | `/jobs/{id}/products/sp3` | Download SP3 product |
| `GET` | `/jobs/{id}/products/residuals` | Download residuals CSV |
| `GET` | `/jobs/{id}/qc` | Download QC HTML report |
| `GET` | `/health` | Healthcheck |
| `GET` | `/openapi.json` | OpenAPI 3.1 schema |

## Feature flags

| Flag | Effect |
|---|---|
| `jwt` | Enables JWT authentication (default on in release) |
| `metrics` | Enables Prometheus metrics endpoint (default off) |

## Running

```bash
siderust-pod-rest --config server.yaml --bind 0.0.0.0:8080
```

## See also

- [`crates/siderust-pod-cli/README.md`](../siderust-pod-cli/README.md)
- [`docs/validation/acceptance-tests.md`](../../docs/validation/acceptance-tests.md) — E2E-11
