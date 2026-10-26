//! The skeptic engine.
//!
//! Runs each composite leg against a corpus case and classifies the case into
//! exactly one of the three epistemic states. The two invariants that make this
//! honest:
//!
//!   1. A leg that did not execute (`SKIPPED`) or errored (`ERROR`) contributes
//!      nothing: it is neither PASS nor "caught nothing". Only `PASS`/`FAIL`
//!      count as *executed*.
//!   2. `classify` never inspects `failure_type` or `expected_status` to decide
//!      the *polarity* of the verdict. It reads `failure_type` only to know
//!      which legs the composite must have run to make a residue claim honest
//!      (coverage), never to declare a case residue because of its type.

use std::collections::HashSet;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::Serialize;

use crate::manifest::{
    CompositeConfig, EpistemicStatus, EvidenceTier, FailureType, TestCase, ToolConfig,
};

/// The state of one composite leg after we tried to run it.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LegState {
    /// Ran, found nothing.
    PASS,
    /// Ran, found something.
    FAIL,
    /// Did not run (tool absent / disabled / not applicable offline).
    SKIPPED,
    /// Tried to run but the result is uninterpretable.
    ERROR,
    /// Reserved for a leg whose executed result cannot be reduced to PASS/FAIL.
    UNKNOWN,
    /// (GAP-E) Executed and interpretable, but NOT allowed to count: its
    /// evidence lies outside the as-of temporal bound (GAP-C), or it is a
    /// provenance-correlated duplicate carrier (GAP-B). Never a catch. Like
    /// SKIPPED/ERROR it is not "executed", so it can only degrade a verdict.
    INADMISSIBLE,
}

impl LegState {
    /// Only PASS and FAIL are "executed" - the evaluator has a real, admissible
    /// result. SKIPPED, ERROR, UNKNOWN, and INADMISSIBLE are all non-executed.
    pub fn executed(&self) -> bool {
        matches!(self, LegState::PASS | LegState::FAIL)
    }
}

/// The result of running a single composite leg against one case.
#[derive(Debug, Clone, Serialize)]
pub struct LegVerdict {
    pub tool: String,
    pub state: LegState,
    pub findings: u32,
    /// True only when this leg *executed* and produced a finding whose id
    /// contains the case's `catch_signature`. A SKIPPED/ERROR leg is always
    /// `false` here — absence of a run is never a catch.
    pub caught_designated: bool,
    pub summary: String,
}

impl LegVerdict {
    fn skipped(tool: &str, why: &str) -> Self {
        LegVerdict {
            tool: tool.to_string(),
            state: LegState::SKIPPED,
            findings: 0,
            caught_designated: false,
            summary: why.to_string(),
        }
    }

    fn error(tool: &str, why: &str) -> Self {
        LegVerdict {
            tool: tool.to_string(),
            state: LegState::ERROR,
            findings: 0,
            caught_designated: false,
            summary: why.to_string(),
        }
    }
}

/// Conservation-with-residual accounting for one case.
///
/// A catch by one composite leg does NOT mean the composite as a whole covered
/// the case: other legs may have been SKIPPED/ERROR and produced no
/// observation. This records that uncovered remainder explicitly, so a verdict
/// of `CAUGHT_BY_COMPOSITE` can never silently imply total coverage. The
/// residual is computed from leg evidence alone and is independent of the
/// verdict.
///
/// Conservation invariant (checked by `holds`):
///   `total_legs == executed_legs + residual_legs.len()`
#[derive(Debug, Clone, Serialize)]
pub struct CaseConservation {
    /// Breadth of the composite considered for this case (number of legs run).
    pub total_legs: usize,
    /// Legs that produced an executed observation (PASS/FAIL).
    pub executed_legs: usize,
    /// Legs that produced no executed observation (SKIPPED/ERROR/UNKNOWN): the
    /// uncovered remainder, preserved even when the case was caught.
    pub residual_legs: Vec<String>,
}

