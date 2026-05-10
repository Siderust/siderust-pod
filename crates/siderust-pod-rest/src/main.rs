//! REST server entrypoint.

use std::path::PathBuf;

use siderust_pod_rest::{router, AppState};

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
