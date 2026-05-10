//! Ephemeris provider — thin compatibility shim over
//! [`siderust::calculus::ephemeris::DynEphemeris`].
//!
//! The `EphemerisProvider` trait that used to live here duplicated `DynEphemeris`
//! exactly; it has been replaced by a type alias so existing code keeps compiling.

pub use siderust::calculus::ephemeris::{DynEphemeris, EphemerisError, Vsop87Ephemeris};

/// Convenience re-export: the concrete VSOP87/ELP2000 ephemeris backend.
///
/// Use `Arc<Vsop87Provider>` as the default provider for production pipelines.
pub use Vsop87Ephemeris as Vsop87Provider;

/// Object-safe dynamic ephemeris used by pod force models.
///
/// This is the trait object type for [`DynEphemeris`] with the `Send + Sync`
/// bounds required by multi-threaded pipelines.
pub type BoxEphemeris = dyn DynEphemeris + Send + Sync;
