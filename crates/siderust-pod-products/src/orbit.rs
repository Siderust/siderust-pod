//! Build SP3 / OEM products from an in-memory state time series.

use siderust_pod_core::OrbitState;
use siderust_pod_io::oem::{write_oem, OemMetadata};
use siderust_pod_io::sp3::{write_sp3, Sp3Epoch, Sp3Position, Sp3Record};
use siderust_pod_io::PodIoError;
use std::io::Write;

/// Convert a Julian Date to a (year, month, day, hour, minute, second) tuple
/// using a proleptic Gregorian calendar. No leap-second handling.
fn jd_to_calendar(jd: f64) -> (i32, u32, u32, u32, u32, f64) {
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
    (year, month, day, hours, mins, secs)
}

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
            let (y, mo, d, h, mi, sec) = jd_to_calendar(s.epoch_tt.jd_value());
            Sp3Epoch {
                year: y,
                month: mo,
                day: d,
                hour: h,
                minute: mi,
                second: sec,
                positions: vec![Sp3Position {
                    sat_id: sat_id.to_string(),
                    x_km: s.rx_km,
                    y_km: s.ry_km,
                    z_km: s.rz_km,
                    clock_us: 999_999.999_999,
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

    fn fake_states() -> Vec<OrbitState> {
        (0..5)
            .map(|i| {
                OrbitState::new(
                    JulianDate::new(2_451_545.0 + i as f64 * 30.0 / 86_400.0),
                    [7000.0 + i as f64, 0.0, 0.0],
                    [0.0, 7.5, 0.0],
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