impl CaseConservation {
    /// The conservation equation. True by construction here; asserted in tests
    /// and in the run report so a future change cannot let coverage vanish.
    pub fn holds(&self) -> bool {
        self.total_legs == self.executed_legs + self.residual_legs.len()
    }
}

/// Compute the composite-coverage residual for a case from its legs. Pointedly
/// independent of the verdict: a `CAUGHT_BY_COMPOSITE` case whose other legs
/// were SKIPPED still has a non-empty residual.
pub fn composite_residual(legs: &[LegVerdict]) -> CaseConservation {
    let total_legs = legs.len();
    let executed_legs = legs.iter().filter(|l| l.state.executed()).count();
    let residual_legs = legs
        .iter()
        .filter(|l| !l.state.executed())
        .map(|l| l.tool.clone())
        .collect();
    CaseConservation {
        total_legs,
        executed_legs,
        residual_legs,
    }
}

/// GAP-B evaluator hook: independence-aware corroboration over evidence SOURCES.
///
/// This is the aggregation the composite will use once it carries multiple
/// evidence carriers per invariant. It is deliberately NOT called by `classify`
/// today: the Phase-1 composite legs (semgrep/opa/opentelemetry/property) are
/// distinct tools, not multiple carriers of one observation, so there is no
/// correlated evidence to de-duplicate in the live path yet. Until such evidence
/// exists this is exercised directly by the acceptance tests. Stating that
/// plainly is the point: the capability is real and tested; its live invocation
/// is not yet earned, so `classify`'s honesty core is left untouched.
pub fn corroboration_of(
    graph: &crate::provenance::ProvenanceGraph,
    source_ids: &[&str],
) -> crate::provenance::Corroboration {
    graph.corroboration(source_ids)
}

/// GAP-E temporal admissibility stage. Given per-leg evidence times and an
/// as-of bound, any executed leg whose evidence lies outside the bound
/// (`t_r > as_of_t_r` OR `t_v > as_of_t_v`) is demoted to `INADMISSIBLE` and
/// stripped of any catch. The result is fed to the UNCHANGED `classify`, so an
/// inadmissible required leg degrades the case to UNKNOWN by the existing
/// weakest-link rule: no new certainty is created, only removed.
///
/// A leg with no supplied timing is passed through unchanged (its temporal
/// admissibility is simply not asserted here). This stage is composable and
/// inert when no timing is supplied, which is the case for the metadata-free
/// Phase-1 corpus; the acceptance tests exercise the active path.
pub fn admit_temporal(
    legs: &[LegVerdict],
    leg_times: &[(String, i64, i64)],
    as_of_t_r: i64,
    as_of_t_v: i64,
) -> Vec<LegVerdict> {
    legs.iter()
        .map(|l| {
            if let Some((_, tr, tv)) = leg_times.iter().find(|(t, _, _)| *t == l.tool) {
                if *tr > as_of_t_r || *tv > as_of_t_v {
                    return LegVerdict {
                        tool: l.tool.clone(),
                        state: LegState::INADMISSIBLE,
                        findings: l.findings,
                        caught_designated: false,
                        summary: format!(
                            "executed but temporally inadmissible (t_r={tr}, t_v={tv} beyond as_of t_r={as_of_t_r}, t_v={as_of_t_v})"
                        ),
                    };
                }
            }
            l.clone()
        })
        .collect()
}

