//! Local-Vertical / Local-Horizontal frame.
//!
//! Re-export of the canonical [`siderust`] marker so POD code does not
//! redefine it.
//!
//! - **Z**: radial inward (= −R̂)
//! - **X**: velocity projected onto the local horizontal
//! - **Y**: = Z×X
//!
//! # Examples
//!
//! ```
//! use siderust_pod::core::frames::lvlh::LVLH;
//! assert_eq!(std::mem::size_of::<LVLH>(), 0);
//! ```

pub use siderust::astro::dynamics::frames::LVLH;
