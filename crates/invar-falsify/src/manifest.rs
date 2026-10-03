//! Corpus manifest types.
//!
//! The manifest is a-priori data: it records what a human *hypothesised* about
//! each case (`expected_status`) alongside the facts the evaluator needs
//! (`failure_type`, `target_path`, `catch_signature`). The evaluator uses the
//! facts and pointedly ignores the hypothesis.

use serde::{Deserialize, Serialize};

/// The kind of failure a corpus case embodies. SCREAMING_SNAKE on the wire.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureType {
    AST_SCHEMA_DESYNC,
    RUNTIME_STATE_LEAK,
    CONCURRENCY_INVARIANT,
    TEMPORAL_DRIFT,
}

/// Mutually exclusive epistemic verdict for a case. These are the ONLY three
/// states a case may resolve to. There is deliberately no "PASS"/"VALIDATED".
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpistemicStatus {
    /// The composite baseline demonstrably isolated the designated failure.
    CAUGHT_BY_COMPOSITE,
    /// The composite fully ran on this case and did NOT catch it. On FIXTURE
    /// evidence this is only ever a *candidate* residue, never a proven one.
    PROVEN_RESIDUE,
    /// We cannot make either claim honestly — e.g. a required leg did not run.
    UNKNOWN,
}

/// Provenance of the evidence backing a case. FIXTURE evidence can never, by
/// itself, promote a category claim; only REAL evidence can.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceTier {
    FIXTURE,
    REAL,
}

impl Default for EvidenceTier {
    fn default() -> Self {
        EvidenceTier::FIXTURE
    }
}

/// One corpus case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub description: String,
    pub cross_repository: bool,
    pub failure_type: FailureType,
    #[serde(default)]
    pub evidence_tier: EvidenceTier,
    pub target_path: String,
    /// A substring the composite's finding IDs must contain for this case to
    /// count as *caught*. `None` means: no local rule claims to isolate this
    /// case, so a mere advisory finding must not be read as a catch.
    #[serde(default)]
    pub catch_signature: Option<String>,
    /// The a-priori human hypothesis. The evaluator MUST NOT read this to
    /// decide a verdict; it exists only for post-hoc comparison in reports.
    #[serde(default)]
    pub expected_status: Option<EpistemicStatus>,
}

/// The top-level corpus manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub corpus_version: String,
    pub description: String,
    pub cases: Vec<TestCase>,
}

/// Per-tool declaration from `composite.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolConfig {
    pub enabled: bool,
    pub bin_path: Option<String>,
    pub rules: Option<String>,
}

impl Default for ToolConfig {
    fn default() -> Self {
        ToolConfig {
            enabled: true,
            bin_path: None,
            rules: None,
        }
    }
}

/// The composite baseline configuration (declarations only — presence here does
/// NOT assert the binary exists on this machine).
#[derive(Debug, Clone, Default)]
pub struct CompositeConfig {
    pub semgrep: ToolConfig,
    pub opa: ToolConfig,
    pub opentelemetry: ToolConfig,
    pub property_testing: ToolConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("failed to read manifest {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse manifest {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("manifest {path} is invalid: {reason}")]
    Invalid { path: String, reason: String },
}

impl Manifest {
    /// Load and structurally validate a manifest file.
    pub fn load(path: &str) -> Result<Manifest, ManifestError> {
        let raw = std::fs::read_to_string(path).map_err(|source| ManifestError::Read {
            path: path.to_string(),
            source,
        })?;
        let manifest: Manifest =
            serde_json::from_str(&raw).map_err(|source| ManifestError::Parse {
                path: path.to_string(),
                source,
            })?;
        manifest.validate(path)?;
        Ok(manifest)
    }

    fn validate(&self, path: &str) -> Result<(), ManifestError> {
        if self.cases.is_empty() {
            return Err(ManifestError::Invalid {
                path: path.to_string(),
                reason: "manifest declares zero cases".to_string(),
            });
        }
        for case in &self.cases {
            if case.id.trim().is_empty() {
                return Err(ManifestError::Invalid {
                    path: path.to_string(),
                    reason: "a case has an empty id".to_string(),
                });
            }
            if !case.target_path.starts_with("./corpus/") {
                return Err(ManifestError::Invalid {
                    path: path.to_string(),
                    reason: format!(
                        "case {} target_path {:?} must live under ./corpus/",
                        case.id, case.target_path
                    ),
                });
            }
        }
        Ok(())
    }
}
