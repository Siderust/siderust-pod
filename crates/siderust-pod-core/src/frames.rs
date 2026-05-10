// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Local orbital frame markers re-exported from
//! [`siderust::astro::dynamics::frames`].
//!
//! Use [`siderust::astro::dynamics::frames::LocalOrbitalFrame`] directly
//! to build orbit-relative rotation objects.

pub use siderust::astro::dynamics::frames::{LocalOrbitalFrame, LVLH, RTN, VNC};
pub use affn::Rotation3;
