//! Force model trait and primitives.
//!
//! A force model evaluates an inertial-frame acceleration (km/s²) given an
//! orbit state. Forces are intentionally `f64`-only: typed wrappers live one
//! level up in the public propagator API.

pub mod drag;
pub mod j2;
pub mod srp;
pub mod third_body;
pub mod two_body;

pub use drag::ExponentialDrag;
pub use j2::J2;
pub use srp::CannonballSrp;
pub use third_body::ThirdBodySunMoon;
pub use two_body::TwoBody;

use siderust_pod_core::OrbitState;

/// Evaluate an acceleration on an inertial state.
pub trait ForceModel: Send + Sync {
    /// Acceleration in km/s², expressed in the same inertial frame as `state`.
    fn acceleration(&self, state: &OrbitState) -> [f64; 3];
}

/// Composite force: sum of N component models.
pub struct CompositeForce {
    components: Vec<Box<dyn ForceModel>>,
}

impl CompositeForce {
    /// Empty force (returns zero).
    pub fn empty() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Append a force component.
    pub fn push(mut self, f: Box<dyn ForceModel>) -> Self {
        self.components.push(f);
        self
    }
}

impl ForceModel for CompositeForce {
    fn acceleration(&self, state: &OrbitState) -> [f64; 3] {
        let mut a = [0.0; 3];
        for f in &self.components {
            let ai = f.acceleration(state);
            a[0] += ai[0];
            a[1] += ai[1];
            a[2] += ai[2];
        }
        a
    }
}
