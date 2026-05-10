//! CCSDS Orbit Ephemeris Message (OEM) writer.
//!
//! Implements a minimal subset of CCSDS 502.0-B-3 sufficient for MVP-1
//! deliverables: ASCII KVN format, single segment, EPHEMERIS_FORMAT = `OEM`,
//! METADATA + DATA blocks, no covariance.

use crate::PodIoError;
use siderust_pod_core::OrbitState;
use std::io::Write;

/// Metadata for a single OEM segment.
#[derive(Debug, Clone)]
pub struct OemMetadata {
    /// Object identifier (free-form).
    pub object_id: String,
    /// Object name (free-form).
    pub object_name: String,
    /// Inertial reference frame string, e.g. "EME2000" or "GCRF".
    pub ref_frame: String,
    /// Time system string, e.g. "TT" or "UTC".
    pub time_system: String,
    /// Centre body, e.g. "EARTH".
    pub center_name: String,
}

/// Write a CCSDS OEM file from a sequence of states.
pub fn write_oem<W: Write>(
    w: &mut W,
    meta: &OemMetadata,
    states: &[OrbitState],
) -> Result<(), PodIoError> {
    if states.is_empty() {
        return Err(PodIoError::Format(
            "write_oem: no states to write".to_string(),
        ));
    }
    writeln!(w, "CCSDS_OEM_VERS = 3.0")?;
    writeln!(w, "CREATION_DATE  = 2026-01-01T00:00:00")?;
    writeln!(w, "ORIGINATOR     = SIDERUST-POD")?;
    writeln!(w)?;
    writeln!(w, "META_START")?;
    writeln!(w, "OBJECT_NAME          = {}", meta.object_name)?;
    writeln!(w, "OBJECT_ID            = {}", meta.object_id)?;
    writeln!(w, "CENTER_NAME          = {}", meta.center_name)?;
    writeln!(w, "REF_FRAME            = {}", meta.ref_frame)?;
    writeln!(w, "TIME_SYSTEM          = {}", meta.time_system)?;
    writeln!(
        w,
        "START_TIME           = {}",
        jd_to_iso8601(states[0].epoch_tt.jd_value())
    )?;
    writeln!(
        w,
        "STOP_TIME            = {}",
        jd_to_iso8601(states.last().unwrap().epoch_tt.jd_value())
    )?;
    writeln!(w, "META_STOP")?;
    writeln!(w)?;
    for s in states {
        let [rx, ry, rz] = s.position_km();
        let [vx, vy, vz] = s.velocity_km_s();
        writeln!(
            w,
            "{} {:.6} {:.6} {:.6} {:.9} {:.9} {:.9}",
            jd_to_iso8601(s.epoch_tt.jd_value()),
            rx, ry, rz, vx, vy, vz
        )?;
    }
    Ok(())
}

/// Convert a Julian Date (any time scale) to an ISO-8601-ish string.
///
/// Conversion is "calendar date in proleptic Gregorian, fractional seconds";
/// no leap-second handling — the caller is responsible for the time system.
fn jd_to_iso8601(jd: f64) -> String {
    let jd_int = (jd + 0.5).floor() as i64;
    let frac = jd + 0.5 - jd_int as f64;

    let a = jd_int + 32_044;
    let b = (4 * a + 3) / 146_097;
    let c = a - (146_097 * b) / 4;
    let d = (4 * c + 3) / 1_461;
    let e = c - (1_461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day = (e - (153 * m + 2) / 5 + 1) as u32;
    let month = (m + 3 - 12 * (m / 10)) as u32;
    let year = (100 * b + d - 4_800 + (m / 10)) as i32;

    let total_seconds = frac * 86_400.0;
    let hours = (total_seconds / 3600.0).floor() as u32 % 24;
    let mins = ((total_seconds % 3600.0) / 60.0).floor() as u32;
    let secs = total_seconds % 60.0;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:09.6}",
        year, month, day, hours, mins, secs
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::time::JulianDate;

    #[test]
    fn writes_header_and_data_lines() {
        let states = vec![
            OrbitState::new(
                JulianDate::new(2_451_545.0),
                [7000.0, 0.0, 0.0],
                [0.0, 7.5, 0.0],
            ),
            OrbitState::new(
                JulianDate::new(2_451_545.0 + 30.0 / 86_400.0),
                [6999.0, 225.0, 0.0],
                [-0.24, 7.49, 0.0],
            ),
        ];
        let mut buf = Vec::new();
        write_oem(
            &mut buf,
            &OemMetadata {
                object_id: "1900-001A".into(),
                object_name: "TEST-SAT".into(),
                ref_frame: "EME2000".into(),
                time_system: "TT".into(),
                center_name: "EARTH".into(),
            },
            &states,
        )
        .unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("CCSDS_OEM_VERS"));
        assert!(text.contains("META_START"));
        assert!(text.contains("META_STOP"));
        assert!(text.contains("EME2000"));
        assert_eq!(text.lines().filter(|l| l.starts_with("2000-")).count(), 2);
    }

    #[test]
    fn jd_2451545_is_j2000() {
        // J2000 = 2000-01-01 12:00:00 (TT).
        let s = jd_to_iso8601(2_451_545.0);
        assert!(s.starts_with("2000-01-01T12:00"), "{}", s);
    }
}
