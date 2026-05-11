//! Run-level provenance.
//!
//! [`RunManifest`] captures everything needed to reproduce or audit a POD
//! run: tool version, configuration hash, hashed inputs, hashed outputs,
//! and the wall-clock interval. JSON serialisation is deterministic
//! (sorted keys, no incidental ordering) so two identical runs produce
//! byte-identical manifests.

use crate::dataset::DatasetRef;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Reproducible record of a single POD run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RunManifest {
    /// Free-form identifier (e.g. UUID) for the run.
    pub run_id: String,
    /// Tool version string (e.g. `"siderust-pod 0.0.0"`).
    pub tool_version: String,
    /// Hex SHA-256 of the canonical run configuration document.
    pub config_sha256: String,
    /// Inputs consumed by the run, sorted by `(kind, path)` for determinism.
    pub inputs: Vec<DatasetRef>,
    /// Artifacts produced by the run, sorted by `(kind, path)` for determinism.
    pub outputs: Vec<DatasetRef>,
    /// RFC-3339 UTC timestamp of run start.
    pub started_at: String,
    /// RFC-3339 UTC timestamp of run end.
    pub finished_at: String,
}

impl RunManifest {
    /// Sort inputs and outputs into the canonical order. Call before serialising.
    pub fn canonicalize(&mut self) {
        let key = |d: &DatasetRef| (d.kind.clone(), d.path.clone());
        self.inputs.sort_by_key(key);
        self.outputs.sort_by_key(&key);
    }

    /// Serialize to a deterministic pretty-printed JSON string.
    #[cfg(feature = "serde")]
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        let mut clone = self.clone();
        clone.canonicalize();
        serde_json::to_string_pretty(&clone)
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample() -> RunManifest {
        RunManifest {
            run_id: "00000000-0000-0000-0000-000000000000".into(),
            tool_version: "siderust-pod test".into(),
            config_sha256: "0".repeat(64),
            inputs: vec![
                DatasetRef {
                    path: PathBuf::from("/in/b.sp3"),
                    kind: "sp3".into(),
                    bytes: 1,
                    sha256: "1".repeat(64),
                },
                DatasetRef {
                    path: PathBuf::from("/in/a.sp3"),
                    kind: "sp3".into(),
                    bytes: 1,
                    sha256: "2".repeat(64),
                },
            ],
            outputs: vec![],
            started_at: "2026-05-11T22:00:00Z".into(),
            finished_at: "2026-05-11T22:00:01Z".into(),
        }
    }

    #[test]
    fn json_round_trip_is_deterministic() {
        let s1 = sample().to_json_pretty().unwrap();
        let s2 = sample().to_json_pretty().unwrap();
        assert_eq!(s1, s2);
        // Canonicalisation sorts inputs by (kind, path).
        let parsed: RunManifest = serde_json::from_str(&s1).unwrap();
        assert_eq!(parsed.inputs[0].path.to_str().unwrap(), "/in/a.sp3");
    }
}
