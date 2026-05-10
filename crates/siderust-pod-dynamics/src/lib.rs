#![allow(clippy::needless_range_loop, clippy::inconsistent_digit_grouping)]
//! `siderust-pod-dynamics` — force models, propagation, and STM for Siderust POD.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod forces;
pub mod integrators;
pub mod stm;

pub mod prelude {
    //! Convenience re-exports.
    pub use crate::forces::{
        drag::ExponentialDrag, j2::J2, srp::CannonballSrp, third_body::ThirdBodySunMoon,
        two_body::TwoBody, CompositeForce, ForceModel,
    };
    pub use crate::integrators::{dopri5_propagate, rk4_propagate, rk4_step, Tolerance};
    pub use crate::stm::finite_diff_stm;
}

pub use forces::{CompositeForce, ForceModel};
pub use integrators::{dopri5_propagate, rk4_propagate, rk4_step, Tolerance};
pub use stm::{finite_diff_stm, finite_diff_stm_series};
