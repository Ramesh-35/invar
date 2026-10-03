//! Deterministic reporting.
//!
//! Emits two artifacts per run: a machine-readable `run_results.json` and a
//! human-readable `EPISTEMIC_REPORT.md`. The report is written to be impossible
//! to read as an over-claim: on FIXTURE evidence it can rise no higher than
//! `CATEGORY_RESIDUE_CANDIDATE`, and it carries an explicit integrity ledger
//! declaring what it does NOT assert.

use serde::Serialize;

use crate::evaluator::{CaseResult, LegState};
use crate::manifest::{EpistemicStatus, EvidenceTier};

pub const ENGINE_VERSION: &str = "invar-falsify 0.1.0-alpha";

#[derive(Debug, Clone, Serialize)]
pub struct RunReport {
    pub run_id: String,
    pub timestamp_us: i64,
    pub engine_version: String,
    pub manifest_path: String,
    pub manifest_hash: String,
    pub composite_path: String,
    pub cases: Vec<CaseResult>,
}

impl RunReport {
    fn count(&self, status: EpistemicStatus) -> usize {
        self.cases.iter().filter(|c| c.status == status).count()
    }

    fn has_real_residue(&self) -> bool {
        self.cases.iter().any(|c| {
            c.status == EpistemicStatus::PROVEN_RESIDUE && c.evidence_tier == EvidenceTier::REAL
        })
    }

    fn has_residue(&self) -> bool {
        self.count(EpistemicStatus::PROVEN_RESIDUE) > 0
    }

    /// The category-level status. Deliberately incapable of emitting any
    /// "VALIDATED"/"PROVEN CATEGORY" wording on fixture-only evidence.
    pub fn category_status(&self) -> String {
        if !self.has_residue() {
            "HYPOTHESIS_UNTESTED".to_string()
        } else if self.has_real_residue() {
            // Even a REAL-tier residue in Phase 1 is a candidate pending a probe.
            "CATEGORY_RESIDUE_CANDIDATE (REAL evidence present — requires probe confirmation)"
                .to_string()
        } else {
            "CATEGORY_RESIDUE_CANDIDATE (FIXTURE tier only)".to_string()
        }
    }

    /// Run-level conservation. Two properties, both must hold:
    ///   1. the three verdict buckets partition every case (nothing dropped);
    ///   2. every case's composite-coverage accounting balances.
    /// Property 1 is trivially true for three total variants, but asserting it
    /// guards against a future verdict state silently escaping the partition,
    /// and property 2 is the per-case residual conservation.
    pub fn conservation_holds(&self) -> bool {
        let total = self.cases.len();
        let partitioned = self.count(EpistemicStatus::CAUGHT_BY_COMPOSITE)
            + self.count(EpistemicStatus::PROVEN_RESIDUE)
            + self.count(EpistemicStatus::UNKNOWN);
        partitioned == total && self.cases.iter().all(|c| c.conservation.holds())
    }

    /// Serialize the run artifact (without any self-referential hash).
    pub fn to_results_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// BLAKE3 hex over the exact run-artifact bytes, for replicability.
    pub fn replicability_hash(results_json: &str) -> String {
        blake3::hash(results_json.as_bytes()).to_hex().to_string()
    }

