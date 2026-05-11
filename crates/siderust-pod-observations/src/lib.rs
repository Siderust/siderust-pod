//! # siderust-pod observation models
//!
//! ## Scientific scope
//!
//! This crate houses the measurement equations that connect estimated
//! spacecraft states to tracked observables. The current scientific focus
//! is GNSS code and carrier data with a minimal SLR model, enough to
//! exercise short-arc POD and validation workflows.
//!
//! The measurement physics is intentionally compact: only the terms needed
//! by the current MVP pipelines are modelled, and more elaborate
//! atmosphere, relativity, and bias treatments are deferred to later
//! milestones.
//!
//! ## Technical scope
//!
//! The crate re-exports the generic `MeasurementModel` abstraction together
//! with GNSS and SLR observation/model structs and the frame-transform
//! provider trait. These APIs return scalar predictions plus partial
//! derivatives, ready for assembly by the estimation layer.
//!
//! Parsing of raw data files and orchestration of estimation loops are
//! outside this crate.
//!
//! ## References
//!
//! - Misra, P., & Enge, P. (2012). Global Positioning System: Signals,
//!   Measurements, and Performance (2nd ed.). Ganga-Jamuna Press.
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod frame_transform;
pub mod gnss;
pub mod model;
pub mod slr;

pub use frame_transform::{FrameTransformError, FrameTransformProvider};
pub use gnss::{CarrierPhaseObs, GnssCarrierModel, GnssCodeModel, PseudorangeObs};
pub use model::{MeasurementModel, Partials, Prediction};
pub use slr::{SlrRangeModel, SlrRangeObs};
