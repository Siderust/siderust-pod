#![allow(clippy::needless_range_loop, clippy::inconsistent_digit_grouping)]
//! `siderust-pod-estimation` — batch least-squares estimator (M3).
//!
//! Implements weighted normal-equation assembly and solves the resulting
//! symmetric positive-definite system with `faer`'s dense Cholesky. Robust
//! sigma-editing and EKF/sequential filters are deferred to M6.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod nonlinear;
pub mod sequential;
pub mod wls;

pub use nonlinear::{gauss_newton, NonlinearError, NonlinearOptions, NonlinearReport};
pub use sequential::{Ekf, EkfError, InnovationRecord};
pub use wls::{NormalEquations, WlsResult, WlsSolverError};
