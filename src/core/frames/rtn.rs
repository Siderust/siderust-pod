//! Radial / Transverse / Normal (also called RIC) local orbital frame.
//!
//! Re-export of the canonical [`siderust`] marker so POD code does not
//! redefine it.
//!
//! - **R**: along the position vector (radial)
//! - **T**: transverse, = N×R
//! - **N**: orbit normal, = normalise(r×v)
//!
//! # Examples
//!
//! ```
//! use siderust_pod::core::frames::rtn::RTN;
//! assert_eq!(std::mem::size_of::<RTN>(), 0);
//! ```

pub use siderust::astro::dynamics::frames::RTN;
