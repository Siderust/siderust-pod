//! # REST server entrypoint
//!
//! ## Scientific scope
//!
//! This binary boots the Axum-based REST wrapper around the POD service
//! crate. The scientific behaviour is inherited completely from the
//! underlying service path; the executable only selects bind addresses,
//! output directories, and server lifetime.
//!
//! It is intended for local experimentation and integration testing with
//! the synthetic pipeline rather than for a fully hardened production
//! deployment.
//!
//! ## Technical scope
//!
//! The `main` function reads environment variables for bind and output
//! paths, initializes logging, builds `AppState`, and serves the router
//! returned by `siderust-pod-rest`. Requests and responses are JSON over
//! HTTP.
//!
//! No estimation, parsing, or product logic lives in this file.
//!
//! ## References
//!
//! - Fielding, R., Nottingham, M., & Reschke, J. (2022). HTTP Semantics.
//!   RFC 9110.
//! - Bray, T. (2017). The JavaScript Object Notation (JSON) Data
//!   Interchange Format. RFC 8259.
use std::path::PathBuf;

use siderust_pod::service::rest::{router, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let bind = std::env::var("SIDERUST_POD_REST_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let out = std::env::var("SIDERUST_POD_REST_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("siderust-pod-rest"));
    std::fs::create_dir_all(&out)?;
    let state = AppState::new(out);
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    log::info!("siderust-pod-rest listening on {}", bind);
    axum::serve(listener, app).await?;
    Ok(())
}
