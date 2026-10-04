//! Acceptance tests for the Phase-1 falsification instrument.
//!
//! Eighteen tests in eight groups: the classifier's epistemic honesty (1-3),
//! the bi-temporal ledger's integrity (4-6), temporal admissibility via as-of
//! reconstruction (7-8, GAP-C), conservation-with-residual (9-10, GAP-F),
//! provenance independence (11-12, GAP-B), executed-but-inadmissible
//! degradation (13-14, GAP-E), reality-tier admission (15-16, GAP-D gate), and
//! the cross-repository differential probe (17-18, GAP-D resolving step, run
//! against the exact bytes of REAL-INCIDENT-001). These are the ground truth
//! the crate is written to satisfy. (Tests are kept in this one file so the
//! crate's acceptance ground truth stays cohesive.)

use invar_falsify::evaluator::{
    admit_reality, admit_temporal, classify, composite_residual, leg_corroboration, LegState,
    LegVerdict,
};
use invar_falsify::ledger::{Ledger, LedgerError};
use invar_falsify::manifest::{
    EpistemicStatus, EvidenceTier, FailureType, TestCase,
};
use invar_falsify::provenance::{Corroboration, Independence, ProvenanceGraph, ProvenanceSource};
use invar_falsify::reality::{
    derive_probe_hash, differential_residue, verify_real_tier, AdmissibilityError,
    RealArtifactProof,
};

// Exact bytes fetched from the two commits of REAL-INCIDENT-001.
const PRODUCER_SCHEMA_P1: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OrderPayload",
  "type": "object",
  "properties": {
    "order_id": { "type": "string" },
    "amount_micro_units": { "type": "integer" }
  },
  "required": ["order_id", "amount_cents"]
}
"#;
const CONSUMER_CLIENT_C1: &str = r#"// Consumer expectation bound to Producer schema.json
function processOrder(payload) {
  if (!payload.amount_cents) {
    throw new Error("Missing invariant requirement: amount_cents");
  }
  return payload.amount_cents / 100;
}
"#;

fn source(id: &str, ancestors: &[&str]) -> ProvenanceSource {
    ProvenanceSource {
        source_id: id.to_string(),
        ancestor_ids: ancestors.iter().map(|s| s.to_string()).collect(),
        trust_tier: EvidenceTier::FIXTURE,
    }
}

fn real_case(failure_type: FailureType, catch_signature: Option<&str>) -> TestCase {
    TestCase {
        id: "TC-REAL".to_string(),
        description: "real-tier placeholder".to_string(),
        cross_repository: true,
        failure_type,
        evidence_tier: EvidenceTier::REAL,
        target_path: "./corpus/real".to_string(),
        catch_signature: catch_signature.map(|s| s.to_string()),
        expected_status: None,
    }
}

fn case(failure_type: FailureType, catch_signature: Option<&str>) -> TestCase {
    TestCase {
        id: "TC-XXX".to_string(),
        description: "synthetic".to_string(),
        cross_repository: false,
        failure_type,
        evidence_tier: EvidenceTier::FIXTURE,
        target_path: "./corpus/synthetic".to_string(),
        catch_signature: catch_signature.map(|s| s.to_string()),
        expected_status: None,
    }
}

fn leg(tool: &str, state: LegState, findings: u32, caught: bool) -> LegVerdict {
    LegVerdict {
        tool: tool.to_string(),
        state,
        findings,
        caught_designated: caught,
        summary: "synthetic leg".to_string(),
    }
}

fn skipped(tool: &str) -> LegVerdict {
    leg(tool, LegState::SKIPPED, 0, false)
}

fn unique_db_path(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("invar_ledger_{tag}_{}_{nanos}.db", std::process::id()))
}

fn cleanup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

// 1. A catch requires an actual catch_signature match — not merely a finding.
#[test]
fn catch_requires_signature_match() {
    let tc = case(FailureType::AST_SCHEMA_DESYNC, Some("tc005"));
    let legs = vec![
        leg("semgrep", LegState::FAIL, 1, true),
        skipped("opa"),
        skipped("opentelemetry"),
        skipped("property_testing"),
    ];
    let (status, _why) = classify(&tc, &legs);
    assert_eq!(status, EpistemicStatus::CAUGHT_BY_COMPOSITE);
}

