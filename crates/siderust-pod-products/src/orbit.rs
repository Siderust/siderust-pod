//! # Orbit product assembly
//!
//! ## Scientific scope
//!
//! Precise orbit determination ultimately produces a time history of
//! spacecraft states that must be exchanged in community-standard orbit
//! formats. This module bridges that scientific result into SP3 and OEM
//! products without changing the underlying trajectory meaning.
//!
//! The validity of the output therefore depends entirely on the supplied
//! state series and metadata. The module does not smooth, interpolate, or
//! otherwise alter the orbit solution.
//!
//! ## Technical scope
//!
//! The public entry points are `write_sp3_from_states` and
//! `write_oem_from_states`. They accept a sequence of `OrbitState` values
//! and the minimal metadata needed by the target file format, then delegate
//! low-level serialization to the IO crate.
//!
//! Residual statistics, manifests, and QC summaries are intentionally
//! handled elsewhere.
//!
//! ## References
//!
//! - Consultative Committee for Space Data Systems. (2010). Orbit Data
//!   Messages, CCSDS 502.0-B-2 / 502.0-B-3.
//! - International GNSS Service. (2020). SP3-c / SP3-d Orbit Format
//!   Specification.
use siderust::astro::dynamics::OrbitState;
use siderust_pod_io::oem::{write_oem, OemMetadata};
use qtty::unit::Kilometer;
use siderust::coordinates::frames::GCRS;
use siderust_pod_io::sp3::{write_sp3, Sp3Epoch, Sp3Position, Sp3Record};
use siderust_pod_io::PodIoError;
use qtty::time::Microseconds;
use qtty::Day;
use std::io::Write;
use tempoch::{JulianDate, Time, TT, UTC};

/// Build an SP3 record from an in-memory state series and write it to `w`.
pub fn write_sp3_from_states<W: Write>(
    w: &mut W,
    sat_id: &str,
    states: &[OrbitState],
) -> Result<(), siderust_pod_io::sp3::Sp3Error> {
    let header = vec![
        format!(
            "#dP2024  1  1  0  0  0.00000000      {:5} ORBIT IGS20 HLM  POD",
            states.len()
        ),
        "##  2295      0.00000000   900.00000000 60310 0.0000000000000".to_string(),
        format!("+    1   {sat_id:<3}                                                     "),
        "++         5                                                                  "
            .to_string(),
        "%c L  cc GPS ccc cccc cccc cccc cccc ccccc ccccc ccccc ccccc".to_string(),
        "%c cc cc ccc ccc cccc cccc cccc cccc ccccc ccccc ccccc ccccc".to_string(),
        "%f  1.2500000  1.025000000  0.00000000000  0.000000000000000".to_string(),
        "%f  0.0000000  0.000000000  0.00000000000  0.000000000000000".to_string(),
        "%i    0    0    0    0      0      0      0      0         0".to_string(),
        "%i    0    0    0    0      0      0      0      0         0".to_string(),
        "/* siderust-pod orbit product".to_string(),
    ];
    let epochs = states
        .iter()
        .map(|s| {
            // Bridge the two tempoch versions (siderust uses crates.io tempoch,
            // siderust-pod-io uses the local path version) via the raw f64 JD.
            let epoch_utc: Time<UTC> = JulianDate::<TT>::try_new(Day::new(s.epoch_tt.jd_value()))
                .expect("OrbitState epoch must be finite")
                .to_time()
                .to_scale::<UTC>();
            Sp3Epoch {
                time: epoch_utc,
                positions: vec![Sp3Position {
                    sat_id: sat_id.to_string(),
                    position: siderust::astro::dynamics::Position::<GCRS, Kilometer>::new(
                        s.position.x().value(),
                        s.position.y().value(),
                        s.position.z().value(),
                    ),
                    clock: Microseconds::new(999_999.999_999),
                }],
            }
        })
        .collect();
    let rec = Sp3Record { header, epochs };
    write_sp3(w, &rec)
}

/// Write an OEM product from the same series.
pub fn write_oem_from_states<W: Write>(
    w: &mut W,
    object_id: &str,
    object_name: &str,
    states: &[OrbitState],
) -> Result<(), PodIoError> {
    let meta = OemMetadata {
        object_id: object_id.into(),
        object_name: object_name.into(),
        ref_frame: "EME2000".into(),
        time_system: "TT".into(),
        center_name: "EARTH".into(),
    };
    write_oem(w, &meta, states)
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::time::JulianDate;
    use siderust::astro::dynamics::{Position, Velocity};

    fn fake_states() -> Vec<OrbitState> {
        (0..5)
            .map(|i| {
                OrbitState::new(
                    JulianDate::new(2_451_545.0 + i as f64 * 30.0 / 86_400.0),
                    Position::new(7000.0 + i as f64, 0.0, 0.0),
                    Velocity::new(0.0, 7.5, 0.0),
                )
            })
            .collect()
    }

    #[test]
    fn sp3_writer_emits_n_epochs() {
        let s = fake_states();
        let mut buf = Vec::new();
        write_sp3_from_states(&mut buf, "L01", &s).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert_eq!(text.lines().filter(|l| l.starts_with("PL01")).count(), 5);
    }

    #[test]
    fn oem_writer_emits_data_lines() {
        let s = fake_states();
        let mut buf = Vec::new();
        write_oem_from_states(&mut buf, "1900-001A", "TEST", &s).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert_eq!(text.lines().filter(|l| l.starts_with("2000-")).count(), 5);
    }
}