/// GAP-D reality-admission stage. If a case claims `EvidenceTier::REAL` and the
/// reality gate refuses it (no proof, no recorded residue, or a proof whose
/// hash does not bind to the recorded residue), every leg is demoted to
/// INADMISSIBLE so the case degrades to UNKNOWN through the existing weakest-
/// link path. FIXTURE cases and admitted REAL cases pass through unchanged.
///
/// This is a NECESSARY, not sufficient, gate: admission here means the proof
/// binds to the recorded residue, NOT that the referenced commits are real.
/// See `crate::reality` for the full scope limit. Like the other admissibility
/// stages it is composable and inert on the metadata-free Phase-1 corpus (all
/// FIXTURE), and is exercised directly by the acceptance tests.
pub fn admit_reality(
    legs: &[LegVerdict],
    tier: crate::manifest::EvidenceTier,
    proof: Option<&crate::reality::RealArtifactProof>,
    recorded_residue_hash: Option<[u8; 32]>,
) -> Vec<LegVerdict> {
    match crate::reality::verify_real_tier(tier, proof, recorded_residue_hash) {
        Ok(()) => legs.to_vec(),
        Err(e) => legs
            .iter()
            .map(|l| LegVerdict {
                tool: l.tool.clone(),
                state: LegState::INADMISSIBLE,
                findings: l.findings,
                caught_designated: false,
                summary: format!("reality-admission gate refused REAL tier: {e}"),
            })
            .collect(),
    }
}

/// GAP-E provenance admissibility: the corroboration standing of the EXECUTED
/// legs that support a verdict, keyed by each leg's provenance source. Carriers
/// that share ancestry collapse to one group, so correlated legs cannot earn
/// `Confident` independent corroboration; they are capped at `Banded`. Legs
/// with no supplied source are ignored for this computation.
pub fn leg_corroboration(
    legs: &[LegVerdict],
    leg_sources: &[(String, String)],
    graph: &crate::provenance::ProvenanceGraph,
) -> crate::provenance::Corroboration {
    let ids: Vec<&str> = legs
        .iter()
        .filter(|l| l.state.executed())
        .filter_map(|l| {
            leg_sources
                .iter()
                .find(|(t, _)| *t == l.tool)
                .map(|(_, s)| s.as_str())
        })
        .collect();
    graph.corroboration(&ids)
}

/// The full evaluated result for one case.
#[derive(Debug, Clone, Serialize)]
pub struct CaseResult {
    pub id: String,
    pub description: String,
    pub failure_type: FailureType,
    pub evidence_tier: EvidenceTier,
    pub cross_repository: bool,
    pub status: EpistemicStatus,
    pub rationale: String,
    pub expected_status: Option<EpistemicStatus>,
    /// Conservation-with-residual accounting: what the composite did NOT cover,
    /// recorded regardless of the verdict.
    pub conservation: CaseConservation,
    pub legs: Vec<LegVerdict>,
}

/// Which legs the composite must have *executed* for a residue claim to be
/// honest for a given failure type. This is a coverage requirement, not a
/// verdict: a missing required leg forces UNKNOWN, never residue.
pub fn required_legs(failure_type: FailureType) -> Vec<&'static str> {
    match failure_type {
        FailureType::AST_SCHEMA_DESYNC => vec!["semgrep"],
        FailureType::RUNTIME_STATE_LEAK
        | FailureType::CONCURRENCY_INVARIANT
        | FailureType::TEMPORAL_DRIFT => vec!["opentelemetry"],
    }
}

/// The classifier. Reads only the leg *evidence* and (for coverage) which legs
/// were required — never the case's declared/expected verdict.
pub fn classify(tc: &TestCase, legs: &[LegVerdict]) -> (EpistemicStatus, String) {
    // 1. Did any executed leg actually isolate the designated failure?
    if let Some(leg) = legs
        .iter()
        .find(|l| l.state.executed() && l.caught_designated)
    {
        return (
            EpistemicStatus::CAUGHT_BY_COMPOSITE,
            format!(
                "composite leg `{}` isolated the designated failure (finding matched catch_signature)",
                leg.tool
            ),
        );
    }

    // 2. No catch. Residue is only honest if every required leg actually ran.
    let executed: HashSet<&str> = legs
        .iter()
        .filter(|l| l.state.executed())
        .map(|l| l.tool.as_str())
        .collect();
    let required = required_legs(tc.failure_type);
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|r| !executed.contains(r))
        .collect();

    if missing.is_empty() {
        let tier_note = match tc.evidence_tier {
            EvidenceTier::FIXTURE => " (FIXTURE tier — residue CANDIDATE only, not proven)",
            EvidenceTier::REAL => "",
        };
        (
            EpistemicStatus::PROVEN_RESIDUE,
            format!(
                "every required composite leg ({}) executed and none isolated the failure{}",
                required.join(", "),
                tier_note
            ),
        )
    } else {
        (
            EpistemicStatus::UNKNOWN,
            format!(
                "cannot claim residue: required leg(s) [{}] did not execute (composite coverage incomplete)",
                missing.join(", ")
            ),
        )
    }
}

