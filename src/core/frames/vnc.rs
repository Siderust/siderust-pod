//! Velocity / Normal / Co-normal local orbital frame.
//!
//! Re-export of the canonical [`siderust`] marker so POD code does not
//! redefine it.
//!
//! - **V**: along the velocity vector
//! - **N**: orbit normal, = normalise(r×v)
//! - **C**: co-normal, = V×N
//!
//! # Examples
//!
//! ```
//! use siderust_pod::core::frames::vnc::VNC;
//! assert_eq!(std::mem::size_of::<VNC>(), 0);
//! ```

pub use siderust::astro::dynamics::frames::VNC;
