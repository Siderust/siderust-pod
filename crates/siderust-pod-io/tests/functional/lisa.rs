use siderust_pod_core::providers::EphemerisProvider;
use siderust_pod_io::lisa::{
    LisaEphemerisProvider, LisaOrbitReader, LisaOrbitSet, LisaProviderError, LisaSpacecraftId,
};

use super::common::{assert_approx, lisa_fixture};

// ── helpers ───────────────────────────────────────────────────────────────────

fn load_sc(sc: LisaSpacecraftId) -> siderust_pod_io::lisa::LisaOrbit {
    let ext = match sc {
        LisaSpacecraftId::SC1 => "oem1",
        LisaSpacecraftId::SC2 => "oem2",
        LisaSpacecraftId::SC3 => "oem3",
    };
    let path = lisa_fixture(&format!("lisa_orbit_sample.{ext}"));
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    LisaOrbitReader::from_str(&raw, sc).expect("parse failed")
}

fn load_all() -> LisaOrbitSet {
    LisaOrbitSet {
        sc1: load_sc(LisaSpacecraftId::SC1),
        sc2: load_sc(LisaSpacecraftId::SC2),
        sc3: load_sc(LisaSpacecraftId::SC3),
    }
}

fn make_provider() -> LisaEphemerisProvider {
    LisaEphemerisProvider::new(load_all())
}

// Epoch0 J2000 seconds for 2036-02-12T12:00:00 TDB.
// Computed as (JD − 2451545.0) × 86400 = 13191.0 × 86400 = 1_139_702_400.0.
const EPOCH0_J2000S: f64 = 1_139_702_400.0;

// Spacing between fixture epochs: 2 days.
const EPOCH_STEP_S: f64 = 2.0 * 86_400.0;

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn lisa_load_three_spacecraft() {
    let set = load_all();
    assert_eq!(set.sc1.spacecraft_id, LisaSpacecraftId::SC1);
    assert_eq!(set.sc2.spacecraft_id, LisaSpacecraftId::SC2);
    assert_eq!(set.sc3.spacecraft_id, LisaSpacecraftId::SC3);
    assert_eq!(set.sc1.points.len(), 10);
    assert_eq!(set.sc2.points.len(), 10);
    assert_eq!(set.sc3.points.len(), 10);
}

#[test]
fn lisa_sc1_first_position() {
    let orbit = load_sc(LisaSpacecraftId::SC1);
    let pt = &orbit.points[0];
    assert_approx(pt.position.x().value(), 100_000_000.0, 1.0, "x");
    assert_approx(pt.position.y().value(), 50_000_000.0, 1.0, "y");
    assert_approx(pt.position.z().value(), 10_000_000.0, 1.0, "z");
}

#[test]
fn lisa_sc2_first_y_offset() {
    let orbit = load_sc(LisaSpacecraftId::SC2);
    let pt = &orbit.points[0];
    // SC2 starts with y0 = 52_500_000 km (2 500 000 km offset from SC1).
    assert_approx(pt.position.y().value(), 52_500_000.0, 1.0, "y");
}

#[test]
fn lisa_sc3_first_z_offset() {
    let orbit = load_sc(LisaSpacecraftId::SC3);
    let pt = &orbit.points[0];
    // SC3 starts with z0 = 12_500_000 km (2 500 000 km offset from SC1).
    assert_approx(pt.position.z().value(), 12_500_000.0, 1.0, "z");
}

#[test]
fn lisa_hermite_interpolation_at_one_day() {
    // Cubic Hermite is exact for linear motion.
    // At t = epoch0 + 1 day (half-way between epoch0 and epoch1 which is 2 days):
    //   p = p0 + v * 86400 = [100_864_000, 51_728_000, 10_432_000] km.
    let provider = make_provider();
    let t = EPOCH0_J2000S + 86_400.0;
    let pt = provider.state(-1001, t).expect("interpolation failed");
    assert_approx(pt.position.x().value(), 100_864_000.0, 1e-3, "x");
    assert_approx(pt.position.y().value(), 51_728_000.0, 1e-3, "y");
    assert_approx(pt.position.z().value(), 10_432_000.0, 1e-3, "z");
}

#[test]
fn lisa_hermite_velocity_at_one_day() {
    // For linear motion, velocity must be constant across the interval.
    let provider = make_provider();
    let t = EPOCH0_J2000S + 86_400.0;
    let pt = provider.state(-1001, t).expect("interpolation failed");
    assert_approx(pt.velocity.x().value(), 10.0, 1e-9, "vx");
    assert_approx(pt.velocity.y().value(), 20.0, 1e-9, "vy");
    assert_approx(pt.velocity.z().value(), 5.0, 1e-9, "vz");
}

#[test]
fn lisa_interpolation_exact_at_tabulated_epoch() {
    // Exact epoch match must return the tabulated value.
    let provider = make_provider();
    let t = EPOCH0_J2000S + EPOCH_STEP_S; // epoch index 1
    let pt = provider.state(-1001, t).expect("interpolation failed");
    // Expected: p0 + v * 2*86400
    assert_approx(
        pt.position.x().value(),
        100_000_000.0 + 10.0 * EPOCH_STEP_S,
        1e-3,
        "x",
    );
    assert_approx(
        pt.position.y().value(),
        50_000_000.0 + 20.0 * EPOCH_STEP_S,
        1e-3,
        "y",
    );
    assert_approx(
        pt.position.z().value(),
        10_000_000.0 + 5.0 * EPOCH_STEP_S,
        1e-3,
        "z",
    );
}

#[test]
fn lisa_provider_unknown_id_error() {
    let provider = make_provider();
    let err = provider.state(399, EPOCH0_J2000S).unwrap_err();
    assert!(
        matches!(err, LisaProviderError::UnknownBody(399)),
        "unexpected error variant: {err:?}"
    );
}

#[test]
fn lisa_provider_out_of_range_error() {
    let provider = make_provider();
    // Before the first epoch.
    let err = provider.state(-1001, EPOCH0_J2000S - 1.0).unwrap_err();
    assert!(
        matches!(err, LisaProviderError::OutOfRange(_, _, _)),
        "unexpected error variant: {err:?}"
    );
}

#[test]
fn lisa_naif_id_roundtrip() {
    for sc in [
        LisaSpacecraftId::SC1,
        LisaSpacecraftId::SC2,
        LisaSpacecraftId::SC3,
    ] {
        let id = sc.naif_id();
        let back = LisaSpacecraftId::from_naif_id(id);
        assert_eq!(back, Some(sc));
    }
    assert_eq!(LisaSpacecraftId::from_naif_id(0), None);
    assert_eq!(LisaSpacecraftId::from_naif_id(399), None);
}
