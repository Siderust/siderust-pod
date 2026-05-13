//! Spacecraft state and physical-properties re-exports.
//!
//! Per the design (`docs/design/siderust_pod_detailed_design_document.md`,
//! §6.1) the canonical spacecraft state — orbit + mass + attitude +
//! physical properties — already lives in `siderust`
//! ([`siderust::astro::dynamics::state::SpacecraftState`]). It is
//! re-exported here so POD code can build on the same typed primitive
//! without duplicating the model.
//!
//! # Examples
//!
//! ```
//! use siderust_pod_core::spacecraft::SpacecraftProperties;
//! // `SpacecraftProperties` carries area / mass / drag / SRP coefficients
//! // backed by typed `qtty` quantities — see siderust's docs for fields.
//! let _ = std::mem::size_of::<SpacecraftProperties>();
//! ```

pub use siderust::astro::dynamics::state::{SpacecraftProperties, SpacecraftState};
