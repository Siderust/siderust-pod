//! Verifies that the runner refuses real-input runs with a structured error
//! rather than silently propagating-and-ignoring the inputs (audit finding
//! C-01 / fix A-03).

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
