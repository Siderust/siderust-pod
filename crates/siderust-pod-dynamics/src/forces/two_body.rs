//! Compatibility shim: the canonical `TwoBody` model now lives upstream
//! in [`siderust::astro::dynamics::forces::TwoBody`].

pub use siderust::astro::dynamics::forces::TwoBody;
