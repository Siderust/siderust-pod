//! # Unsupported real-input rejection test
//!
//! ## Scientific scope
//!
//! This regression test protects the current scientific contract of the
//! service crate: until the real-data ingestion path is implemented,
//! configurations that claim real POD inputs must be rejected explicitly.
//! That avoids a more dangerous failure mode where real inputs are silently
//! ignored or misinterpreted.
//!
//! The test therefore enforces a boundary condition on workflow validity
//! rather than on orbit-estimation accuracy.
//!
//! ## Technical scope
//!
//! The test builds a configuration that points at nominal real inputs,
//! invokes the runner, and asserts that a structured error is returned. It
//! is a narrow contract test for runner validation logic.
//!
//! No estimation or product-writing success path is exercised here.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
use siderust_pod_service::config::{ForcesConfig, InputsConfig, RunConfig};
use siderust_pod_service::run;

#[test]
fn real_input_path_is_refused_until_m9() {
    let cfg = RunConfig {
        schema_version: "1.0.0".to_string(),
        run_id: "audit-a03".to_string(),
        inputs: InputsConfig {
            sp3: Some("/tmp/does-not-need-to-exist.sp3".to_string()),
            rinex_obs: None,
            rinex_nav: None,
            antex: None,
        },
        output_dir: tempdir().into_os_string().into_string().unwrap(),
        forces: ForcesConfig {
            two_body: true,
            j2: false,
            third_body: false,
        },
    };

    let err = run(&cfg, "<inline>").expect_err("must refuse real-input runs");
    assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    let msg = err.to_string();
    assert!(
        msg.contains("not implemented") && msg.contains("M9"),
        "unexpected error message: {msg}",
    );
}

fn tempdir() -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("siderust-pod-audit-a03-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}
