// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! # `siderust-pod-dynamics`
//!
//! POD-aware composition layer on top of [`siderust_dynamics`] (which itself
//! delegates to the canonical [`siderust::astro::dynamics`] primitives).
//!
//! ## Scope
//!
//! This crate **does not** reimplement integrators, force-model physics, or
//! variational machinery — those live in `siderust_dynamics` /
//! `siderust::astro::dynamics`. Instead it provides:
//!
//! 1. A **named force-model registry** ([`registry`]) that can be driven by
//!    a configuration file (string keys → factories).
//! 2. **Periodic empirical accelerations** (1-CPR, 2-CPR — [`empirical_periodic`])
//!    that complement the constant RTN model already shipped upstream.
//! 3. A **typed integrator adapter** ([`integrator_adapter`]) wrapping the
//!    [`Integrator`](siderust_dynamics::Integrator) trait so callers can
//!    propagate a [`SpacecraftState`](siderust::astro::dynamics::SpacecraftState)
//!    against a registry-built composite without touching `Box<dyn>`
//!    plumbing.
//! 4. **Parameter STM extension** ([`variational`]) — finite-difference
//!    columns of `∂y/∂p` for estimable parameters (drag scale Cd_scale,
//!    SRP scale Crp_scale, empirical-accel coefficients) that sit *outside*
//!    the 6×6 state STM.
//! 5. **Process-noise** Q-matrix construction ([`process_noise`]) for
//!    sequential (EKF) filters with Gauss–Markov scale terms and white
//!    empirical noise. Time constants are typed [`Second`](siderust::qtty::Second)
//!    values from `qtty`/`tempoch`.
//! 6. **Thrust-arc** parameter declaration ([`thrust_arc`]) consumed by the
//!    estimator parameter vector.
//!
//! All public errors funnel through [`PodDynamicsError`].
//!
//! ## Re-exports
//!
//! The canonical [`Integrator`](siderust_dynamics::Integrator) and concrete
//! RK4 / DOPRI5 / DOP853 integrators are re-exported so consumers can
//! `use siderust_pod_dynamics::*;` and pick up everything needed to wire a
//! POD batch window.
//!
//! ## Example
//!
//! ```
//! use siderust_pod_dynamics::registry::{ForceModelRegistry, ForceModelSpec, ForceModelParams};
//! let mut reg = ForceModelRegistry::with_builtins();
//! let composite = reg.build(&[
//!     ForceModelSpec::named("two_body"),
//!     ForceModelSpec::named("j2"),
//! ]).unwrap();
//! assert_eq!(composite.len(), 2);
//! ```

#![forbid(unsafe_code)]

pub mod empirical_periodic;
pub mod error;
pub mod force_config;
pub mod forces;
pub mod integrator_adapter;
pub mod process_noise;
pub mod registry;
pub mod thrust_arc;
pub mod variational;

pub use empirical_periodic::{EmpiricalPeriodicAcceleration, PeriodicHarmonic};
pub use error::PodDynamicsError;
pub use force_config::ForceModelConfig;
pub use forces::{
    AccelPartials, Acceleration3, CartesianState, DragForce, Epoch, ForceModel,
    ForceModelRegistry as EvaluatingForceModelRegistry, J2PerturbationForce,
    SolarRadiationPressureForce, TwoBodyForce,
};
pub use integrator_adapter::{propagate_orbit, propagate_spacecraft};
pub use process_noise::{
    GaussMarkovParams, PiecewiseSegment, ProcessNoise, ProcessNoiseModel, WhiteAccelPsd,
};
pub use registry::{ForceModelFactory, ForceModelParams, ForceModelRegistry, ForceModelSpec};
pub use thrust_arc::ThrustArcConfig;
pub use variational::{
    param_partials_central_diff, ParamColumn, ParamStmReport, PropagatedArc, VariationalPropagator,
};

#[doc(no_inline)]
pub use siderust_dynamics::{Dop853Integrator, Dopri5Integrator, Integrator, Rk4Integrator};
