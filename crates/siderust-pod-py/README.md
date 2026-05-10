# Python bindings (PyO3)

This crate exposes the Siderust POD MVP-1 pipeline as a Python module.

## Build

```
pip install maturin
maturin develop -m crates/siderust-pod-py/Cargo.toml
```

## Use

```python
import siderust_pod
result = siderust_pod.run_mvp1("/tmp/pod-out", run_id="demo", enable_j2=False)
print(result["manifest_path"], result["iterations"], result["chi2"])
```