// 2. A stray advisory finding with no signature match is residue, not a catch.
#[test]
fn advisory_finding_is_residue_not_catch() {
    let tc = case(FailureType::AST_SCHEMA_DESYNC, None);
    let legs = vec![
        // semgrep executed and even fired an advisory (FAIL, findings=1) but did
        // NOT match a catch_signature, so it is not a catch.
        leg("semgrep", LegState::FAIL, 1, false),
        skipped("opa"),
        skipped("opentelemetry"),
        skipped("property_testing"),
    ];
    let (status, _why) = classify(&tc, &legs);
    assert_eq!(status, EpistemicStatus::PROVEN_RESIDUE);
}

// 3. A missing required leg forces UNKNOWN — never residue by association.
#[test]
fn missing_required_leg_is_unknown() {
    // CONCURRENCY_INVARIANT requires the OpenTelemetry leg, which is SKIPPED.
    let tc = case(FailureType::CONCURRENCY_INVARIANT, None);
    let legs = vec![
        leg("semgrep", LegState::PASS, 0, false),
        skipped("opa"),
        skipped("opentelemetry"),
        skipped("property_testing"),
    ];
    let (status, _why) = classify(&tc, &legs);
    assert_eq!(status, EpistemicStatus::UNKNOWN);

    // And an AST case whose required semgrep leg is SKIPPED is likewise UNKNOWN.
    let tc2 = case(FailureType::AST_SCHEMA_DESYNC, Some("tc005"));
    let legs2 = vec![skipped("semgrep")];
    let (status2, _) = classify(&tc2, &legs2);
    assert_eq!(status2, EpistemicStatus::UNKNOWN);
}

// 4. The chain verifies, and valid-time is kept distinct from transaction-time.
#[test]
fn ledger_chain_verifies_with_distinct_times() {
    let path = unique_db_path("chain");
    {
        let ledger = Ledger::open(path.to_str().unwrap()).unwrap();
        let n1 = ledger
            .append(1_000, "case_verdict", "TC-001", "content-a")
            .unwrap();
        let _n2 = ledger
            .append_now("case_verdict", "TC-005", "content-b")
            .unwrap();

        // Explicit valid time is preserved and distinct from the record time.
        assert_eq!(n1.valid_time_us, 1_000);
        assert_ne!(n1.valid_time_us, n1.transaction_time_us);

        assert_eq!(ledger.verify().unwrap(), 2);
    }
    cleanup(&path);
}

// 5. A silent, out-of-band row edit is detected by recomputation.
#[test]
fn ledger_detects_silent_row_edit() {
    let path = unique_db_path("tamper");
    {
        let ledger = Ledger::open(path.to_str().unwrap()).unwrap();
        ledger
            .append_now("case_verdict", "TC-001", "original-content")
            .unwrap();
        ledger
            .append_now("case_verdict", "TC-005", "second-content")
            .unwrap();
        assert_eq!(ledger.verify().unwrap(), 2);
    } // drop the ledger connection before tampering out-of-band.

    {
        let raw = rusqlite::Connection::open(&path).unwrap();
        raw.execute(
            "UPDATE lineage SET content = 'tampered' WHERE id = 1",
            [],
        )
        .unwrap();
    }

    let ledger = Ledger::open(path.to_str().unwrap()).unwrap();
    match ledger.verify() {
        Err(LedgerError::StateMismatch { id }) => assert_eq!(id, 1),
        other => panic!("expected StateMismatch at id 1, got {other:?}"),
    }
    cleanup(&path);
}

// 6. WAL journal mode is actually set on disk.
#[test]
fn ledger_uses_wal_on_disk() {
    let path = unique_db_path("wal");
    {
        let ledger = Ledger::open(path.to_str().unwrap()).unwrap();
        assert_eq!(ledger.journal_mode().unwrap().to_lowercase(), "wal");
    }
    cleanup(&path);
}

// 7. (GAP-C) A fact recorded later (t_r) is invisible to an earlier as-of
//    reconstruction: later knowledge cannot contaminate an earlier judgment.
#[test]
fn replay_as_of_excludes_later_knowledge() {
    let path = unique_db_path("asof_tr");
    {
        let ledger = Ledger::open(path.to_str().unwrap()).unwrap();
        // A: valid at 10, recorded at 100. B: valid at 20, recorded at 200.
        ledger.append_at(10, 100, "case_verdict", "A", "content-a").unwrap();
        ledger.append_at(20, 200, "case_verdict", "B", "content-b").unwrap();

        // As known at t_r = 150 (valid-time open), only A is admissible.
        let seen = ledger.replay_as_of(150, i64::MAX).unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].subject, "A");

        // As known at t_r = 200, both are admissible.
        let seen_all = ledger.replay_as_of(200, i64::MAX).unwrap();
        assert_eq!(seen_all.len(), 2);

        // The full chain still verifies regardless of the as-of slice.
        assert_eq!(ledger.verify().unwrap(), 2);
    }
    cleanup(&path);
}

