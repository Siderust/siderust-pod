//! `siderust-pod-py` — PyO3 bindings exposing the MVP-1 pipeline to Python.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::useless_conversion)]

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use siderust_pod_service::{generate, run_synth, OrbitState, SyntheticArcConfig};

/// Run the synthetic-arc MVP-1 pipeline and return a status dict.
#[pyfunction]
#[pyo3(signature = (output_dir, run_id="mvp1", enable_j2=false))]
#[allow(clippy::useless_conversion)]
fn run_mvp1(output_dir: &str, run_id: &str, enable_j2: bool) -> PyResult<PyObject> {
    let cfg = SyntheticArcConfig::default();
    let arc = generate(&cfg);
    let t0 = arc.truth_states[0];
    let [r0x, r0y, r0z] = t0.position_km();
    let [v0x, v0y, v0z] = t0.velocity_km_s();
    let init = OrbitState::new(
        t0.epoch_tt,
        [r0x + 0.05, r0y - 0.05, r0z + 0.05],
        [v0x + 5e-5, v0y - 5e-5, v0z + 5e-5],
    );
    let report = run_synth(
        &arc,
        init,
        0.0,
        std::path::Path::new(output_dir),
        run_id,
        enable_j2,
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Python::with_gil(|py| {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("manifest_path", report.manifest_path.display().to_string())?;
        dict.set_item("iterations", report.estimator.iterations)?;
        dict.set_item("chi2", report.estimator.last.chi2)?;
        Ok(dict.into())
    })
}

/// Initialise the Python module.
#[pymodule]
fn siderust_pod(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_mvp1, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
