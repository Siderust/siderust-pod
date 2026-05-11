//! Functional tests for the CCSDS OEM orbit ephemeris message (writer only).
//!
//! The current crate only has a writer (`write_oem`). Tests verify the output
//! text against the CCSDS KVN structure defined in CCSDS 502.0-B-2/3.

use super::common::assert_approx;
use siderust::astro::dynamics::{OrbitState, Position, Velocity};
use siderust::time::JulianDate;
use siderust_pod_io::oem::{write_oem, OemMetadata};

// ── helpers ──────────────────────────────────────────────────────────────────

fn sample_meta() -> OemMetadata {
    OemMetadata {
        object_id: "2024-001A".into(),
        object_name: "TEST-SAT".into(),
        ref_frame: "EME2000".into(),
        time_system: "TT".into(),
        center_name: "EARTH".into(),
    }
}

/// Two states spaced 30 seconds around J2000.
fn two_states() -> Vec<OrbitState> {
    vec![
        OrbitState::new(
            JulianDate::new(2_451_545.0),
            Position::new(7_000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        ),
        OrbitState::new(
            JulianDate::new(2_451_545.0 + 30.0 / 86_400.0),
            Position::new(6_999.0, 225.0, 0.0),
            Velocity::new(-0.24, 7.49, 0.0),
        ),
    ]
}

fn write_to_string(meta: &OemMetadata, states: &[OrbitState]) -> String {
    let mut buf = Vec::new();
    write_oem(&mut buf, meta, states).expect("write_oem");
    String::from_utf8(buf).expect("valid UTF-8")
}

// ── header structure tests ────────────────────────────────────────────────────

#[test]
fn oem_ccsds_version_keyword_present() {
    let text = write_to_string(&sample_meta(), &two_states());
    assert!(
        text.contains("CCSDS_OEM_VERS"),
        "CCSDS_OEM_VERS keyword missing"
    );
}

#[test]
fn oem_creation_date_present() {
    let text = write_to_string(&sample_meta(), &two_states());
    assert!(text.contains("CREATION_DATE"), "CREATION_DATE missing");
}

#[test]
fn oem_originator_present() {
    let text = write_to_string(&sample_meta(), &two_states());
    assert!(text.contains("ORIGINATOR"), "ORIGINATOR missing");
}

#[test]
fn oem_meta_block_delimiters() {
    let text = write_to_string(&sample_meta(), &two_states());
    assert!(text.contains("META_START"), "META_START missing");
    assert!(text.contains("META_STOP"), "META_STOP missing");
}

// ── metadata field tests ──────────────────────────────────────────────────────

#[test]
fn oem_object_name_written() {
    let text = write_to_string(&sample_meta(), &two_states());
    assert!(
        text.contains("OBJECT_NAME          = TEST-SAT"),
        "OBJECT_NAME not found"
    );
}

#[test]
fn oem_object_id_written() {
    let text = write_to_string(&sample_meta(), &two_states());
    assert!(
        text.contains("OBJECT_ID            = 2024-001A"),
        "OBJECT_ID not found"
    );
}

#[test]
fn oem_center_name_written() {
    let text = write_to_string(&sample_meta(), &two_states());
    assert!(text.contains("CENTER_NAME          = EARTH"));
}

#[test]
fn oem_ref_frame_written() {
    let text = write_to_string(&sample_meta(), &two_states());
    assert!(text.contains("REF_FRAME            = EME2000"));
}

#[test]
fn oem_time_system_written() {
    let text = write_to_string(&sample_meta(), &two_states());
    assert!(text.contains("TIME_SYSTEM          = TT"));
}

// ── state vector tests ────────────────────────────────────────────────────────

#[test]
fn oem_state_vector_line_count() {
    let text = write_to_string(&sample_meta(), &two_states());
    // Both states are at J2000 (2000-01-01).
    let data_lines = text.lines().filter(|l| l.starts_with("2000-")).count();
    assert_eq!(
        data_lines, 2,
        "expected 2 state-vector lines starting with '2000-'"
    );
}

#[test]
fn oem_state_vector_fields_finite() {
    let text = write_to_string(&sample_meta(), &two_states());
    for line in text.lines().filter(|l| l.starts_with("2000-")) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        // Format: epoch  X  Y  Z  Vx  Vy  Vz  (7 fields)
        assert_eq!(
            fields.len(),
            7,
            "state line should have 7 fields: epoch X Y Z Vx Vy Vz; got {line:?}"
        );
        for f in &fields[1..] {
            let v: f64 = f
                .parse()
                .unwrap_or_else(|_| panic!("field not numeric: {f}"));
            assert!(v.is_finite(), "state field {f} is not finite");
        }
    }
}

