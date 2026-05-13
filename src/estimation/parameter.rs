// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! # Estimator parameter descriptors
//!
//! ## Scientific scope
//!
//! Estimator state vectors in POD mix physically different quantities such
//! as Cartesian position, Cartesian velocity, clock biases, and carrier
//! ambiguities. This module names those quantities explicitly so linear
//! algebra over the parameter vector retains a documented physical meaning.
//!
//! The regime is bookkeeping rather than modelling: it does not decide how
//! parameters evolve dynamically or how they are observed. It only
//! describes what each slot in the solved-for vector represents.
//!
//! ## Technical scope
//!
//! The public types are `ParameterKind` and `Parameter`. `ParameterKind`
//! enumerates supported solve-for categories, while `Parameter` stores an a
//! priori value and one-sigma scale in the native units of that quantity.
//!
//! No propagation, residual formation, or covariance transport occurs here.
//! The module is consumed by estimation and service code that needs a
//! stable parameter ordering.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
/// Kinds of parameter the batch/EKF estimator can solve for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParameterKind {
    /// Inertial position component (km), index 0–2.
    Position(u8),
    /// Inertial velocity component (km/s), index 0–2.
    Velocity(u8),
    /// Drag coefficient (`Cd`).
    DragCoefficient,
    /// Solar-radiation pressure coefficient (`Cr`).
    SrpCoefficient,
    /// Receiver clock bias (s).
    ReceiverClockBias,
    /// Float carrier ambiguity (cycles), keyed by satellite + signal id.
    FloatAmbiguity {
        /// PRN / satellite identifier.
        sat_id: u32,
        /// Signal index (e.g. 0 = L1, 1 = L2).
        signal: u8,
    },
}

/// A single parameter in the estimation problem.
#[derive(Debug, Clone, Copy)]
pub struct Parameter {
    /// What this parameter is.
    pub kind: ParameterKind,
    /// A priori value.
    pub a_priori: f64,
    /// A priori standard deviation (same unit as the parameter).
    pub a_priori_sigma: f64,
}
