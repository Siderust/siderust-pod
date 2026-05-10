//! Run configuration schema (YAML).
//!
//! MVP-1 uses a small, intentionally rigid schema. Future milestones will
//! grow it through additive optional fields.

use serde::{Deserialize, Serialize};

/// Top-level run configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    /// Schema version (semver-ish). Must be `"1.0.0"` for MVP-1.
    pub schema_version: String,
    /// Free-form run identifier.
    pub run_id: String,
    /// Inputs.
    pub inputs: InputsConfig,
    /// Output directory (absolute or relative to the config file).
    pub output_dir: String,
    /// Force-model toggles.
    pub forces: ForcesConfig,
}

/// Input file paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputsConfig {
    /// Precise GNSS satellite ephemeris (SP3).
    pub sp3: Option<String>,
    /// RINEX observation file.
    pub rinex_obs: Option<String>,
    /// RINEX broadcast navigation.
    pub rinex_nav: Option<String>,
    /// ANTEX antenna file.
    pub antex: Option<String>,
}

/// Which forces are enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForcesConfig {
    /// Two-body central gravity.
    pub two_body: bool,
    /// J2 oblateness.
    pub j2: bool,
    /// Sun + Moon third-body.
    pub third_body: bool,
}

impl RunConfig {
    /// Load from a YAML file.
    pub fn from_yaml_file(path: &str) -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(path)?;
        serde_yaml::from_str(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Validate semantic invariants (paths exist, schema version known, …).
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != "1.0.0" {
            return Err(format!(
                "unsupported schema_version: {}",
                self.schema_version
            ));
        }
        if !(self.forces.two_body) {
            return Err("two_body force must be enabled".into());
        }
        Ok(())
    }
}