#[test]
fn oem_first_state_position_values() {
    let text = write_to_string(&sample_meta(), &two_states());
    let first_data = text
        .lines()
        .find(|l| l.starts_with("2000-"))
        .expect("no data lines");
    let fields: Vec<f64> = first_data
        .split_whitespace()
        .skip(1)
        .map(|s| s.parse().unwrap())
        .collect();
    assert_approx(fields[0], 7_000.0, 1e-3, "state[0].X [km]");
    assert_approx(fields[1], 0.0, 1e-3, "state[0].Y [km]");
    assert_approx(fields[2], 0.0, 1e-3, "state[0].Z [km]");
}

#[test]
fn oem_first_state_velocity_values() {
    let text = write_to_string(&sample_meta(), &two_states());
    let first_data = text
        .lines()
        .find(|l| l.starts_with("2000-"))
        .expect("no data lines");
    let fields: Vec<f64> = first_data
        .split_whitespace()
        .skip(1)
        .map(|s| s.parse().unwrap())
        .collect();
    assert_approx(fields[3], 0.0, 1e-6, "state[0].Vx [km/s]");
    assert_approx(fields[4], 7.5, 1e-6, "state[0].Vy [km/s]");
    assert_approx(fields[5], 0.0, 1e-6, "state[0].Vz [km/s]");
}

// ── multi-segment test ────────────────────────────────────────────────────────

#[test]
fn oem_multi_segment_two_meta_blocks() {
    // CCSDS OEM allows multiple segments; writing two produces two META_START blocks.
    let meta1 = OemMetadata {
        object_id: "2024-001A".into(),
        object_name: "SAT-A".into(),
        ref_frame: "EME2000".into(),
        time_system: "TT".into(),
        center_name: "EARTH".into(),
    };
    let meta2 = OemMetadata {
        object_id: "2024-002B".into(),
        object_name: "SAT-B".into(),
        ref_frame: "EME2000".into(),
        time_system: "TT".into(),
        center_name: "EARTH".into(),
    };
    let states2 = vec![OrbitState::new(
        JulianDate::new(2_451_546.0),
        Position::new(-7_000.0, 100.0, 0.0),
        Velocity::new(0.1, -7.5, 0.0),
    )];

    let mut buf = Vec::new();
    write_oem(&mut buf, &meta1, &two_states()).expect("write segment 1");
    write_oem(&mut buf, &meta2, &states2).expect("write segment 2");
    let text = String::from_utf8(buf).expect("utf-8");

    assert_eq!(
        text.matches("META_START").count(),
        2,
        "expected 2 META_START sections for multi-segment OEM"
    );
    assert!(text.contains("SAT-A"), "SAT-A missing from output");
    assert!(text.contains("SAT-B"), "SAT-B missing from output");
}

// ── error tests ───────────────────────────────────────────────────────────────

#[test]
fn oem_empty_states_returns_error() {
    let mut buf = Vec::new();
    let err = write_oem(&mut buf, &sample_meta(), &[]).expect_err("empty states should fail");
    match err {
        siderust_pod_io::PodIoError::Format(msg) => {
            assert!(
                msg.contains("no states"),
                "expected 'no states' in error: {msg}"
            );
        }
        other => panic!("expected PodIoError::Format, got {other:?}"),
    }
}

// ── heavy/ignored tests ──────────────────────────────────────────────────────

/// Future test: verify OEM output against CCSDS sample files or LISA orbit
/// files once such fixtures are available.
///
/// Place a CCSDS OEM sample file at `test-data/official/sample.oem` or a
/// LISA orbit file at `test-data/official/lisa.oem` and implement parsing
/// (currently out of scope — the crate only writes OEM).
///
/// Remove `#[ignore]` when a reader is available.
#[test]
#[ignore]
fn oem_official_ccsds_sample_geometry() {
    // Placeholder: once read_oem exists, parse `test-data/official/sample.oem`,
    // verify CCSDS_OEM_VERS, originator, metadata block fields, and that
    // all state vector positions and velocities are finite.
    todo!("implement read_oem before enabling this test");
}
