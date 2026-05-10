//! Orbit-vs-orbit comparison (RTN/RIC) at matched epochs.

use affn::cartesian::Displacement;
use affn::frames::GCRS;
use qtty::units::Kilometer;
use siderust::astro::dynamics::OrbitState;
use siderust::astro::dynamics::frames::RTN;

/// Per-epoch RTN difference between an estimated and reference state.
#[derive(Debug, Clone)]
pub struct RtnDiff {
    /// Julian Date (TT) of the comparison epoch.
    pub jd_tt: f64,
    /// Radial difference (m).
    pub r_m: f64,
    /// Along-track difference (m).
    pub t_m: f64,
    /// Cross-track difference (m).
    pub n_m: f64,
}

impl RtnDiff {
    /// Position difference as a typed displacement in the RTN frame (kilometres).
    pub fn position_rtn_km(&self) -> Displacement<RTN, Kilometer> {
        Displacement::new(self.r_m * 1e-3, self.t_m * 1e-3, self.n_m * 1e-3)
    }
}

/// Aggregate RTN statistics over a comparison run.
#[derive(Debug, Clone)]
pub struct RtnSummary {
    /// Number of compared epochs.
    pub n: usize,
    /// 3-D RMS, metres.
    pub rms_3d_m: f64,
    /// RMS of radial component, metres.
    pub rms_r_m: f64,
    /// RMS of along-track, metres.
    pub rms_t_m: f64,
    /// RMS of cross-track, metres.
    pub rms_n_m: f64,
}

/// Pairwise RTN difference between estimated and reference states; epochs are
/// matched by index (caller is responsible for alignment and time-tag check).
pub fn rtn_diff(estimated: &[OrbitState], reference: &[OrbitState]) -> Vec<RtnDiff> {
    let n = estimated.len().min(reference.len());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let e = estimated[i];
        let r = reference[i];
        // Position difference in GCRS (typed Displacement<GCRS, Km>).
        let d_gcrs: Displacement<GCRS, Kilometer> = e.position - r.position;
        let frame = siderust::astro::dynamics::frames::LocalOrbitalFrame::<RTN>::from_state(&r);
        let d_rtn = frame.to_local(d_gcrs);
        out.push(RtnDiff {
            jd_tt: e.epoch_tt.jd_value(),
            r_m: d_rtn.x().value() * 1000.0,
            t_m: d_rtn.y().value() * 1000.0,
            n_m: d_rtn.z().value() * 1000.0,
        });
    }
    out
}

/// Aggregate RMS statistics over a list of RTN differences.
pub fn rtn_summary(diffs: &[RtnDiff]) -> RtnSummary {
    let n = diffs.len();
    if n == 0 {
        return RtnSummary {
            n: 0,
            rms_3d_m: 0.0,
            rms_r_m: 0.0,
            rms_t_m: 0.0,
            rms_n_m: 0.0,
        };
    }
    let mut sr = 0.0;
    let mut st = 0.0;
    let mut sn = 0.0;
    for d in diffs {
        sr += d.r_m * d.r_m;
        st += d.t_m * d.t_m;
        sn += d.n_m * d.n_m;
    }
    let rms_r = (sr / n as f64).sqrt();
    let rms_t = (st / n as f64).sqrt();
    let rms_n = (sn / n as f64).sqrt();
    let rms_3d = (rms_r * rms_r + rms_t * rms_t + rms_n * rms_n).sqrt();
    RtnSummary {
        n,
        rms_3d_m: rms_3d,
        rms_r_m: rms_r,
        rms_t_m: rms_t,
        rms_n_m: rms_n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::time::JulianDate;
    use siderust::astro::dynamics::{Position, Velocity};

    #[test]
    fn rtn_zero_for_identical_orbits() {
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            Position::new(7000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        );
        let d = rtn_diff(&[s], &[s]);
        assert_eq!(d.len(), 1);
        assert!(d[0].r_m.abs() < 1e-9);
        let summary = rtn_summary(&d);
        assert!(summary.rms_3d_m < 1e-9);
        assert_eq!(summary.n, 1);
    }

    #[test]
    fn radial_offset_only_appears_in_r_component() {
        let r = OrbitState::new(
            JulianDate::new(2_451_545.0),
            Position::new(7000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        );
        let e = OrbitState::new(
            JulianDate::new(2_451_545.0),
            Position::new(7000.001, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        );
        let d = rtn_diff(&[e], &[r]);
        assert!((d[0].r_m - 1.0).abs() < 1e-9);
        assert!(d[0].t_m.abs() < 1e-9);
        assert!(d[0].n_m.abs() < 1e-9);
    }
}