/// Evaluate one case end-to-end: run every leg, then classify.
pub fn evaluate_case(tc: &TestCase, composite: &CompositeConfig, timeout: Duration) -> CaseResult {
    let legs = vec![
        run_semgrep(&composite.semgrep, tc, timeout),
        run_opa(&composite.opa, tc),
        run_otel(&composite.opentelemetry, tc),
        run_property(&composite.property_testing, tc),
    ];
    let conservation = composite_residual(&legs);
    let (status, rationale) = classify(tc, &legs);
    CaseResult {
        id: tc.id.clone(),
        description: tc.description.clone(),
        failure_type: tc.failure_type,
        evidence_tier: tc.evidence_tier,
        cross_repository: tc.cross_repository,
        status,
        rationale,
        expected_status: tc.expected_status,
        conservation,
        legs,
    }
}

enum ExecOutcome {
    Completed(std::process::Output),
    NotFound,
    SpawnError(String),
    TimedOut,
}

/// Run a command to completion with a soft timeout. On timeout we report an
/// ERROR leg rather than blocking forever; the child may outlive us, which is
/// acceptable for the tiny Phase-1 corpus and never converts to PASS.
fn exec_with_timeout(bin: &str, args: Vec<String>, timeout: Duration) -> ExecOutcome {
    let mut cmd = Command::new(bin);
    cmd.args(&args);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(cmd.output());
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => ExecOutcome::Completed(output),
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => ExecOutcome::NotFound,
        Ok(Err(e)) => ExecOutcome::SpawnError(e.to_string()),
        Err(_) => ExecOutcome::TimedOut,
    }
}

/// SECURITY-SPEC S3-1: the program the semgrep leg may run. A program the
/// repository's composite.toml names runs only when the operator names the
/// same program on the command line; the tool's own name, resolved through
/// the operator's PATH, needs no flag. Otherwise nothing runs.
fn semgrep_program(config: Option<&str>, flag: Option<&str>) -> Result<String, String> {
    match (config, flag) {
        (Some(b), Some(f)) if b != f && b != "semgrep" => Err(refusal(b)),
        (_, Some(f)) => Ok(f.to_string()),
        (None | Some("semgrep"), None) => Ok("semgrep".to_string()),
        (Some(b), None) => Err(refusal(b)),
    }
}

fn refusal(b: &str) -> String {
    let b = crate::term::escape_value(b);
    format!("composite.toml names bin_path {b}; a program named by the repository runs only when the command line names it too: --semgrep-bin {b}")
}

