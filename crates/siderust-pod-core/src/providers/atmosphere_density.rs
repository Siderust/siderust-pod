// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Atmosphere-density provider.
//!
//! Re-exports the canonical types from `siderust::astro::dynamics::atmosphere`.

pub use siderust::astro::dynamics::atmosphere::{
    ConstantDensity, DensityProvider, ExponentialAtmosphere,
};
