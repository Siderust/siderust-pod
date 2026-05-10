// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Frame-transform provider trait.
//!
//! Encapsulates the full ITRF↔GCRF chain (precession, nutation, Earth rotation
//! angle, polar motion). Default implementations are wired in once the EOP
//! source is bound.

use affn::Rotation3;
use siderust::time::JulianDate;
use thiserror::Error;

/// Error returned by [`FrameTransformProvider`].
#[derive(Debug, Error)]
pub enum FrameTransformError {
    /// The provider could not compute the rotation at the requested epoch.
    #[error("frame transform failed: {0}")]
    Failed(String),
}

/// Provider for frame rotations between Earth-fixed and inertial frames.
///
/// The `at(jd_tt, jd_ut1)` API takes both TT and UT1 because the precession-
/// nutation rotation is parameterised by TT while ERA is parameterised by UT1.
pub trait FrameTransformProvider: Send + Sync {
    /// Rotation from ITRF to GCRF at the given TT/UT1 epoch.
    fn itrf_to_gcrf(
        &self,
        jd_tt: JulianDate,
        jd_ut1: JulianDate,
    ) -> Result<Rotation3, FrameTransformError>;

    /// Rotation from GCRF to ITRF at the given TT/UT1 epoch.
    fn gcrf_to_itrf(
        &self,
        jd_tt: JulianDate,
        jd_ut1: JulianDate,
    ) -> Result<Rotation3, FrameTransformError> {
        Ok(self.itrf_to_gcrf(jd_tt, jd_ut1)?.inverse())
    }
}
