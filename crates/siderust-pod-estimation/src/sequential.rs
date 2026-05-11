//! # Sequential Kalman estimation
//!
//! ## Scientific scope
//!
//! This module provides a sequential estimator shaped like an Extended
//! Kalman Filter for POD-style state updates. It is intended for settings
//! where observations arrive one at a time and the caller can supply both a
//! transition model and scalar measurement linearization.
//!
//! The implementation targets compact, deterministic replay scenarios
//! rather than full operational filtering. Process-noise modelling,
//! smoothing, and advanced numerical stabilization are intentionally kept
//! minimal in the current MVP stage.
//!
//! ## Technical scope
//!
//! The main public items are `Ekf`, `EkfError`, and `InnovationRecord`. The
//! filter operates on a caller-owned state vector and covariance matrix,
//! accepts a propagated mean plus state-transition matrix, and applies
//! scalar observation updates using predicted values, partial derivatives,
//! and sigmas returned by the caller.
//!
//! This module stays generic on purpose: it does not depend on orbit-state
//! types, measurement-format structs, or specific force models.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
use affn::matrix3::{FrameMatrix3, SymmetricFrameMatrix3};
use siderust::astro::dynamics::covariance::StateCovariance;
use siderust::astro::dynamics::{OrbitState, Position, Velocity};
use siderust::coordinates::frames::GCRS;
use siderust::time::JulianDate;
use thiserror::Error;

/// EKF error type.
#[derive(Debug, Error)]
pub enum EkfError {
    /// Singular innovation covariance (S ≤ 0 for a scalar measurement).
    #[error("singular innovation covariance: {0}")]
    Singular(f64),
}

/// Per-measurement innovation/NIS record produced by `update_scalar`.
#[derive(Debug, Clone, Copy)]
pub struct InnovationRecord {
    /// Measurement minus prediction.
    pub innovation: f64,
    /// Innovation variance S = HPHᵀ + R.
    pub variance: f64,
    /// Normalised innovation squared (innovation² / S).
    pub nis: f64,
}

/// EKF state container.
#[derive(Debug, Clone)]
pub struct Ekf {
    /// State vector x.
    pub x: Vec<f64>,
    /// Covariance matrix P, row-major.
    pub p: Vec<f64>,
    /// State dimension n.
    pub n: usize,
}

impl Ekf {
    /// New filter with initial state and diagonal covariance built from `sigma`.
    pub fn new_diag(x0: Vec<f64>, sigma: &[f64]) -> Self {
        let n = x0.len();
        assert_eq!(sigma.len(), n, "sigma must match state dim");
        let mut p = vec![0.0; n * n];
        for i in 0..n {
            p[i * n + i] = sigma[i] * sigma[i];
        }
        Self { x: x0, p, n }
    }

    /// New filter with explicit row-major covariance.
    pub fn new(x0: Vec<f64>, p0: Vec<f64>) -> Self {
        let n = x0.len();
        assert_eq!(p0.len(), n * n, "P must be n×n");
        Self { x: x0, p: p0, n }
    }

    /// Time update: replace the mean with `x_pred` and propagate covariance
    /// as `P ← Φ P Φᵀ + Q`. Caller supplies `phi` row-major and an optional
    /// process-noise matrix `q` row-major (skipped if `None`).
    pub fn predict(&mut self, x_pred: Vec<f64>, phi: &[f64], q: Option<&[f64]>) {
        assert_eq!(x_pred.len(), self.n);
        assert_eq!(phi.len(), self.n * self.n);
        let n = self.n;
        let pp = matmul(phi, &self.p, n, n, n);
        let phi_t = transpose(phi, n);
        let mut p_next = matmul(&pp, &phi_t, n, n, n);
        if let Some(qmat) = q {
            assert_eq!(qmat.len(), n * n);
            for k in 0..n * n {
                p_next[k] += qmat[k];
            }
        }
        self.x = x_pred;
        self.p = p_next;
    }

