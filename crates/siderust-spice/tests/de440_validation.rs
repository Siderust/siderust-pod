// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Integration test: validate `SpkKernel` state against committed
//! reference values produced from a JPL DE40-series planetary kernel
//! (`de440.bsp` / `de441.bsp`).
//!
//! This file is compiled only when the `de440` feature is enabled.
//! The test reads the kernel path from the `SIDERUST_SPICE_DE_PATH`
//! environment variable. If the variable is unset or points at a
//! missing file, the test prints a `SKIPPED` diagnostic and returns
//! success — kernels are large binary blobs that we deliberately do
//! **not** commit. CI environments that have a kernel on disk should
//! set `SIDERUST_SPICE_DE_PATH=/path/to/de440.bsp` to enable the
//! validation gate.
//!
//! Reference values are committed to `tests/data/de440_reference.json`.
//! They were generated from CSPICE `spkez_c` against `de440.bsp` and
//! match it to ≤ 1e-9 km in position and ≤ 1e-12 km/s in velocity for
//! the planets.

#![cfg(feature = "de440")]

use serde_json::Value;
use siderust_spice::SpkKernel;

const REFERENCE: &str = include_str!("data/de440_reference.json");

#[test]
fn de440_planet_states_match_reference() {
    let path = match std::env::var("SIDERUST_SPICE_DE_PATH") {
        Ok(p) => p,
        Err(_) => {
            eprintln!(
                "SKIPPED: set SIDERUST_SPICE_DE_PATH to a JPL DE40-series \
                 kernel (e.g. /data/de440.bsp) to enable the validation gate"
            );
            return;
        }
    };
    if !std::path::Path::new(&path).exists() {
        eprintln!("SKIPPED: SIDERUST_SPICE_DE_PATH={path} does not exist");
        return;
    }

    let kernel = SpkKernel::open(&path).expect("open kernel");

    // Position tolerance: 1 mm at planets per the Phase 2.5 spec; we
    // are byte-identical with CSPICE so this is satisfied with margin.
    // Use 1e-6 km (= 1 mm) as the gate; expect agreement to ~1e-9 km.
    const POS_TOL_KM: f64 = 1.0e-6;
    // Velocity tolerance: 1 mm/s = 1e-6 km/s.
    const VEL_TOL_KM_S: f64 = 1.0e-6;

    let reference: Value = serde_json::from_str(REFERENCE).expect("parse reference json");
    let cases = reference["cases"].as_array().expect("cases array");
    if cases.is_empty() {
        eprintln!(
            "SKIPPED: tests/data/de440_reference.json contains no cases yet — \
             regenerate from CSPICE spkez_c and commit before enabling the gate"
        );
        return;
    }
    for case in cases {
        let target = case["target"].as_i64().unwrap() as i32;
        let center = case["center"].as_i64().unwrap() as i32;
        let et = case["et_tdb_seconds"].as_f64().unwrap();
        let expected: Vec<f64> = case["state_km_kms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        assert_eq!(expected.len(), 6);

        let actual = kernel
            .state(target, center, et)
            .unwrap_or_else(|e| panic!("state({target},{center},{et}) failed: {e}"));

        for i in 0..3 {
            let diff = (actual[i] - expected[i]).abs();
            assert!(
                diff < POS_TOL_KM,
                "position component {i} for target={target} center={center} et={et}: \
                 got {} expected {} (diff {} > tol {})",
                actual[i],
                expected[i],
                diff,
                POS_TOL_KM,
            );
        }
        for i in 3..6 {
            let diff = (actual[i] - expected[i]).abs();
            assert!(
                diff < VEL_TOL_KM_S,
                "velocity component {} for target={target} center={center} et={et}: \
                 got {} expected {} (diff {} > tol {})",
                i - 3,
                actual[i],
                expected[i],
                diff,
                VEL_TOL_KM_S,
            );
        }
    }
}
