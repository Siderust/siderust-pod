//! Estimator parameter blocks.

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