// 8. (GAP-C) The valid-time bound is enforced independently of t_r, and a
//    tampered row is still caught during as-of reconstruction.
#[test]
fn replay_as_of_respects_valid_time_and_catches_tamper() {
    let path = unique_db_path("asof_tv");
    {
        let ledger = Ledger::open(path.to_str().unwrap()).unwrap();
        ledger.append_at(10, 100, "case_verdict", "A", "content-a").unwrap();
        // B becomes true in the world only at t_v = 300, though recorded early.
        ledger.append_at(300, 110, "case_verdict", "B", "content-b").unwrap();

        // As-of valid-time 50 excludes B even though it was recorded by t_r=110.
        let seen = ledger.replay_as_of(i64::MAX, 50).unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].subject, "A");
    }

    // Tamper with row 1 out-of-band; as-of reconstruction must refuse.
    {
        let raw = rusqlite::Connection::open(&path).unwrap();
        raw.execute("UPDATE lineage SET content = 'tampered' WHERE id = 1", [])
            .unwrap();
    }
    let ledger = Ledger::open(path.to_str().unwrap()).unwrap();
    match ledger.replay_as_of(i64::MAX, i64::MAX) {
        Err(LedgerError::StateMismatch { id }) => assert_eq!(id, 1),
        other => panic!("expected StateMismatch at id 1, got {other:?}"),
    }
    cleanup(&path);
}

// 9. (GAP-F) A catch does not clear the residual: a CAUGHT_BY_COMPOSITE case
//    whose other legs were SKIPPED still records them as uncovered.
#[test]
fn catch_preserves_composite_residual() {
    let tc = case(FailureType::AST_SCHEMA_DESYNC, Some("tc005"));
    let legs = vec![
        leg("semgrep", LegState::FAIL, 1, true),
        skipped("opa"),
        skipped("opentelemetry"),
        skipped("property_testing"),
    ];
    let (status, _why) = classify(&tc, &legs);
    assert_eq!(status, EpistemicStatus::CAUGHT_BY_COMPOSITE);

    let cons = composite_residual(&legs);
    assert_eq!(cons.total_legs, 4);
    assert_eq!(cons.executed_legs, 1);
    assert_eq!(
        cons.residual_legs,
        vec![
            "opa".to_string(),
            "opentelemetry".to_string(),
            "property_testing".to_string()
        ]
    );
    assert!(cons.holds(), "conservation: total == executed + residual");
}

// 10. (GAP-F) The conservation equation balances whether all legs executed or
//     none did: the residual is accounted for, never silently dropped.
#[test]
fn conservation_balances_across_coverage() {
    // All four executed: residual empty.
    let all = vec![
        leg("semgrep", LegState::PASS, 0, false),
        leg("opa", LegState::PASS, 0, false),
        leg("opentelemetry", LegState::FAIL, 1, false),
        leg("property_testing", LegState::PASS, 0, false),
    ];
    let c_all = composite_residual(&all);
    assert_eq!(c_all.executed_legs, 4);
    assert!(c_all.residual_legs.is_empty());
    assert!(c_all.holds());

    // None executed: everything is residual, nothing vanishes.
    let none = vec![
        skipped("semgrep"),
        skipped("opa"),
        skipped("opentelemetry"),
        skipped("property_testing"),
    ];
    let c_none = composite_residual(&none);
    assert_eq!(c_none.executed_legs, 0);
    assert_eq!(c_none.residual_legs.len(), 4);
    assert!(c_none.holds());
}

// 11. (GAP-B) Two sources descending from one ancestor are not independent,
//     and an unknown source can never be declared independent.
#[test]
fn shared_ancestor_sources_are_not_independent() {
    let mut g = ProvenanceGraph::new();
    g.add_source(source("root", &[]));
    g.add_source(source("a", &["root"]));
    g.add_source(source("b", &["root"]));

    assert!(!g.are_independent("a", "b"));
    assert_eq!(g.independence("a", "b"), Independence::Correlated);

    // A source is never independent of itself.
    assert_eq!(g.independence("a", "a"), Independence::Correlated);

    // Unknown provenance is Unknown, never Independent.
    assert_eq!(g.independence("a", "ghost"), Independence::Unknown);
    assert!(!g.are_independent("a", "ghost"));

    // Genuinely disjoint lineages are independent.
    g.add_source(source("c", &[]));
    assert!(g.are_independent("a", "c"));
    assert_eq!(g.independence("a", "c"), Independence::Independent);
}

