//! `siderust-pod-rest` — minimal REST surface for the POD service.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use siderust_pod_core::{Position, Velocity};
use siderust_pod_service::{generate, run_synth, OrbitState, SyntheticArcConfig};
use uuid::Uuid;

/// Job status snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum JobStatus {
    /// Job has been accepted but not yet started.
    Pending,
    /// Job is currently running.
    Running,
    /// Job finished successfully.
    Finished {
        /// Path to the run manifest.
        manifest_path: String,
    },
    /// Job failed.
    Failed {
        /// Error message returned by the worker.
        error: String,
    },
}

/// Shared application state (in-memory job table).
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<HashMap<String, JobStatus>>>,
    /// Base output directory for produced artefacts.
    pub output_root: Arc<PathBuf>,
}

impl AppState {
    /// Create a new state rooted at the given output directory.
    pub fn new(output_root: PathBuf) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            output_root: Arc::new(output_root),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct JobRequest {
    #[serde(default)]
    enable_j2: bool,
}

#[derive(Debug, Serialize)]
struct JobAccepted {
    id: String,
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn submit_job(
    State(state): State<AppState>,
    body: Option<Json<JobRequest>>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let req = body.map(|Json(r)| r).unwrap_or_default();
    {
        let mut t = state.inner.lock().unwrap();
        t.insert(id.clone(), JobStatus::Pending);
    }
    let id_clone = id.clone();
    let state_clone = state.clone();
    tokio::task::spawn_blocking(move || {
        run_job(state_clone, id_clone, req.enable_j2);
    });
    (StatusCode::ACCEPTED, Json(JobAccepted { id }))
}

async fn job_status(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let table = state.inner.lock().unwrap();
    match table.get(&id) {
        Some(s) => (StatusCode::OK, Json(s.clone())).into_response(),
        None => (StatusCode::NOT_FOUND, "unknown job id").into_response(),
    }
}

fn run_job(state: AppState, id: String, enable_j2: bool) {
    {
        let mut t = state.inner.lock().unwrap();
        t.insert(id.clone(), JobStatus::Running);
    }
    let cfg = SyntheticArcConfig::default();
    let arc = generate(&cfg);
    let t0 = arc.truth_states[0];
    let [r0x, r0y, r0z] = t0.position_km();
    let [v0x, v0y, v0z] = t0.velocity_km_s();
    let init = OrbitState::new(
        t0.epoch_tt,
        Position::new(r0x + 0.05, r0y - 0.05, r0z + 0.05),
        Velocity::new(v0x + 5e-5, v0y - 5e-5, v0z + 5e-5),
    );
    let out = state.output_root.join(&id);
    let result = run_synth(&arc, init, 0.0, &out, &id, enable_j2);
    let mut t = state.inner.lock().unwrap();
    match result {
        Ok(report) => {
            t.insert(
                id,
                JobStatus::Finished {
                    manifest_path: report.manifest_path.display().to_string(),
                },
            );
        }
        Err(e) => {
            t.insert(
                id,
                JobStatus::Failed {
                    error: e.to_string(),
                },
            );
        }
    }
}

/// Build the axum router with the given application state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/jobs", post(submit_job))
        .route("/jobs/:id", get(job_status))
        .with_state(state)
}
