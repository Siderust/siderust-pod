// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! `siderust-dynamics` — domain-agnostic astrodynamics primitives.
//!
//! ## Scope
//!
//! `siderust-dynamics` is a **thin gap-filler** above
//! [`siderust::astro::dynamics`]. The upstream `siderust` crate already owns
//! the canonical implementations of:
//!
//! * [`OrbitState`], [`ForceModel`], [`CompositeForce`],
//! * the RK4 / DOPRI5 / DOP853 integrators (with `FixedStepper` /
//!   `AdaptiveStepper` traits and a high-level [`Propagator`] facade),
//! * the variational State-Transition-Matrix propagator
//!   [`propagate_stm`] and the finite-difference validation helper
//!   `finite_diff_stm` (see `siderust::astro::dynamics`),
//! * [`DynamicsContext`] and [`siderust::astro::dynamics::errors::DynamicsError`].
//!
//! Per the dependency-rules and design contract, this crate **must not
//! duplicate** any of the above. Instead it adds:
//!
//! 1. A small uniform [`Integrator`] trait that abstracts RK4 / DOPRI5 /
//!    DOP853 behind a single `propagate(force, state, dt, ctx)` method
//!    (delegating to upstream).  Useful for code that wants to swap
//!    integrators at runtime without matching on
//!    [`siderust::astro::dynamics::IntegratorChoice`] every call site.
//! 2. The finite-burn / thrust-arc physical model ([`thrust`]).
//! 3. Low-thrust ΔV bookkeeping ([`low_thrust`]) using the Tsiolkovsky
//!    rocket equation.
//! 4. A validation harness ([`validation`]) that asserts the variational
//!    STM and the finite-difference STM agree to better than `1e-7`
//!    (relative, two-body baseline).
//! 5. A unified [`DynamicsError`] enum that funnels every error originating
//!    from this crate (and any error reraised from upstream) into a single
//!    type, so consumers do not have to import three different error types.
//!
//! ### What this crate does not do
//!
//! * It does not own integrator algorithms; new integrator math goes in
//!   [`siderust::astro::dynamics::integrators`].
//! * It does not own force models; new force-model physics goes in
//!   [`siderust::astro::dynamics::forces`].
//! * It does not own POD-specific orchestration, parameter wiring, or IO —
//!   those live in `siderust-pod-*`.
//!
//! ## Re-exports
//!
//! For convenience, the canonical upstream types are re-exported here so
//! a single `use siderust_dynamics::{...}` can replace several upstream
//! paths.
//!
//! ## Example
//!
//! ```
//! use siderust_dynamics::{Integrator, Rk4Integrator, OrbitState, Position, Velocity};
//! use siderust_dynamics::{DynamicsContext, TwoBody};
//! use siderust::coordinates::frames::GCRS;
//! use siderust::time::JulianDate;
//! use siderust::qtty::Second;
//!
//! let s0 = OrbitState::new_at_jd(
//!     JulianDate::new(2_451_545.0),
//!     Position::<GCRS>::new(7_000.0, 0.0, 0.0),
//!     Velocity::<GCRS>::new(0.0, 7.5450, 0.0),
//! );
//! let force = TwoBody::earth();
//! let ctx = DynamicsContext::empty();
//! let integ = Rk4Integrator { step: Second::new(10.0) };
//! let s1 = integ.propagate(&force, s0, Second::new(600.0), &ctx).unwrap();
//! assert!((s1.epoch - s0.epoch).value().abs() > 0.0);
//! ```

#![forbid(unsafe_code)]

mod error;
pub mod integrators;
pub mod low_thrust;
pub mod thrust;
pub mod validation;

pub use error::DynamicsError;
pub use integrators::{Dop853Integrator, Dopri5Integrator, Integrator, Rk4Integrator};
pub use low_thrust::{LowThrustLog, LowThrustRecord};
pub use thrust::{mass_flow_rate, thrust_acceleration, ManeuverError, ThrustArc, G0_M_PER_S2};

// Convenience re-exports of the upstream canonical types so consumers can
// import a single namespace.
#[doc(no_inline)]
pub use siderust::astro::dynamics::{
    forces::{CompositeForce, ForceModel, TwoBody, J2},
    propagate_stm, DynamicsContext, IntegratorChoice, OrbitState, Position, Propagator,
    PropagatorConfig, StateTransitionMatrix, Velocity,
};
