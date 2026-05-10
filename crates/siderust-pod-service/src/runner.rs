//! Pipeline runner.
//!
//! Two execution paths are supported:
//!
//! * **Synthetic-arc MVP-1.** When the configuration carries no real GNSS
//!   inputs (i.e. `inputs.sp3` and `inputs.rinex_obs` are both `None`),
//!   the runner generates a deterministic synthetic GPS arc and runs
//!   the full estimation pipeline through [`run_synth`]. This is the
//!   path exercised by the headline integration test
//!   `tests/mvp1_e2e.rs`.
//!
//! * **Real-input ingestion.** When the configuration carries real
//!   inputs the runner currently refuses with
//!   [`PodError::NotImplemented`]. The pre-M9 implementation only
//!   propagated the initial state and silently ignored the real inputs,
//!   which made every integration test green for the wrong reason. That
//!   dead branch was removed in the M8 audit pass; the real ingestion
//!   path is scheduled for milestone M9 (see `plan.md` §13.3).

use crate::config::RunConfig;
use crate::pipeline::run_synth;
use crate::synth::{generate, SyntheticArcConfig};
use siderust_pod_core::{OrbitState, PodError, Position, Velocity};
use std::path::PathBuf;

/// Outcome of a run.
#[derive(Debug)]
pub struct RunReport {
    /// Final state at the end of the propagated arc.
    pub final_state: OrbitState,
    /// Run manifest written to disk.
    pub manifest_path: PathBuf,
    /// Number of integration steps.
    pub n_steps: usize,
}

/// Run a configuration end-to-end.
pub fn run(cfg: &RunConfig, _config_path: &str) -> std::io::Result<RunReport> {
    cfg.validate()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let output_dir = PathBuf::from(&cfg.output_dir);

    if cfg.inputs.sp3.is_some() || cfg.inputs.rinex_obs.is_some() {
        // Refuse rather than silently propagate-and-ignore.
        let err = PodError::NotImplemented(
            "real GNSS ingestion (SP3 / RINEX OBS → estimator) is scheduled for milestone M9; \
             see plan.md §13.3. The current build only supports the synthetic-arc MVP-1 \
             pipeline (`inputs.sp3` and `inputs.rinex_obs` both null)."
                .to_string(),
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            err.to_string(),
        ));
    }

    // Synthetic-arc MVP-1 pipeline.
    let synth_cfg = SyntheticArcConfig::default();
    let arc = generate(&synth_cfg);
    let t0 = arc.truth_states[0];
    let [r0x, r0y, r0z] = t0.position_km();
    let [v0x, v0y, v0z] = t0.velocity_km_s();
    let init = OrbitState::new(
        t0.epoch_tt,
        Position::new(r0x + 0.05, r0y - 0.05, r0z + 0.05),
        Velocity::new(v0x + 5e-5, v0y - 5e-5, v0z + 5e-5),
    );
    let report = run_synth(&arc, init, 0.0, &output_dir, &cfg.run_id, cfg.forces.j2)
        .map_err(std::io::Error::other)?;
    Ok(RunReport {
        final_state: report.estimated_final,
        manifest_path: report.manifest_path,
        n_steps: arc.epochs.len(),
    })
}