// 12. (GAP-B) Shared-ancestry corroboration stays BANDED; only disjoint
//     lineages promote to CONFIDENT. Adding more carriers from the same
//     lineage must not inflate standing.
#[test]
fn shared_ancestry_does_not_promote_standing() {
    let mut g = ProvenanceGraph::new();
    g.add_source(source("root", &[]));
    g.add_source(source("a", &["root"]));
    g.add_source(source("b", &["root"]));

    // a and b share "root": correlated, so they cannot corroborate as two.
    assert_eq!(g.corroboration(&["a", "b"]), Corroboration::Banded);

    // A genuinely independent source with disjoint lineage promotes to CONFIDENT.
    g.add_source(source("c", &[]));
    assert_eq!(g.corroboration(&["a", "c"]), Corroboration::Confident);

    // a + b + c is still only TWO independent groups ({a,b}, {c}): adding b, a
    // non-independent carrier, does not inflate beyond CONFIDENT nor collapse it.
    assert_eq!(g.corroboration(&["a", "b", "c"]), Corroboration::Confident);

    // Degenerate cases: one source is BANDED, none is UNSUPPORTED.
    assert_eq!(g.corroboration(&["a"]), Corroboration::Banded);
    assert_eq!(g.corroboration(&[]), Corroboration::Unsupported);
}

// 13. (GAP-E) An executed, caught leg whose evidence lies beyond the as-of
//     bound is temporally inadmissible: it cannot count, so a case that would
//     have been CAUGHT degrades to UNKNOWN and the leg is logged as residual.
#[test]
fn executed_but_temporally_inadmissible_degrades_to_unknown() {
    let tc = case(FailureType::AST_SCHEMA_DESYNC, Some("tc005"));
    // semgrep (the required leg) executed and even caught the designated failure.
    let legs = vec![
        leg("semgrep", LegState::FAIL, 1, true),
        skipped("opa"),
        skipped("opentelemetry"),
        skipped("property_testing"),
    ];
    // Without admissibility, this is a catch.
    assert_eq!(classify(&tc, &legs).0, EpistemicStatus::CAUGHT_BY_COMPOSITE);

    // But semgrep's evidence was recorded/valid at 500, beyond the as-of bound 100.
    let times = vec![("semgrep".to_string(), 500i64, 500i64)];
    let admitted = admit_temporal(&legs, &times, 100, 100);

    // The required leg is now INADMISSIBLE (not executed), so the honest verdict
    // is UNKNOWN, not an unearned catch.
    assert_eq!(
        admitted.iter().find(|l| l.tool == "semgrep").unwrap().state,
        LegState::INADMISSIBLE
    );
    assert_eq!(classify(&tc, &admitted).0, EpistemicStatus::UNKNOWN);

    // The inadmissible leg is accounted for as residual, never silently dropped.
    let cons = composite_residual(&admitted);
    assert_eq!(cons.executed_legs, 0);
    assert!(cons.residual_legs.contains(&"semgrep".to_string()));
    assert!(cons.holds());
}

// 14. (GAP-E) Executed legs that claim to corroborate but share provenance
//     cannot earn CONFIDENT standing; only genuinely independent legs do.
#[test]
fn correlated_executed_legs_do_not_earn_confident() {
    let legs = vec![
        leg("semgrep", LegState::FAIL, 1, true),
        leg("opa", LegState::FAIL, 1, true),
        skipped("opentelemetry"),
        skipped("property_testing"),
    ];
    let leg_sources = vec![
        ("semgrep".to_string(), "semgrep_src".to_string()),
        ("opa".to_string(), "opa_src".to_string()),
    ];

    // Both carriers descend from one origin: correlated, capped at BANDED.
    let mut correlated = ProvenanceGraph::new();
    correlated.add_source(source("origin", &[]));
    correlated.add_source(source("semgrep_src", &["origin"]));
    correlated.add_source(source("opa_src", &["origin"]));
    assert_eq!(
        leg_corroboration(&legs, &leg_sources, &correlated),
        Corroboration::Banded
    );

    // Disjoint lineages: the same two executed legs now earn CONFIDENT.
    let mut independent = ProvenanceGraph::new();
    independent.add_source(source("semgrep_src", &[]));
    independent.add_source(source("opa_src", &[]));
    assert_eq!(
        leg_corroboration(&legs, &leg_sources, &independent),
        Corroboration::Confident
    );
}

