# siderust-pod-cli

Command-line interface for `siderust-pod`.

## Purpose

Binary crate exposing subcommands:

| Subcommand | Description |
|---|---|
| `run` | Run a full POD arc from a config file |
| `propagate` | Propagate an orbit forward / backward |
| `lambert` | Solve a Lambert transfer problem |
| `sgp4` | Propagate a TLE with SGP4/SDP4 |
| `lisa-poc` | Run the LISA heliocentric three-spacecraft POC |
| `qc` | Generate a QC HTML report from existing residuals |
| `validate-config` | Validate a POD config file without running |

## Usage

```bash
siderust-pod run --config leo_gnss.yaml --output /tmp/pod-out/
siderust-pod propagate --state "7000 0 0 0 7.5 0" --duration 86400s
siderust-pod lambert --r1 "6578 0 0" --r2 "0 42164 0" --tof 18000s
```

## Feature flags

None — the CLI binary links all workspace features.

## Building

```bash
cargo build --release -p siderust-pod-cli
```

## See also

- [`crates/siderust-pod-rest/README.md`](../siderust-pod-rest/README.md) — HTTP REST interface
- [`examples/README.md`](../../examples/README.md) — runnable examples
