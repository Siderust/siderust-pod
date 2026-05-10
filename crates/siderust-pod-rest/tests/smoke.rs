//! Smoke test for the REST surface.

use axum::body::{to_bytes, Body};
use axum::http::Request;
use siderust_pod_rest::{router, AppState, JobStatus};
use tower::ServiceExt;

#[tokio::test]
async fn end_to_end_job_lifecycle() {
    let out = std::env::temp_dir().join(format!("siderust_pod_rest_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).unwrap();
    let state = AppState::new(out);
    let app = router(state);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/jobs")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 202);
    let bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let id = v["id"].as_str().unwrap().to_string();

    for _ in 0..200 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/jobs/{}", id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
        let s: JobStatus = serde_json::from_slice(&bytes).unwrap();
        if matches!(s, JobStatus::Finished { .. }) {
            return;
        }
        if let JobStatus::Failed { error } = s {
            panic!("job failed: {error}");
        }
    }
    panic!("job did not finish in time");
}
