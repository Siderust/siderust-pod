//! Compatibility shim: integrators now live upstream in
//! [`siderust::astro::dynamics::integrators`].

pub use siderust::astro::dynamics::integrators::dopri5;
pub use siderust::astro::dynamics::integrators::rk4;
pub use siderust::astro::dynamics::integrators::{
    dopri5_propagate, dopri5_step, rk4_propagate, rk4_propagate_series, rk4_step, Tolerance,
};
