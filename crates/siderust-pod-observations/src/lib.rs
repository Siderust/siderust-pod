//! `siderust-pod-observations` — measurement models for POD.
//!
//! M3 status: GNSS pseudorange and carrier-phase prediction with float
//! ambiguities and receiver clock bias. SLR / DORIS / VLBI are M5+.
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
