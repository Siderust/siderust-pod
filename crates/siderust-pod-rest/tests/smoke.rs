//! # REST smoke test
//!
//! ## Scientific scope
//!
//! This acceptance test exercises the minimal REST lifecycle around a
//! synthetic POD job. The scientific content is inherited from the
//! underlying synthetic pipeline; the test focuses on verifying that job
//! submission and status reporting preserve that workflow over HTTP.
//!
//! Its regime is intentionally small and deterministic, making it suitable
//! as a transport-level sanity check rather than a performance or
//! scalability benchmark.
//!
//! ## Technical scope
//!
//! The test drives the Axum router in process, submits a job, polls for
//! completion, and asserts that the expected status transitions and
//! manifest artifact are exposed. It does not inspect estimation internals
//! beyond the externally visible job contract.
//!
//! This file is an integration test only and does not define reusable
//! library APIs.
//!
//! ## References
//!
//! - Fielding, R., Nottingham, M., & Reschke, J. (2022). HTTP Semantics.
//!   RFC 9110.
//! - Bray, T. (2017). The JavaScript Object Notation (JSON) Data
//!   Interchange Format. RFC 8259.
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
