// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Compatibility shim: 6×6 covariance utilities moved upstream into
//! [`siderust::astro::dynamics::covariance`].
//!
//! New code should use [`siderust::astro::dynamics::covariance::StateCovariance`]
//! (frame-tagged) directly.  This module preserves the previous untyped
//! function-based API so the older callers keep compiling.

pub use siderust::astro::dynamics::covariance::{
    block_diag, similarity, Covariance6, StateCovariance,
};

use siderust::astro::dynamics::covariance::StateCovariance as TypedCovariance;
use siderust::astro::dynamics::frames::{LocalOrbitalFrame, RTN};
use siderust::astro::dynamics::OrbitState;
use siderust::coordinates::frames::GCRS;

/// Identity 6×6 covariance.
#[inline]
pub fn identity6() -> Covariance6 {
    TypedCovariance::<GCRS>::identity().into_matrix()
}

/// Convert a Cartesian inertial covariance to RTN at the given state.
pub fn cartesian_to_rtn(state: &OrbitState, p_inertial: &Covariance6) -> Covariance6 {
    let f = LocalOrbitalFrame::<RTN>::from_state(state);
    TypedCovariance::<GCRS>::from_matrix(*p_inertial)
        .transform_into::<RTN>(&f)
        .into_matrix()
}

/// Convert an RTN covariance back to Cartesian inertial.
pub fn rtn_to_cartesian(state: &OrbitState, p_rtn: &Covariance6) -> Covariance6 {
    let f = LocalOrbitalFrame::<RTN>::from_state(state);
    TypedCovariance::<RTN>::from_matrix(*p_rtn)
        .transform_into_inertial(&f)
        .into_matrix()
}
