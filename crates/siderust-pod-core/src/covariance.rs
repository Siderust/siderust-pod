//! 6×6 covariance utilities for Cartesian and RTN representations.
//!
//! The covariance is always carried as a row-major 6×6 array indexed as
//! `[r,v]` with sub-blocks
//! ```text
//! [ Prr  Prv ]
//! [ Pvr  Pvv ]
//! ```
//! The Cartesian↔RTN transform uses the block-diagonal rotation
//! `T = blockdiag(R, R)`, i.e. position and velocity are rotated by the
//! same RTN basis matrix. This is the standard *instantaneous* convention
//! and ignores the time-derivative of `R` (valid for short propagation
//! windows; see `docs/architecture/dispatch.md` for the modelling note).

use crate::frames::{rtn_from_state, transpose3, Rotation3};
use crate::state::OrbitState;

/// 6×6 row-major covariance matrix.
pub type Covariance6 = [[f64; 6]; 6];

/// Identity 6×6.
pub fn identity6() -> Covariance6 {
    let mut m = [[0.0; 6]; 6];
    for i in 0..6 {
        m[i][i] = 1.0;
    }
    m
}

/// Build a 6×6 block-diagonal rotation from a 3×3 rotation `R`.
///
/// The result `T` satisfies `[r;v]_local = T · [r;v]_inertial`.
pub fn block_diag(r: &Rotation3) -> Covariance6 {
    let mut t = [[0.0; 6]; 6];
    for i in 0..3 {
        for j in 0..3 {
            t[i][j] = r[i][j];
            t[i + 3][j + 3] = r[i][j];
        }
    }
    t
}

/// `out = a · b`.
fn matmul6(a: &Covariance6, b: &Covariance6) -> Covariance6 {
    let mut out = [[0.0; 6]; 6];
    for i in 0..6 {
        for k in 0..6 {
            let aik = a[i][k];
            if aik == 0.0 {
                continue;
            }
            for j in 0..6 {
                out[i][j] += aik * b[k][j];
            }
        }
    }
    out
}

fn transpose6(a: &Covariance6) -> Covariance6 {
    let mut out = [[0.0; 6]; 6];
    for i in 0..6 {
        for j in 0..6 {
            out[i][j] = a[j][i];
        }
    }
    out
}

/// Rotate `P` by `T`: returns `T · P · Tᵀ`.
pub fn similarity(t: &Covariance6, p: &Covariance6) -> Covariance6 {
    let tp = matmul6(t, p);
    let tt = transpose6(t);
    matmul6(&tp, &tt)
}

/// Convert a Cartesian inertial covariance `P_i` to RTN at the given state.
pub fn cartesian_to_rtn(state: &OrbitState, p_inertial: &Covariance6) -> Covariance6 {
    let r = rtn_from_state(state);
    let t = block_diag(&r);
    similarity(&t, p_inertial)
}

/// Convert an RTN covariance back to Cartesian inertial.
pub fn rtn_to_cartesian(state: &OrbitState, p_rtn: &Covariance6) -> Covariance6 {
    let r = rtn_from_state(state);
    let r_t = transpose3(&r);
    let t = block_diag(&r_t);
    similarity(&t, p_rtn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::time::JulianDate;

    #[test]
    fn identity_round_trip() {
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0),
            [7000.0, 100.0, -200.0],
            [0.5, 7.5, 0.1],
        );
        let p = identity6();
        let p_rtn = cartesian_to_rtn(&s, &p);
        let p_back = rtn_to_cartesian(&s, &p_rtn);
        for i in 0..6 {
            for j in 0..6 {
                assert!((p_back[i][j] - p[i][j]).abs() < 1e-12);
            }
        }
    }
}
