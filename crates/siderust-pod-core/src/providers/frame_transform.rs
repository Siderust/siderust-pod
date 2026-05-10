// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Frame-transform provider trait.
//!
//! Encapsulates the full ITRF↔GCRF chain (precession, nutation, Earth rotation
//! angle, polar motion). Default implementations are wired in
//! `siderust-pod-dynamics::frames` once the EOP source is bound.

use crate::error::Result;
use affn::Rotation3;
use siderust::time::JulianDate;

/// Provider for frame rotations between Earth-fixed and inertial frames.
///
/// The `at(jd_tt, jd_ut1)` API takes both TT and UT1 because the precession-
/// nutation rotation is parameterised by TT while ERA is parameterised by UT1.
pub trait FrameTransformProvider: Send + Sync {
    /// Rotation from ITRF to GCRF at the given TT/UT1 epoch.
    fn itrf_to_gcrf(&self, jd_tt: JulianDate, jd_ut1: JulianDate) -> Result<Rotation3>;

    /// Rotation from GCRF to ITRF at the given TT/UT1 epoch.
    fn gcrf_to_itrf(&self, jd_tt: JulianDate, jd_ut1: JulianDate) -> Result<Rotation3> {
        Ok(self.itrf_to_gcrf(jd_tt, jd_ut1)?.inverse())
    }
}
