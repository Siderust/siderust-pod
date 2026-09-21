# Examples

The repository currently ships two standalone orbit-mechanics examples plus a configuration-driven synthetic POD pipeline.

| Example | Purpose |
| --- | --- |
| `03_lambert_earth_to_mars` | Demonstrates the typed Lambert solver on an interplanetary transfer geometry |
| `04_sgp4_from_tle` | Parses a TLE and propagates it through the SGP4 integration |

Run them with:

```bash
cargo run --example 03_lambert_earth_to_mars
cargo run --example 04_sgp4_from_tle
```

The synthetic POD service configuration lives at `examples/configs/leo_gnss_mvp1.yaml`:

```bash
cargo run --bin siderust-pod -- validate-config examples/configs/leo_gnss_mvp1.yaml
cargo run --bin siderust-pod -- run examples/configs/leo_gnss_mvp1.yaml
```

The project is pre-1.0. Additional real-data and validation examples will be added when their end-to-end paths are reproducible and covered by CI.