// 15. (GAP-D) A case that self-declares REAL with no proof is refused by the
//     reality-admission gate: every leg is demoted INADMISSIBLE and the verdict
//     degrades to UNKNOWN. This neutralizes the trivial M4 self-promotion.
#[test]
fn unbacked_real_tier_is_refused_and_degrades_to_unknown() {
    let tc = real_case(FailureType::AST_SCHEMA_DESYNC, Some("tc005"));
    // Legs that, on FIXTURE, would be a clean catch.
    let legs = vec![
        leg("semgrep", LegState::FAIL, 1, true),
        skipped("opa"),
        skipped("opentelemetry"),
        skipped("property_testing"),
    ];
    // On FIXTURE this is CAUGHT; the REAL claim is what must be gated.
    assert_eq!(
        verify_real_tier(tc.evidence_tier, None, None),
        Err(AdmissibilityError::MissingProof)
    );
    let admitted = admit_reality(&legs, tc.evidence_tier, None, None);
    assert!(admitted.iter().all(|l| l.state == LegState::INADMISSIBLE));
    assert_eq!(classify(&tc, &admitted).0, EpistemicStatus::UNKNOWN);
}

// 16. (GAP-D) A REAL claim whose proof BINDS to the recorded residue is
//     admitted (the verdict stands); a proof whose hash does not bind is
//     refused and degrades to UNKNOWN. NOTE: binding is a necessary, not
//     sufficient, check; it does not prove the commits are real.
#[test]
fn backed_real_tier_is_admitted_and_mismatch_is_refused() {
    let tc = real_case(FailureType::AST_SCHEMA_DESYNC, Some("tc005"));
    let legs = vec![
        leg("semgrep", LegState::FAIL, 1, true),
        skipped("opa"),
        skipped("opentelemetry"),
        skipped("property_testing"),
    ];
    let residue = [7u8; 32];
    let proof = RealArtifactProof {
        producer_repo: "example/api-producer".to_string(),
        producer_commit: "aaaaaaaaaaaa".to_string(),
        consumer_repo: "example/api-consumer".to_string(),
        consumer_commit: "bbbbbbbbbbbb".to_string(),
        diff_probe_hash: residue,
    };

    // Proof binds to the recorded residue: admitted, caught verdict stands.
    assert!(verify_real_tier(tc.evidence_tier, Some(&proof), Some(residue)).is_ok());
    let admitted = admit_reality(&legs, tc.evidence_tier, Some(&proof), Some(residue));
    assert_eq!(
        admitted.iter().find(|l| l.tool == "semgrep").unwrap().state,
        LegState::FAIL
    );
    assert_eq!(
        classify(&tc, &admitted).0,
        EpistemicStatus::CAUGHT_BY_COMPOSITE
    );

    // Proof that does NOT bind to the residue is refused, degrading to UNKNOWN.
    let bad = RealArtifactProof {
        diff_probe_hash: [9u8; 32],
        ..proof.clone()
    };
    assert_eq!(
        verify_real_tier(tc.evidence_tier, Some(&bad), Some(residue)),
        Err(AdmissibilityError::ResidueMismatch)
    );
    let demoted = admit_reality(&legs, tc.evidence_tier, Some(&bad), Some(residue));
    assert_eq!(classify(&tc, &demoted).0, EpistemicStatus::UNKNOWN);
}

// 17. (GAP-D resolving) The differential probe, run on the EXACT real bytes of
//     REAL-INCIDENT-001, isolates the one field the consumer requires that the
//     producer schema does not define: amount_cents. The probe hash is
//     deterministic.
#[test]
fn differential_probe_isolates_real_residue() {
    let residue = differential_residue(PRODUCER_SCHEMA_P1, CONSUMER_CLIENT_C1);
    assert_eq!(residue, vec!["amount_cents".to_string()]);

    // The producer defines amount_micro_units and order_id; neither is residue.
    assert!(!residue.contains(&"amount_micro_units".to_string()));
    assert!(!residue.contains(&"order_id".to_string()));

    // Hash is deterministic over the canonical residue.
    assert_eq!(
        derive_probe_hash(&residue),
        derive_probe_hash(&["amount_cents".to_string()])
    );
}

// 18. (GAP-D resolving, control) When the consumer only reads a field the
//     producer DOES define, there is no residue: the probe does not invent one.
#[test]
fn differential_probe_empty_when_contract_holds() {
    let consumer_ok = r#"function processOrder(payload) { return payload.order_id; }"#;
    let residue = differential_residue(PRODUCER_SCHEMA_P1, consumer_ok);
    assert!(residue.is_empty(), "no residue when the field is defined");
}
