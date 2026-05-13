//! Local orbital frame markers used in POD covariance/residual reports.
//!
//! POD code commonly rotates state-only covariances and position residuals
//! into a *local-orbital* frame (RTN, LVLH, or VNC) for display and QC.
//! Per design §6.1 these markers exist as zero-sized affine reference-frame
//! tags so that operations consuming "an RTN displacement" cannot
//! accidentally accept a raw inertial vector.
//!
//! The marker types and their `affn`/state-driven constructors live in
//! [`siderust::astro::dynamics::frames`]; this module re-exports them
//! verbatim so POD callers depend on a single source of truth.
//!
//! # Examples
//!
//! ```
//! use siderust_pod::core::frames::{rtn::RTN, lvlh::LVLH, vnc::VNC};
//! assert_eq!(std::mem::size_of::<RTN>(), 0);
//! assert_eq!(std::mem::size_of::<LVLH>(), 0);
//! assert_eq!(std::mem::size_of::<VNC>(), 0);
//! ```

pub mod lvlh;
pub mod rtn;
pub mod vnc;

pub use lvlh::LVLH;
pub use rtn::RTN;
pub use vnc::VNC;

/// Re-export of the typed local-orbital frame wrapper from `siderust`.
///
/// Construct via the `try_from_state` constructors on
/// [`LocalOrbitalFrame`] (`<RTN>`, `<LVLH>`, `<VNC>`) — see the upstream
/// docs for the analytic basis definition and the failure modes
/// (zero-magnitude position/velocity, parallel `r ∥ v`).
pub use siderust::astro::dynamics::frames::LocalOrbitalFrame;
