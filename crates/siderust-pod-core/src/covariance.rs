// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! 6×6 covariance utilities re-exported from
//! [`siderust::astro::dynamics::covariance`].
//!
//! Use [`StateCovariance<F>`] directly for frame-tagged covariance arithmetic.

pub use siderust::astro::dynamics::covariance::{
    block_diag, similarity, Covariance6, StateCovariance,
};
