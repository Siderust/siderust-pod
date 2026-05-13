// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Integration test: Q matrix is positive semi-definite.
//!
//! Exercises [`ProcessNoiseModel::is_psd`] for a variety of configurations
//! and representative EKF time steps.

use siderust::qtty::{KmPerSecondsSquared, Second};
use siderust_pod_dynamics::{GaussMarkovParams, ProcessNoiseModel, WhiteAccelPsd};

/// White-only PSD (6×6 block) is PSD for a range of dt values.
#[test]
fn white_accel_only_is_psd() {
    let m = ProcessNoiseModel {
        position_velocity: WhiteAccelPsd::isotropic(KmPerSecondsSquared::new(1e-9)),
        drag_scale: None,
        srp_scale: None,
        empirical_white: None,
    };
    for dt_s in [1.0, 10.0, 60.0, 300.0, 3600.0] {
        assert!(
            m.is_psd(Second::new(dt_s)).unwrap(),
            "Q not PSD for dt = {dt_s} s (white-only)"
        );
    }
}

/// Model with Gauss–Markov drag-scale and SRP-scale terms (8×8 block).
#[test]
fn drag_srp_gauss_markov_is_psd() {
    let m = ProcessNoiseModel {
        position_velocity: WhiteAccelPsd::isotropic(KmPerSecondsSquared::new(1e-9)),
        drag_scale: Some(GaussMarkovParams {
            sigma: 0.1,
            tau: Second::new(3_600.0),
        }),
        srp_scale: Some(GaussMarkovParams {
            sigma: 0.05,
            tau: Second::new(7_200.0),
        }),
        empirical_white: None,
    };
    for dt_s in [1.0, 60.0, 600.0, 3600.0] {
        assert!(
            m.is_psd(Second::new(dt_s)).unwrap(),
            "Q not PSD for dt = {dt_s} s (drag+SRP)"
        );
    }
}

/// Model with white empirical RTN accelerations (9×9 block).
#[test]
fn empirical_white_is_psd() {
    let sigma = KmPerSecondsSquared::new(1e-8);
    let m = ProcessNoiseModel {
        position_velocity: WhiteAccelPsd::isotropic(KmPerSecondsSquared::new(1e-9)),
        drag_scale: None,
        srp_scale: None,
        empirical_white: Some([sigma, sigma, sigma]),
    };
    for dt_s in [1.0, 30.0, 300.0] {
        assert!(
            m.is_psd(Second::new(dt_s)).unwrap(),
            "Q not PSD for dt = {dt_s} s (empirical)"
        );
    }
}

/// Full model (drag + SRP + empirical, 11×11 block).
#[test]
fn full_model_is_psd() {
    let sigma_e = KmPerSecondsSquared::new(1e-8);
    let m = ProcessNoiseModel {
        position_velocity: WhiteAccelPsd::isotropic(KmPerSecondsSquared::new(1e-9)),
        drag_scale: Some(GaussMarkovParams {
            sigma: 0.15,
            tau: Second::new(1_800.0),
        }),
        srp_scale: Some(GaussMarkovParams {
            sigma: 0.08,
            tau: Second::new(3_600.0),
        }),
        empirical_white: Some([sigma_e, sigma_e, sigma_e]),
    };
    assert_eq!(m.dim(), 11);
    for dt_s in [1.0, 60.0, 600.0, 3600.0, 86400.0] {
        assert!(
            m.is_psd(Second::new(dt_s)).unwrap(),
            "Q not PSD for dt = {dt_s} s (full)"
        );
    }
}

/// dt = 0 yields a zero matrix, which is PSD (boundary case).
#[test]
fn zero_dt_is_psd() {
    let m = ProcessNoiseModel {
        position_velocity: WhiteAccelPsd::isotropic(KmPerSecondsSquared::new(1e-9)),
        drag_scale: Some(GaussMarkovParams {
            sigma: 0.1,
            tau: Second::new(3600.0),
        }),
        srp_scale: None,
        empirical_white: None,
    };
    assert!(m.is_psd(Second::new(0.0)).unwrap());
}
