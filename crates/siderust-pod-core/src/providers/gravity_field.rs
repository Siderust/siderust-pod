// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Gravity-field provider.
//!
//! Re-exports the canonical types from `siderust::astro::dynamics::gravity`.

pub use siderust::astro::dynamics::gravity::{
    GravityConstants, GravityFieldProvider, TwoBodyEarth,
};
