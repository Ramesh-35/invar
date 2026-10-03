//! Acceptance tests for the Phase-1 falsification instrument.
//!
//! Twelve tests in five groups: the classifier's epistemic honesty (1-3), the
//! bi-temporal ledger's integrity (4-6), temporal admissibility via as-of
//! reconstruction (7-8, GAP-C), conservation-with-residual (9-10, GAP-F), and
//! provenance independence (11-12, GAP-B). These are the ground truth the crate
//! is written to satisfy. (Tests 11-12 are kept here rather than in a separate
//! acceptance.rs so the crate's acceptance ground truth stays in one file.)

use invar_falsify::evaluator::{classify, composite_residual, LegState, LegVerdict};
use invar_falsify::ledger::{Ledger, LedgerError};
use invar_falsify::manifest::{
    EpistemicStatus, EvidenceTier, FailureType, TestCase,
};
use invar_falsify::provenance::{Corroboration, Independence, ProvenanceGraph, ProvenanceSource};

fn source(id: &str, ancestors: &[&str]) -> ProvenanceSource {
    ProvenanceSource {
        source_id: id.to_string(),
        ancestor_ids: ancestors.iter().map(|s| s.to_string()).collect(),
        trust_tier: EvidenceTier::FIXTURE,
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