    pub fn to_markdown(&self, replicability_hash: &str) -> String {
        let total = self.cases.len();
        let mut out = String::new();

        out.push_str("# EPISTEMIC REPORT\n\n");
        out.push_str(&format!("- **Run ID:** `{}`\n", self.run_id));
        out.push_str(&format!("- **Timestamp (µs since epoch):** {}\n", self.timestamp_us));
        out.push_str(&format!("- **Engine:** {}\n", self.engine_version));
        out.push_str(&format!(
            "- **Manifest:** `{}` (BLAKE3 `{}`)\n",
            self.manifest_path, self.manifest_hash
        ));
        out.push_str(&format!("- **Composite config:** `{}`\n", self.composite_path));
        out.push_str(&format!("- **Replicability hash (BLAKE3 of run artifact):** `{}`\n\n", replicability_hash));

        out.push_str("## Category status\n\n");
        out.push_str(&format!("**{}**\n\n", self.category_status()));
        out.push_str(
            "> This is a falsification instrument. A `CATEGORY_RESIDUE_CANDIDATE` on FIXTURE \
             evidence is NOT a proof that INVAR is a distinct category of tool. It only records \
             that, for a hand-built fixture, a fully-executed composite baseline failed to \
             isolate the failure. Promotion requires a REAL cross-repository incident and a \
             confirming differential probe (see docs/decisions/DR-003).\n\n",
        );

        out.push_str("## Case summary\n\n");
        if total == 0 {
            out.push_str("_No cases evaluated._\n\n");
        } else {
            out.push_str(&format!(
                "- Total cases: {}\n- CAUGHT_BY_COMPOSITE: {}\n- PROVEN_RESIDUE: {}\n- UNKNOWN: {}\n\n",
                total,
                self.count(EpistemicStatus::CAUGHT_BY_COMPOSITE),
                self.count(EpistemicStatus::PROVEN_RESIDUE),
                self.count(EpistemicStatus::UNKNOWN),
            ));
        }

        out.push_str("## Per-case detail\n\n");
        for c in &self.cases {
            out.push_str(&format!("### {} - {:?}\n\n", c.id, c.status));
            out.push_str(&format!("_{}_\n\n", c.description));
            out.push_str(&format!(
                "- failure_type: `{:?}` | evidence tier: `{:?}` | cross-repository: `{}`\n",
                c.failure_type, c.evidence_tier, c.cross_repository
            ));
            out.push_str(&format!("- rationale: {}\n", c.rationale));
            out.push_str(&format!(
                "- coverage (conservation-with-residual): {}/{} composite leg(s) executed; residual (uncovered, NOT cleared by any catch): [{}]\n",
                c.conservation.executed_legs,
                c.conservation.total_legs,
                c.conservation.residual_legs.join(", ")
            ));
            out.push_str("- composite legs:\n");
            for leg in &c.legs {
                let caught = if leg.caught_designated {
                    "  [CAUGHT designated failure]"
                } else {
                    ""
                };
                let state = leg_state_str(leg.state);
                out.push_str(&format!(
                    "  - `{}`: {} - {}{}\n",
                    leg.tool, state, leg.summary, caught
                ));
            }
            if let Some(expected) = c.expected_status {
                out.push_str(&format!(
                    "- a-priori hypothesis (NOT used by the evaluator): `{:?}`\n",
                    expected
                ));
            }
            out.push('\n');
        }

        out.push_str("## Epistemic integrity ledger\n\n");
        out.push_str("This run asserts, structurally:\n\n");
        out.push_str("- [x] No CBR-001 market-regime detector result was transferred onto AST/source analysis.\n");
        out.push_str("- [x] No performance/security/zero-knowledge benchmark was manufactured.\n");
        out.push_str("- [x] No fabricated tool verdict: `SKIPPED` and `ERROR` legs were counted as neither PASS nor \"caught nothing\".\n");
        out.push_str("- [x] No verdict was assigned from a case's `failure_type` or `expected_status`.\n");
        out.push_str("- [x] FIXTURE evidence was not promoted to a proven category claim.\n");
        out.push_str(&format!(
            "- [{}] Conservation holds: the three verdict buckets partition all {} case(s) and every case's composite-coverage residual is preserved, never cleared by a catch.\n\n",
            if self.conservation_holds() { "x" } else { " " },
            self.cases.len()
        ));
        out.push_str(&format!(
            "Leg-state legend: `PASS`/`FAIL` = executed; `SKIPPED` = did not run (tool absent/offline); \
             `ERROR` = ran but uninterpretable; `UNKNOWN` = executed but irreducible. \
             Only executed legs contribute to a verdict.\n",
        ));

        out
    }
}

fn leg_state_str(state: LegState) -> &'static str {
    match state {
        LegState::PASS => "PASS",
        LegState::FAIL => "FAIL",
        LegState::SKIPPED => "SKIPPED",
        LegState::ERROR => "ERROR",
        LegState::UNKNOWN => "UNKNOWN",
    }
}
