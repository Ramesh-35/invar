//! Reality-admission gate (GAP-D, structural precondition only).
//!
//! HONEST SCOPE, stated first because this is the one gate most able to lie.
//! This blocks the trivial self-promotion attack: a case may not reach
//! `EvidenceTier::REAL` merely by setting the field. REAL now requires a
//! `RealArtifactProof` whose `diff_probe_hash` BINDS to the residue actually
//! recorded for the evidence under evaluation.
//!
//! What this does NOT do, and cannot do locally and offline:
//!   * It does not verify that `producer_commit` / `consumer_commit` exist in
//!     real repositories.
//!   * It does not verify that the differential probe was actually run against
//!     them, or that `diff_probe_hash` is the hash of a real residue rather than
//!     of invented bytes.
//!
//! The hash comparison is a binding/consistency check between a proof and the
//! evidence it accompanies, NOT proof of external reality. A fabricated proof
//! whose hash matches a fabricated residue still passes this gate. True REAL
//! admission additionally requires resolving the referenced artifacts against
//! the outside world (fetching the commits, re-running the probe), which an
//! offline check cannot perform.
//!
//! Therefore this gate is a NECESSARY, not sufficient, condition for REAL, and
//! Primitive D remains PARTIAL. It is never "(synthetic) SATISFIED": the whole
//! point of D is that it cannot be closed by synthetic means.

use std::collections::BTreeSet;

use crate::manifest::EvidenceTier;

/// External-resolving step (GAP-D, run-time half).
///
/// Given a producer JSON-schema and a consumer source, return the fields the
/// consumer reads off `payload` that the producer's schema does NOT define as
/// properties. A non-empty result is the isolated cross-repository residue: a
/// contract the consumer depends on that the producer no longer provides.
///
/// This is deterministic and pure: it operates on bytes that were fetched and
/// externally resolved out of band (the commit SHAs were confirmed to resolve,
/// with a negative control). It does not itself perform network I/O, so it
/// cannot by itself attest that the bytes came from real commits; that attest
/// is the audited fetch. Honest division of labor, stated so it is not misread.
pub fn differential_residue(producer_schema_json: &str, consumer_source: &str) -> Vec<String> {
    let defined: BTreeSet<String> =
        serde_json::from_str::<serde_json::Value>(producer_schema_json)
            .ok()
            .and_then(|v| {
                v.get("properties")
                    .and_then(|p| p.as_object())
                    .map(|o| o.keys().cloned().collect())
            })
            .unwrap_or_default();

    let mut referenced: BTreeSet<String> = BTreeSet::new();
    let bytes = consumer_source.as_bytes();
    let needle = "payload.";
    let mut i = 0usize;
    while let Some(pos) = consumer_source[i..].find(needle) {
        let start = i + pos + needle.len();
        let mut end = start;
        while end < consumer_source.len() {
            let c = bytes[end];
            if c.is_ascii_alphanumeric() || c == b'_' {
                end += 1;
            } else {
                break;
            }
        }
        if end > start {
            referenced.insert(consumer_source[start..end].to_string());
        }
        i = start.max(i + pos + 1);
    }

    referenced.difference(&defined).cloned().collect()
}

/// Derive the BLAKE3 probe hash of a residue, canonicalized (sorted, newline
/// joined) so the same residue always hashes identically.
pub fn derive_probe_hash(residue: &[String]) -> [u8; 32] {
    let mut sorted = residue.to_vec();
    sorted.sort();
    *blake3::hash(sorted.join("\n").as_bytes()).as_bytes()
}

/// A claim that a residue was isolated from a specific cross-repository pair.
/// The fields are asserted by whoever supplies the proof; this module checks
/// only that `diff_probe_hash` binds to the recorded residue, never that the
/// commits are real.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RealArtifactProof {
    pub producer_repo: String,
    pub producer_commit: String,
    pub consumer_repo: String,
    pub consumer_commit: String,
    /// BLAKE3 of the isolated differential residue, as claimed by the probe.
    pub diff_probe_hash: [u8; 32],
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AdmissibilityError {
    #[error("REAL tier claimed but no RealArtifactProof was supplied")]
    MissingProof,
    #[error("REAL tier claimed but no recorded residue hash was supplied to bind the proof to")]
    MissingResidue,
    #[error("RealArtifactProof.diff_probe_hash does not bind to the recorded residue")]
    ResidueMismatch,
}

/// Admit or refuse a REAL-tier claim.
///
/// FIXTURE needs no proof and is always admitted. REAL requires BOTH a proof
/// and the residue hash recorded for the evidence under evaluation, and the
/// proof's `diff_probe_hash` must equal that residue. The residue is passed in
/// separately and must come from the evidence itself, never from the proof, so
/// the proof cannot vouch for itself. See the module header for what a pass
/// does and does not establish.
pub fn verify_real_tier(
    tier: EvidenceTier,
    proof: Option<&RealArtifactProof>,
    recorded_residue_hash: Option<[u8; 32]>,
) -> Result<(), AdmissibilityError> {
    if tier != EvidenceTier::REAL {
        return Ok(());
    }
    let proof = proof.ok_or(AdmissibilityError::MissingProof)?;
    let residue = recorded_residue_hash.ok_or(AdmissibilityError::MissingResidue)?;
    if proof.diff_probe_hash != residue {
        return Err(AdmissibilityError::ResidueMismatch);
    }
    Ok(())
}
