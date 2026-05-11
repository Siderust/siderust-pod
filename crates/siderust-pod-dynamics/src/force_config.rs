//! Configurable force-model selection for a POD run.
//!
//! [`ForceModelConfig`] is a declarative description; instantiation into a
//! concrete `siderust::astro::dynamics` force-model composite is performed
//! by the POD service layer when a run is materialised. Keeping the
//! representation declarative lets the same configuration drive
//! deterministic logging, manifest hashing, and config-validation tools
//! without dragging the heavy numerical types into the config layer.

/// Declarative description of which force-model contributions are active.
#[derive(Debug, Clone, PartialEq)]
pub struct ForceModelConfig {
    pub two_body: bool,
    pub j2: bool,
    /// Maximum spherical-harmonics degree/order. `None` disables non-J2 harmonics.
    pub harmonics_degree: Option<u32>,
    pub third_body_sun: bool,
    pub third_body_moon: bool,
    pub drag: bool,
    pub srp: bool,
    pub relativity: bool,
    pub empirical_acceleration: bool,
}

impl Default for ForceModelConfig {
    /// MVP-1 default: two-body + J2 only.
    fn default() -> Self {
        Self {
            two_body: true,
            j2: true,
            harmonics_degree: None,
            third_body_sun: false,
            third_body_moon: false,
            drag: false,
            srp: false,
            relativity: false,
            empirical_acceleration: false,
        }
    }
}
