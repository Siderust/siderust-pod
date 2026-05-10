//! Spacecraft and orbit state primitives.
//!
//! These types are intentionally small and `repr(C)`-friendly. They carry only
//! the *minimum* information force models and integrators need; richer state
//! (including spacecraft mass, parameter blocks, and provenance) lives in
//! [`SpacecraftState`].

use siderust::time::JulianDate;

/// Cartesian inertial position + velocity in km / (km/s) in GCRF.
///
/// Units are encoded as plain `f64` here for cache-friendly inner loops; typed
/// accessors are exposed on [`SpacecraftState`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct OrbitState {
    /// Epoch (TT scale, Julian Date).
    pub epoch_tt: JulianDate,
    /// Position x, km (GCRF).
    pub rx_km: f64,
    /// Position y, km (GCRF).
    pub ry_km: f64,
    /// Position z, km (GCRF).
    pub rz_km: f64,
    /// Velocity x, km/s (GCRF).
    pub vx_km_s: f64,
    /// Velocity y, km/s (GCRF).
    pub vy_km_s: f64,
    /// Velocity z, km/s (GCRF).
    pub vz_km_s: f64,
}

impl OrbitState {
    /// Construct from explicit components.
    pub fn new(epoch_tt: JulianDate, r_km: [f64; 3], v_km_s: [f64; 3]) -> Self {
        Self {
            epoch_tt,
            rx_km: r_km[0],
            ry_km: r_km[1],
            rz_km: r_km[2],
            vx_km_s: v_km_s[0],
            vy_km_s: v_km_s[1],
            vz_km_s: v_km_s[2],
        }
    }

    /// Position vector as `[x, y, z]` km.
    #[inline]
    pub fn position_km(&self) -> [f64; 3] {
        [self.rx_km, self.ry_km, self.rz_km]
    }

    /// Velocity vector as `[vx, vy, vz]` km/s.
    #[inline]
    pub fn velocity_km_s(&self) -> [f64; 3] {
        [self.vx_km_s, self.vy_km_s, self.vz_km_s]
    }

    /// 6-vector `[r, v]` packing.
    #[inline]
    pub fn to_array6(&self) -> [f64; 6] {
        [
            self.rx_km,
            self.ry_km,
            self.rz_km,
            self.vx_km_s,
            self.vy_km_s,
            self.vz_km_s,
        ]
    }

    /// Construct from 6-vector packing at a given epoch.
    #[inline]
    pub fn from_array6(epoch_tt: JulianDate, x: [f64; 6]) -> Self {
        Self::new(epoch_tt, [x[0], x[1], x[2]], [x[3], x[4], x[5]])
    }

    /// Position magnitude squared (km²).
    #[inline]
    pub fn r2(&self) -> f64 {
        self.rx_km * self.rx_km + self.ry_km * self.ry_km + self.rz_km * self.rz_km
    }
}

/// Spacecraft properties carried alongside the orbit state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacecraftProperties {
    /// Total mass, kg.
    pub mass_kg: f64,
    /// Cross-section for drag, m².
    pub drag_area_m2: f64,
    /// Drag coefficient (dimensionless).
    pub cd: f64,
    /// Cross-section for SRP, m².
    pub srp_area_m2: f64,
    /// SRP coefficient (dimensionless).
    pub cr: f64,
}

impl SpacecraftProperties {
    /// Reasonable demo defaults for a small LEO platform.
    pub fn demo_leo() -> Self {
        Self {
            mass_kg: 500.0,
            drag_area_m2: 2.0,
            cd: 2.2,
            srp_area_m2: 2.0,
            cr: 1.3,
        }
    }
}

/// Combined spacecraft state used by the propagator and the estimator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacecraftState {
    /// Orbit state.
    pub orbit: OrbitState,
    /// Spacecraft properties.
    pub properties: SpacecraftProperties,
}
