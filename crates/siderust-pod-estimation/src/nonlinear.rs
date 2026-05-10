//! Gauss-Newton nonlinear iteration around an initial parameter guess.
//!
//! At each iteration we ask the caller to assemble a fresh
//! [`super::NormalEquations`] from the current best estimate, then solve
//! and apply the update. Convergence is declared when the relative norm
//! of the update vector falls below `tol_rel` *and* the change in
//! reduced χ² between successive iterations falls below `tol_chi2_rel`.

use crate::wls::{NormalEquations, WlsResult, WlsSolverError};
use thiserror::Error;

/// Convergence options for [`gauss_newton`].
#[derive(Debug, Clone, Copy)]
pub struct NonlinearOptions {
    /// Maximum number of iterations.
    pub max_iter: usize,
    /// Relative-update convergence tolerance (parameter L2-norm).
    pub tol_rel: f64,
    /// Relative-χ² convergence tolerance.
    pub tol_chi2_rel: f64,
}

impl Default for NonlinearOptions {
    fn default() -> Self {
        Self {
            max_iter: 12,
            tol_rel: 1e-9,
            tol_chi2_rel: 1e-6,
        }
    }
}

/// Errors during nonlinear iteration.
#[derive(Debug, Error)]
pub enum NonlinearError {
    /// WLS solve failed.
    #[error(transparent)]
    Solver(#[from] WlsSolverError),
    /// Reached `max_iter` without convergence.
    #[error("did not converge in {0} iterations")]
    DidNotConverge(usize),
}

/// Result of a nonlinear iteration loop.
#[derive(Debug, Clone)]
pub struct NonlinearReport {
    /// Final parameter vector.
    pub parameters: Vec<f64>,
    /// Final WLS solve.
    pub last: WlsResult,
    /// Number of iterations performed.
    pub iterations: usize,
}

/// Run Gauss-Newton iterations.
///
/// `assemble(params) -> NormalEquations` is invoked at every iteration
/// with the *current* best parameter vector and is responsible for
/// producing prefit residuals and partials around that point.
pub fn gauss_newton<F>(
    initial: Vec<f64>,
    opts: NonlinearOptions,
    mut assemble: F,
) -> Result<NonlinearReport, NonlinearError>
where
    F: FnMut(&[f64]) -> Result<NormalEquations, NonlinearError>,
{
    let mut params = initial;
    let mut last_chi2 = f64::INFINITY;
    #[allow(unused_assignments)]
    let mut last_result: Option<WlsResult> = None;
    for it in 0..opts.max_iter {
        let ne = assemble(&params)?;
        let result = ne.solve()?;
        let mut max_rel = 0.0_f64;
        for (i, dp) in result.update.iter().enumerate() {
            let scale = params[i].abs().max(1.0);
            let rel = dp.abs() / scale;
            if rel > max_rel {
                max_rel = rel;
            }
            params[i] += dp;
        }
        let chi2 = result.reduced_chi2();
        let chi2_rel = if last_chi2.is_finite() && last_chi2 > 0.0 {
            (chi2 - last_chi2).abs() / last_chi2
        } else {
            f64::INFINITY
        };
        last_chi2 = chi2;
        last_result = Some(result);
        if max_rel < opts.tol_rel && chi2_rel < opts.tol_chi2_rel {
            return Ok(NonlinearReport {
                parameters: params,
                last: last_result.unwrap(),
                iterations: it + 1,
            });
        }
    }
    Err(NonlinearError::DidNotConverge(opts.max_iter))
}