/// Static-analysis leg. Invokes Semgrep via an argument vector (never a shell
/// string), always against the LOCAL ruleset (never `--config auto`).
pub fn run_semgrep(cfg: &ToolConfig, tc: &TestCase, timeout: Duration) -> LegVerdict {
    const TOOL: &str = "semgrep";
    if !cfg.enabled {
        return LegVerdict::skipped(TOOL, "semgrep disabled in composite.toml");
    }
    let bin = match semgrep_program(cfg.bin_path.as_deref(), cfg.operator_bin.as_deref()) {
        Ok(b) => b,
        Err(why) => return LegVerdict::error(TOOL, &why),
    };
    let rules = match cfg.rules.as_deref() {
        Some(r) => r,
        None => {
            return LegVerdict::error(TOOL, "no local ruleset configured (refusing --config auto)")
        }
    };

    let args: Vec<String> = vec![
        "--config".into(),
        rules.into(),
        tc.target_path.clone(),
        "--json".into(),
        "--quiet".into(),
        "--no-git-ignore".into(),
    ];

    let output = match exec_with_timeout(&bin, args, timeout) {
        ExecOutcome::Completed(o) => o,
        ExecOutcome::NotFound => {
            return LegVerdict::skipped(TOOL, "semgrep binary not found on PATH")
        }
        ExecOutcome::SpawnError(e) => {
            return LegVerdict::error(TOOL, &format!("semgrep spawn error: {e}"))
        }
        ExecOutcome::TimedOut => return LegVerdict::error(TOOL, "semgrep timed out"),
    };

    let parsed: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(e) => {
            return LegVerdict::error(
                TOOL,
                &format!("semgrep stdout was not parseable JSON: {e}"),
            )
        }
    };

    let results = parsed.get("results").and_then(|r| r.as_array());
    let results = match results {
        Some(r) => r,
        None => return LegVerdict::error(TOOL, "semgrep JSON missing `results` array"),
    };

    let check_ids: Vec<String> = results
        .iter()
        .filter_map(|r| r.get("check_id").and_then(|c| c.as_str()))
        .map(|s| s.to_string())
        .collect();
    let findings = check_ids.len() as u32;

    let caught_designated = match &tc.catch_signature {
        Some(sig) => check_ids.iter().any(|id| id.contains(sig)),
        None => false,
    };

    let state = if findings > 0 {
        LegState::FAIL
    } else {
        LegState::PASS
    };

    let summary = if findings == 0 {
        "semgrep executed, no findings".to_string()
    } else {
        format!("semgrep executed, {findings} finding(s): {}", check_ids.join(", "))
    };

    LegVerdict {
        tool: TOOL.to_string(),
        state,
        findings,
        caught_designated,
        summary,
    }
}

/// Policy leg. A real OPA integration would evaluate the rego policy against a
/// structured input document; Phase 1 has no such document pipeline yet, so
/// this leg honestly reports SKIPPED rather than fabricating a policy pass.
pub fn run_opa(cfg: &ToolConfig, _tc: &TestCase) -> LegVerdict {
    if !cfg.enabled {
        return LegVerdict::skipped("opa", "opa disabled in composite.toml");
    }
    LegVerdict::skipped(
        "opa",
        "opa leg not wired to an input document in Phase 1 — reported SKIPPED, not PASS",
    )
}

/// Runtime-observability leg. Requires a live OpenTelemetry collector and a
/// running workload. Offline, there is nothing to observe, so this is SKIPPED
/// (never PASS). This is why the runtime-shaped cases resolve to UNKNOWN.
pub fn run_otel(cfg: &ToolConfig, _tc: &TestCase) -> LegVerdict {
    if !cfg.enabled {
        return LegVerdict::skipped("opentelemetry", "opentelemetry disabled in composite.toml");
    }
    LegVerdict::skipped(
        "opentelemetry",
        "no live collector/workload offline — runtime leg cannot execute, reported SKIPPED",
    )
}

/// Property-testing leg. Requires a bound runner and per-case properties, which
/// Phase 1 does not ship. SKIPPED, never PASS.
pub fn run_property(cfg: &ToolConfig, _tc: &TestCase) -> LegVerdict {
    if !cfg.enabled {
        return LegVerdict::skipped("property_testing", "property_testing disabled in composite.toml");
    }
    LegVerdict::skipped(
        "property_testing",
        "no bound runner/properties in Phase 1 — reported SKIPPED, not PASS",
    )
}
