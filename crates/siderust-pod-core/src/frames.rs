// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Compatibility shim: local orbital frames moved upstream into
//! [`siderust::astro::dynamics::frames`].
//!
//! New code should depend on the upstream module directly.  This file
//! preserves the previous public surface (`RTN`, `VNC`, `LVLH`,
//! `Rotation3`, `*_from_state`, `rotate_gcrs_to_*`) so that existing
//! callers keep compiling.

pub use affn::Rotation3;
pub use siderust::astro::dynamics::frames::{LocalOrbitalFrame, LVLH, RTN, VNC};

use affn::cartesian::Displacement;
use siderust::astro::dynamics::frames::{LVLH as LvlhMarker, RTN as RtnMarker, VNC as VncMarker};
use siderust::astro::dynamics::OrbitState;
use siderust::coordinates::frames::GCRS;
use siderust::qtty::Kilometer;

/// Build the GCRS → RTN rotation matrix from an orbit state.
#[inline]
pub fn rtn_from_state(s: &OrbitState) -> Rotation3 {
    LocalOrbitalFrame::<RtnMarker>::from_state(s).rotation()
}

/// Build the GCRS → VNC rotation matrix from an orbit state.
#[inline]
pub fn vnc_from_state(s: &OrbitState) -> Rotation3 {
    LocalOrbitalFrame::<VncMarker>::from_state(s).rotation()
}

/// Build the GCRS → LVLH rotation matrix from an orbit state.
#[inline]
pub fn lvlh_from_state(s: &OrbitState) -> Rotation3 {
    LocalOrbitalFrame::<LvlhMarker>::from_state(s).rotation()
}

/// Rotate a GCRS displacement into the RTN frame of `state`.
#[inline]
pub fn rotate_gcrs_to_rtn(
    state: &OrbitState,
    v: Displacement<GCRS, Kilometer>,
) -> Displacement<RtnMarker, Kilometer> {
    LocalOrbitalFrame::<RtnMarker>::from_state(state).to_local(v)
}

/// Rotate a GCRS displacement into the VNC frame of `state`.
#[inline]
pub fn rotate_gcrs_to_vnc(
    state: &OrbitState,
    v: Displacement<GCRS, Kilometer>,
) -> Displacement<VncMarker, Kilometer> {
    LocalOrbitalFrame::<VncMarker>::from_state(state).to_local(v)
}

/// Rotate a GCRS displacement into the LVLH frame of `state`.
#[inline]
pub fn rotate_gcrs_to_lvlh(
    state: &OrbitState,
    v: Displacement<GCRS, Kilometer>,
) -> Displacement<LvlhMarker, Kilometer> {
    LocalOrbitalFrame::<LvlhMarker>::from_state(state).to_local(v)
}
