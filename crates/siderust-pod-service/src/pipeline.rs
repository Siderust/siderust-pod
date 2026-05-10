//! End-to-end MVP-1 batch POD pipeline.
//!
//! Takes a synthetic GNSS arc (or in the future a real one assembled from
//! RINEX/SP3), runs Gauss-Newton iterations on the joint orbit + clock-bias
//! parameter vector, and writes the MVP-1 artifact set:
//!
//! ```text
//! output_dir/
//!   run.manifest.json
//!   products/orbit.sp3
//!   products/orbit.oem
//!   residuals/residuals.csv
//!   qc/qc.json
//! ```

use crate::synth::SyntheticArc;
use siderust_pod_core::{OrbitState, Position, Velocity, RunManifest};
use siderust_pod_dynamics::forces::{CompositeForce, ForceModel, TwoBody, J2};
use siderust_pod_dynamics::integrators::rk4_propagate_series;
use siderust_pod_dynamics::stm::finite_diff_stm_series;
use siderust_pod_estimation::{
    gauss_newton, NonlinearError, NonlinearOptions, NonlinearReport, NormalEquations,
};
use siderust_pod_observations::gnss::{CarrierPhaseObs, PseudorangeObs};
use siderust_pod_observations::model::{MeasurementModel, Prediction};
use siderust_pod_products::qc_json::QcDocument;
use siderust_pod_products::{
    write_oem_from_states, write_qc_json, write_residuals_csv, write_sp3_from_states, ResidualRow,
};
use siderust_pod_qc::ResidualsByGroup;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Identifier for a simulated GPS satellite.
#[derive(Debug, Clone)]
pub struct GpsSatellite {
    /// Three-character ID (e.g. `G01`).
    pub id: String,
    /// Slot index used by the synthetic ephemeris.
    pub slot: usize,
}

/// Per-epoch bundle of observations.
#[derive(Debug, Clone)]
pub struct ArcEpoch {
    /// Index of the corresponding state in the propagated series.
    pub state_index: usize,
    /// Code observations.
    pub code: Vec<(GpsSatellite, PseudorangeObs)>,
    /// Carrier observations.
    pub carrier: Vec<(GpsSatellite, CarrierPhaseObs)>,
}