    /// Scalar measurement update.
    ///
    /// `h_sparse` is the row of partial derivatives ∂y/∂x as
    /// `(state_index, value)` pairs (zeros may be omitted).
    /// `r` is the measurement variance σ². Returns the innovation record.
    pub fn update_scalar(
        &mut self,
        innovation: f64,
        h_sparse: &[(usize, f64)],
        r: f64,
    ) -> Result<InnovationRecord, EkfError> {
        let n = self.n;
        let mut h = vec![0.0; n];
        for &(i, v) in h_sparse {
            if i < n {
                h[i] = v;
            }
        }
        // P h
        let mut ph = vec![0.0; n];
        for i in 0..n {
            let mut s = 0.0;
            for j in 0..n {
                s += self.p[i * n + j] * h[j];
            }
            ph[i] = s;
        }
        // S = h^T P h + R
        let mut s = r;
        for j in 0..n {
            s += h[j] * ph[j];
        }
        if s.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
            return Err(EkfError::Singular(s));
        }
        // K = P h / S
        let k: Vec<f64> = ph.iter().map(|v| v / s).collect();
        // x ← x + K * innovation
        for i in 0..n {
            self.x[i] += k[i] * innovation;
        }
        // P ← (I − K h^T) P (Joseph form for symmetry preservation)
        // Simpler P ← P − K (h^T P) is fine for an MVP.
        let mut htp = vec![0.0; n];
        for j in 0..n {
            let mut acc = 0.0;
            for i in 0..n {
                acc += h[i] * self.p[i * n + j];
            }
            htp[j] = acc;
        }
        for i in 0..n {
            for j in 0..n {
                self.p[i * n + j] -= k[i] * htp[j];
            }
        }
        // Symmetrize.
        for i in 0..n {
            for j in (i + 1)..n {
                let v = 0.5 * (self.p[i * n + j] + self.p[j * n + i]);
                self.p[i * n + j] = v;
                self.p[j * n + i] = v;
            }
        }
        let nis = innovation * innovation / s;
        Ok(InnovationRecord {
            innovation,
            variance: s,
            nis,
        })
    }

    /// Return the orbit state covariance as a typed [`StateCovariance<GCRS>`]
    /// when `n == 6` (position + velocity state).
    ///
    /// Returns `None` for any other state dimension.
    pub fn state_covariance(&self) -> Option<StateCovariance<GCRS>> {
        if self.n != 6 {
            return None;
        }
        let n = self.n;
        let p = &self.p;
        let rr = SymmetricFrameMatrix3::<GCRS>::from_upper([
            [p[0 * n + 0], p[0 * n + 1], p[0 * n + 2]],
            [p[1 * n + 0], p[1 * n + 1], p[1 * n + 2]],
            [p[2 * n + 0], p[2 * n + 1], p[2 * n + 2]],
        ]);
        let rv = FrameMatrix3::<GCRS>::from_array([
            [p[0 * n + 3], p[0 * n + 4], p[0 * n + 5]],
            [p[1 * n + 3], p[1 * n + 4], p[1 * n + 5]],
            [p[2 * n + 3], p[2 * n + 4], p[2 * n + 5]],
        ]);
        let vv = SymmetricFrameMatrix3::<GCRS>::from_upper([
            [p[3 * n + 3], p[3 * n + 4], p[3 * n + 5]],
            [p[4 * n + 3], p[4 * n + 4], p[4 * n + 5]],
            [p[5 * n + 3], p[5 * n + 4], p[5 * n + 5]],
        ]);
        Some(StateCovariance::<GCRS>::from_blocks(rr, rv, vv))
    }

    /// Return a typed orbit state at a given epoch when `n == 6`.
    ///
    /// The state vector is assumed to contain `[x, y, z, vx, vy, vz]`
    /// in kilometres and km/s respectively.
    ///
    /// Returns `None` for any other state dimension.
    pub fn orbit_state(&self, epoch: JulianDate) -> Option<OrbitState> {
        if self.n != 6 {
            return None;
        }
        let pos = Position::<GCRS>::new(self.x[0], self.x[1], self.x[2]);
        let vel = Velocity::<GCRS>::new(self.x[3], self.x[4], self.x[5]);
        Some(OrbitState::new(epoch, pos, vel))
    }
}

fn matmul(a: &[f64], b: &[f64], rows_a: usize, cols_a: usize, cols_b: usize) -> Vec<f64> {
    let mut out = vec![0.0; rows_a * cols_b];
    for i in 0..rows_a {
        for k in 0..cols_a {
            let aik = a[i * cols_a + k];
            for j in 0..cols_b {
                out[i * cols_b + j] += aik * b[k * cols_b + j];
            }
        }
    }
    out
}

fn transpose(m: &[f64], n: usize) -> Vec<f64> {
    let mut t = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            t[j * n + i] = m[i * n + j];
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_update_reduces_variance() {
        let mut f = Ekf::new_diag(vec![0.0], &[1.0]);
        let rec = f.update_scalar(0.5, &[(0, 1.0)], 0.01).unwrap();
        assert!(f.p[0] < 1.0);
        assert!(rec.nis > 0.0);
        // Posterior mean should move toward 0.5.
        assert!(f.x[0] > 0.0 && f.x[0] < 0.5);
    }

    #[test]
    fn predict_inflates_with_q() {
        let mut f = Ekf::new_diag(vec![1.0, 2.0], &[0.1, 0.1]);
        let identity = vec![1.0, 0.0, 0.0, 1.0];
        let q = vec![0.04, 0.0, 0.0, 0.04];
        f.predict(vec![1.0, 2.0], &identity, Some(&q));
        assert!(f.p[0] > 0.01);
        assert!(f.p[3] > 0.01);
    }
}
