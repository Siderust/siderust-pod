//! Gravity-field provider trait.
//!
//! Exposes `(C_nm, S_nm)` Stokes coefficients and the field constants
//! `(GM, R)`. The default implementation is the spherical two-body field
//! (degree 0 only). Real geopotential models (EGM2008, EIGEN-6C, …) are
//! plugged in by `siderust-pod-io::gravity`.

use crate::error::Result;
use qtty::length::Kilometers;

/// Gravity-field constants.
#[derive(Debug, Clone, Copy)]
pub struct GravityConstants {
    /// `GM = G·M_central`, in km³/s².
    pub gm_km3_s2: f64,
    /// Equatorial reference radius of the field, in km.
    pub radius_km: Kilometers,
    /// Maximum degree available.
    pub max_degree: u16,
}

/// Provider returning normalised geopotential coefficients.
pub trait GravityFieldProvider: Send + Sync {
    /// Field constants.
    fn constants(&self) -> GravityConstants;

    /// Normalised cosine coefficient `C_{n,m}`.
    fn c_nm(&self, n: u16, m: u16) -> Result<f64>;

    /// Normalised sine coefficient `S_{n,m}`.
    fn s_nm(&self, n: u16, m: u16) -> Result<f64>;
}

/// Trivial two-body gravity field. `GM = 398600.4418 km³/s²`, `R = 6378.137 km`.
#[derive(Debug, Clone, Copy)]
pub struct TwoBodyEarth;

impl GravityFieldProvider for TwoBodyEarth {
    fn constants(&self) -> GravityConstants {
        GravityConstants {
            gm_km3_s2: 398_600.441_8,
            radius_km: Kilometers::new(6_378.137),
            max_degree: 0,
        }
    }
    fn c_nm(&self, n: u16, _m: u16) -> Result<f64> {
        Ok(if n == 0 { 1.0 } else { 0.0 })
    }
    fn s_nm(&self, _n: u16, _m: u16) -> Result<f64> {
        Ok(0.0)
    }
}