/// Pipeline errors.
#[derive(Debug, Error)]
pub enum PipelineError {
    /// I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Estimation failure.
    #[error("estimation: {0}")]
    Estimation(#[from] NonlinearError),
    /// SP3 write failure.
    #[error("sp3: {0}")]
    Sp3(#[from] siderust_pod_io::sp3::Sp3Error),
    /// I/O umbrella errors from products.
    #[error("io: {0}")]
    PodIo(#[from] siderust_pod_io::PodIoError),
}

/// Outcome of an MVP-1 pipeline run.
#[derive(Debug)]
pub struct PipelineReport {
    /// Estimator convergence report.
    pub estimator: NonlinearReport,
    /// Estimated initial state (epoch t0).
    pub estimated_initial: OrbitState,
    /// Estimated receiver clock bias, metres.
    pub estimated_clock_bias_m: f64,
    /// Final state at end of arc.
    pub estimated_final: OrbitState,
    /// Path to the manifest written.
    pub manifest_path: PathBuf,
    /// Output directory.
    pub output_dir: PathBuf,
}

/// Run MVP-1 against a synthetic arc and write all artifacts to `output_dir`.
pub fn run_synth(
    arc: &SyntheticArc,
    initial_guess: OrbitState,
    initial_clock_guess_m: f64,
    output_dir: &Path,
    run_id: &str,
    enable_j2: bool,
) -> Result<PipelineReport, PipelineError> {
    let dt_s = step_size(arc);
    let n_steps = arc.epochs.len().saturating_sub(1);
    let n_sats = arc.gps_sats.len();
    let n_params = 6 + 1 + n_sats; // state + clock + per-sat float ambiguity

    let mut params = vec![0.0_f64; n_params];
    params[0] = initial_guess.position.x().value();
    params[1] = initial_guess.position.y().value();
    params[2] = initial_guess.position.z().value();
    params[3] = initial_guess.velocity.x().value();
    params[4] = initial_guess.velocity.y().value();
    params[5] = initial_guess.velocity.z().value();
    params[6] = initial_clock_guess_m;
    // Ambiguities (params[7..]) start at zero.

    let opts = NonlinearOptions {
        max_iter: 20,
        tol_rel: 1e-7,
        tol_chi2_rel: 1e-4,
    };

    let force = build_force(enable_j2);
    let arc_ref = arc;
    let report = gauss_newton(
        params,
        opts,
        |params| -> Result<NormalEquations, NonlinearError> {
            assemble_normal_equations(arc_ref, params, dt_s, n_steps, &force)
                .map_err(NonlinearError::Solver)
        },
    )?;

    let estimated_initial = OrbitState::new(
        initial_guess.epoch_tt,
        Position::new(report.parameters[0], report.parameters[1], report.parameters[2]),
        Velocity::new(report.parameters[3], report.parameters[4], report.parameters[5]),
    );
    let clk = report.parameters[6];
    let estimated_states = rk4_propagate_series(&force, estimated_initial, dt_s, n_steps);
    let estimated_final = *estimated_states.last().unwrap();

    // Postfit residuals.
    let residual_rows = postfit_residuals(arc, &estimated_states, &report.parameters);

    // Group statistics.
    let groups = ResidualsByGroup::from_pairs(
        residual_rows
            .iter()
            .map(|r| (residual_kind(&r.kind), r.residual_m)),
    );

    fs::create_dir_all(output_dir.join("products"))?;
    fs::create_dir_all(output_dir.join("residuals"))?;
    fs::create_dir_all(output_dir.join("qc"))?;

    // SP3.
    {
        let mut f = fs::File::create(output_dir.join("products/orbit.sp3"))?;
        write_sp3_from_states(&mut f, "L01", &estimated_states)?;
    }
    // OEM.
    {
        let mut f = fs::File::create(output_dir.join("products/orbit.oem"))?;
        write_oem_from_states(&mut f, "1900-001A", "POD-LEO", &estimated_states)?;
    }
    // Residuals CSV.
    {
        let mut f = fs::File::create(output_dir.join("residuals/residuals.csv"))?;
        write_residuals_csv(&mut f, &residual_rows)?;
    }
    // qc.json.
    {
        let doc = QcDocument {
            schema_version: "0.1.0".into(),
            run_id: run_id.into(),
            software_version: env!("CARGO_PKG_VERSION").into(),
            n_obs: residual_rows.len(),
            n_params,
            reduced_chi2: report.last.reduced_chi2(),
            iterations: report.iterations,
            residuals: groups,
        };
        let mut f = fs::File::create(output_dir.join("qc/qc.json"))?;
        write_qc_json(&mut f, &doc)?;
    }

    // Manifest.
    let mut manifest = RunManifest::new(env!("CARGO_PKG_VERSION"));
    manifest.config_sha256 = String::new();
    manifest
        .outputs
        .push(siderust_pod_core::DatasetRef::from_path(
            "orbit-sp3",
            "SP3",
            &output_dir.join("products/orbit.sp3").to_string_lossy(),
        )?);
    manifest
        .outputs
        .push(siderust_pod_core::DatasetRef::from_path(
            "orbit-oem",
            "OEM",
            &output_dir.join("products/orbit.oem").to_string_lossy(),
        )?);
    manifest
        .outputs
        .push(siderust_pod_core::DatasetRef::from_path(
            "residuals",
            "CSV",
            &output_dir.join("residuals/residuals.csv").to_string_lossy(),
        )?);
    manifest
        .outputs
        .push(siderust_pod_core::DatasetRef::from_path(
            "qc",
            "JSON",
            &output_dir.join("qc/qc.json").to_string_lossy(),
        )?);
    let manifest_path = output_dir.join("run.manifest.json");
    fs::write(
        &manifest_path,
        siderust_pod_core::manifest::canonical_json(&manifest),
    )?;

    Ok(PipelineReport {
        estimator: report,
        estimated_initial,
        estimated_clock_bias_m: clk,
        estimated_final,
        manifest_path,
        output_dir: output_dir.to_path_buf(),
    })
}

fn build_force(enable_j2: bool) -> CompositeForce {
    let mut f = CompositeForce::empty().push(Box::new(TwoBody::earth()));
    if enable_j2 {
        f = f.push(Box::new(J2::earth()));
    }
    f
}

fn step_size(arc: &SyntheticArc) -> f64 {
    if arc.truth_states.len() < 2 {
        return 30.0;
    }
    let dt_jd = arc.truth_states[1].epoch_tt.jd_value() - arc.truth_states[0].epoch_tt.jd_value();
    dt_jd * 86_400.0
}

fn assemble_normal_equations<F: ForceModel>(
    arc: &SyntheticArc,
    params: &[f64],
    dt_s: f64,
    n_steps: usize,
    force: &F,
) -> Result<NormalEquations, siderust_pod_estimation::WlsSolverError> {
    let s0 = OrbitState::new(
        arc.truth_states[0].epoch_tt,
        Position::new(params[0], params[1], params[2]),
        Velocity::new(params[3], params[4], params[5]),
    );
    let states = rk4_propagate_series(force, s0, dt_s, n_steps);
    let stms = finite_diff_stm_series(force, s0, dt_s, n_steps);
    let n_sats = arc.gss_count();
    let n_params = 6 + 1 + n_sats;
    let mut ne = NormalEquations::new(n_params);
    let extras_for_state = &params[6..];

    for ep in &arc.epochs {
        let s = &states[ep.state_index];
        let phi = &stms[ep.state_index];
        for (_sat, obs) in &ep.code {
            let model = siderust_pod_observations::gnss::GnssCodeModel {
                obs: *obs,
                clock_bias_index: 0,
            };
            let pred = model.predict(s, extras_for_state);
            let row = chain_row_sparse(&pred, phi);
            let resid = obs.measured_m - pred.value;
            ne.add_row(&row, resid, obs.sigma_m)?;
        }
        for (sat, obs) in &ep.carrier {
            let model = siderust_pod_observations::gnss::GnssCarrierModel {
                obs: *obs,
                clock_bias_index: 0,
                ambiguity_index: 1 + sat.slot,
            };
            let pred = model.predict(s, extras_for_state);
            let row = chain_row_sparse(&pred, phi);
            let resid = obs.measured_m - pred.value;
            ne.add_row(&row, resid, obs.sigma_m)?;
        }
    }
    Ok(ne)
}

fn chain_row_sparse(pred: &Prediction, phi: &[[f64; 6]; 6]) -> Vec<(usize, f64)> {
    let mut state_partials = [0.0_f64; 6];
    let mut extras: Vec<(usize, f64)> = Vec::new();
    for (j, v) in pred.partials.entries.iter() {
        if *j < 6 {
            state_partials[*j] = *v;
        } else {
            extras.push((*j, *v));
        }
    }
    let mut out = Vec::with_capacity(6 + extras.len());
    for k in 0..6 {
        let mut acc = 0.0;
        for i in 0..6 {
            acc += state_partials[i] * phi[i][k];
        }
        if acc != 0.0 {
            out.push((k, acc));
        }
    }
    out.extend(extras);
    out
}

fn postfit_residuals(
    arc: &SyntheticArc,
    states: &[OrbitState],
    params: &[f64],
) -> Vec<ResidualRow> {
    let extras = &params[6..];
    let mut out = Vec::new();
    for ep in &arc.epochs {
        let s = &states[ep.state_index];
        for (sat, obs) in &ep.code {
            let model = siderust_pod_observations::gnss::GnssCodeModel {
                obs: *obs,
                clock_bias_index: 0,
            };
            let p = model.predict(s, extras);
            out.push(ResidualRow {
                jd_tt: s.epoch_tt.jd_value(),
                kind: format!("code-{}", sat.id),
                measured_m: obs.measured_m,
                predicted_m: p.value,
                residual_m: obs.measured_m - p.value,
                sigma_m: obs.sigma_m,
            });
        }
        for (sat, obs) in &ep.carrier {
            let model = siderust_pod_observations::gnss::GnssCarrierModel {
                obs: *obs,
                clock_bias_index: 0,
                ambiguity_index: 1 + sat.slot,
            };
            let p = model.predict(s, extras);
            out.push(ResidualRow {
                jd_tt: s.epoch_tt.jd_value(),
                kind: format!("phase-{}", sat.id),
                measured_m: obs.measured_m,
                predicted_m: p.value,
                residual_m: obs.measured_m - p.value,
                sigma_m: obs.sigma_m,
            });
        }
    }
    out
}

fn residual_kind(name: &str) -> &str {
    if name.starts_with("code-") {
        "code"
    } else if name.starts_with("phase-") {
        "phase"
    } else {
        "other"
    }
}

impl SyntheticArc {
    fn gss_count(&self) -> usize {
        self.gps_sats.len()
    }
}
