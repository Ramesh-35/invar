//! Acceptance tests for the Phase-1 falsification instrument.
//!
//! Twenty-three tests in twelve groups: the classifier's epistemic honesty (1-3),
//! the bi-temporal ledger's integrity (4-6), temporal admissibility via as-of
//! reconstruction (7-8, GAP-C), conservation-with-residual (9-10, GAP-F),
//! provenance independence (11-12, GAP-B), executed-but-inadmissible
//! degradation (13-14, GAP-E), reality-tier admission (15-16, GAP-D gate), the
//! cross-repository differential probe (17-18, run against the exact bytes of
//! REAL-INCIDENT-001), the general invariant engine's EventContract and
//! LockOrderContract kinds (19-20, synthetic), `invar init` zero-touch discovery
//! (21), `invar check --ci` in-memory remote resolution through a mock
//! transport (22), and the GitHub Action surface: PR-report rendering and the
//! structure of action.yml (23). These are the ground truth the crate is
//! written to satisfy.
//! (Tests are kept in this one file so the acceptance ground truth stays
//! cohesive.)

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
use invar_falsify::invariant::{check_contract, ContractInput, Step};
use invar_falsify::check::{evaluate, render_report_md, ContractOutcome, Witness, REPORT_MARKER};
use invar_falsify::config::parse_sections;
use invar_falsify::init::{discover, render_manifest};
use invar_falsify::resolver::{
    curl_args, github_request, parse_ref, CurlTransport, Fetched, MockTransport, ResolveError,
    Resolver, Transport,
};

fn temp_workspace(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn put(root: &std::path::Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

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

// 19. (General engine, synthetic) EventContract: a consumer handler reading a
//     payload field the producer does not emit is a violation; a compatible
//     handler holds.
#[test]
fn event_contract_detects_unemitted_field() {
    let producer_payload = r#"{ "order_id": "o1", "amount_micro_units": 500 }"#;
    let consumer_handler = r#"function onOrder(payload) { return payload.currency + payload.order_id; }"#;
    let residue = check_contract(&ContractInput::Event {
        producer_payload,
        consumer_handler,
    });
    assert_eq!(residue, vec!["currency".to_string()]);

    // Control: a handler that only reads emitted fields holds.
    let compatible = r#"function onOrder(payload) { return payload.order_id; }"#;
    let ok = check_contract(&ContractInput::Event {
        producer_payload,
        consumer_handler: compatible,
    });
    assert!(ok.is_empty());
}

// 20. (General engine, synthetic) LockOrderContract: acquiring a lock AFTER the
//     dependent trigger fired is a sequence inversion; the correct order holds.
#[test]
fn lock_order_contract_detects_inversion() {
    let before = Step::new("ServiceA", "acquire", "LockX");
    let after = Step::new("ServiceB", "trigger", "EventY");

    // Inverted: B triggers before A acquires.
    let inverted = vec![
        Step::new("ServiceB", "trigger", "EventY"),
        Step::new("ServiceA", "acquire", "LockX"),
    ];
    let residue = check_contract(&ContractInput::LockOrder {
        before: before.clone(),
        after: after.clone(),
        observed: inverted,
    });
    assert_eq!(residue.len(), 1);
    assert!(residue[0].starts_with("ORDER_INVERSION"));

    // Correct order holds.
    let ordered = vec![
        Step::new("ServiceA", "acquire", "LockX"),
        Step::new("ServiceB", "trigger", "EventY"),
    ];
    let ok = check_contract(&ContractInput::LockOrder {
        before,
        after,
        observed: ordered,
    });
    assert!(ok.is_empty());
}

const OPENAPI_JSON: &str = r#"{
  "openapi": "3.0.0",
  "info": { "title": "orders", "version": "1" },
  "paths": {},
  "components": { "schemas": { "Order": {
    "type": "object",
    "properties": { "order_id": { "type": "string" }, "amount_micro_units": { "type": "integer" } },
    "required": ["order_id", "amount_cents"]
  } } }
}"#;

// 21. `invar init` discovers specs, consumers, routes and models, proposes only
//     runnable contracts (JSON Schema and OpenAPI-JSON component, each paired
//     with a consumer by field overlap), skips vendored directories, lists the
//     rest as inventory, and emits a manifest that parses and actually runs.
#[test]
fn init_discovers_and_emits_runnable_manifest() {
    let root = temp_workspace("init");
    put(&root, "schemas/order.schema.json", PRODUCER_SCHEMA_P1);
    put(&root, "api/openapi.json", OPENAPI_JSON);
    put(&root, "api/openapi.yaml", "openapi: 3.0.0\ninfo:\n  title: orders\n");
    put(&root, "proto/order.proto", "syntax = \"proto3\";\nmessage Order { string order_id = 1; }\n");
    put(&root, "src/client.js", CONSUMER_CLIENT_C1);
    put(
        &root,
        "server/app.py",
        "from fastapi import FastAPI\nfrom pydantic import BaseModel\napp = FastAPI()\n\nclass Order(BaseModel):\n    order_id: str\n\n@app.get(\"/orders\")\ndef list_orders():\n    return []\n",
    );
    put(&root, "server/routes.rs", "let app = Router::new().route(\"/orders\", get(list));\n");
    // A vendored decoy that would be a consumer if it were scanned.
    put(&root, "node_modules/pkg/client.js", "x = payload.amount_cents;\n");

    let d = discover(&root);
    assert_eq!(d.schemas.len(), 2, "json-schema + openapi component");
    assert_eq!(d.consumers.len(), 1, "node_modules must be skipped");
    assert_eq!(d.consumers[0].path, "src/client.js");
    assert_eq!(d.proposals.len(), 2);

    let manifest = render_manifest(&d);
    // Non-checkable artifacts are inventoried, explicitly not proposed.
    for needle in [
        "api/openapi.yaml",
        "proto/order.proto",
        "pydantic Order",
        "fastapi GET \"/orders\"",
        "axum \"/orders\"",
        "HEURISTIC",
    ] {
        assert!(manifest.contains(needle), "manifest should mention {needle}");
    }

    // It parses with the flat-TOML parser...
    let sections = parse_sections("invar.toml", &manifest).unwrap();
    let blocks = sections.iter().filter(|(n, _)| n == "invariant").count();
    assert_eq!(blocks, 2);

    // ...and it runs: both proposed contracts find the real residue.
    std::fs::write(root.join("invar.toml"), &manifest).unwrap();
    let outcomes = evaluate(&sections, &Resolver::local(&root)).unwrap();
    assert_eq!(outcomes.len(), 2);
    for o in &outcomes {
        assert_eq!(o.residue, vec!["amount_cents".to_string()], "{}", o.id);
        assert!(o.provenance.is_empty(), "local artifacts carry no remote provenance");
    }
    let _ = std::fs::remove_dir_all(&root);
}

// 22. `invar check --ci`: remote references are parsed strictly, requests are
//     built correctly (raw endpoint without a token, contents API with one, the
//     token never printed), remote use fails closed without --ci or without a
//     transport, and with a transport the consumer is fetched into memory,
//     hashed, evaluated, and never written to disk.
#[test]
fn ci_resolves_remote_consumer_in_memory() {
    const C1: &str = "fe73889bf250275711339d065ecc37f21879fc72";
    let spec = format!("github:Ramesh-35/invar-demo-consumer@{C1}:client.js");

    // Strict parsing.
    let r = parse_ref(&spec, None).unwrap();
    assert_eq!((r.owner.as_str(), r.repo.as_str(), r.path.as_str()), ("Ramesh-35", "invar-demo-consumer", "client.js"));
    assert_eq!(parse_ref("github:o/r@{base}:a.json", Some("main")).unwrap().git_ref, "main");
    assert!(matches!(parse_ref("github:o/r@{base}:a.json", None), Err(ResolveError::MissingBaseRef(_))));
    assert!(matches!(parse_ref("github:o@main:a.json", None), Err(ResolveError::BadReference(_))));
    assert!(matches!(parse_ref("github:o/r@main:../a.json", None), Err(ResolveError::BadReference(_))));
    assert!(matches!(parse_ref("gitlab:o/r@main:a.json", None), Err(ResolveError::Unsupported(_))));

    // Request construction, and the token never leaks through Debug.
    let raw_url = format!("https://raw.githubusercontent.com/Ramesh-35/invar-demo-consumer/{C1}/client.js");
    assert_eq!(github_request(&r, None).url, raw_url);
    let authed = github_request(&r, Some("SECRET_T"));
    assert_eq!(
        authed.url,
        format!("https://api.github.com/repos/Ramesh-35/invar-demo-consumer/contents/client.js?ref={C1}")
    );
    assert!(authed.headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer SECRET_T"));
    assert!(!format!("{authed:?}").contains("SECRET_T"));
    // The live curl transport never puts the token on the command line (it goes
    // over stdin), is HTTPS-only, and a missing binary is an error, not a body.
    let argv = curl_args(&authed);
    assert!(argv.iter().all(|a| !a.contains("SECRET_T")));
    assert!(argv.windows(2).any(|w| w[0] == "--proto" && w[1] == "=https"));
    assert!(argv.windows(2).any(|w| w[0] == "--proto-redir" && w[1] == "=https"));
    let missing = CurlTransport { program: "/nonexistent/invar-no-curl".into() };
    assert!(missing.get(&authed).unwrap_err().contains("could not run"));

    // Workspace: local producer, remote consumer.
    let root = temp_workspace("ci");
    put(&root, "schema.json", PRODUCER_SCHEMA_P1);
    let manifest = format!(
        "[[invariant]]\nkind = \"schema\"\nid = \"ci-remote-consumer\"\nproducer = \"schema.json\"\nconsumer = \"{spec}\"\n"
    );
    put(&root, "invar.toml", &manifest);
    let sections = parse_sections("invar.toml", &manifest).unwrap();

    // Fail closed: no --ci, and --ci without a live transport.
    let e = evaluate(&sections, &Resolver::local(&root)).unwrap_err().to_string();
    assert!(e.contains("requires --ci"), "{e}");
    let mut no_transport = Resolver::local(&root);
    no_transport.allow_remote = true;
    let e = evaluate(&sections, &no_transport).unwrap_err().to_string();
    assert!(e.contains("no live network transport"), "{e}");

    // With a transport: fetched into memory and evaluated.
    let mock = MockTransport::default().with(&raw_url, CONSUMER_CLIENT_C1);
    let mut ci = Resolver::local(&root);
    ci.allow_remote = true;
    ci.transport = Some(&mock as &dyn Transport);
    let outcomes = evaluate(&sections, &ci).unwrap();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].residue, vec!["amount_cents".to_string()]);
    let f = &outcomes[0].provenance[0];
    assert_eq!(f.reference, spec);
    assert_eq!(f.bytes, CONSUMER_CLIENT_C1.len());
    assert_eq!(f.blake3_hex.len(), 64);
    // Deterministic hash over the same bytes.
    let again = evaluate(&sections, &ci).unwrap();
    assert_eq!(again[0].provenance[0].blake3_hex, f.blake3_hex);

    // Never written to disk: the workspace still holds only its two files.
    let mut names: Vec<String> = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["invar.toml".to_string(), "schema.json".to_string()]);

    // A remote miss is an ERROR, never a verdict.
    let empty = MockTransport::default();
    ci.transport = Some(&empty as &dyn Transport);
    let e = evaluate(&sections, &ci).unwrap_err().to_string();
    assert!(e.contains("HTTP 404"), "{e}");
    let _ = std::fs::remove_dir_all(&root);
}

fn unescaped_pipes(line: &str) -> usize {
    let b = line.as_bytes();
    (0..b.len()).filter(|&i| b[i] == b'|' && (i == 0 || b[i - 1] != b'\\')).count()
}

/// Default of an action input, read structurally (no YAML parser in this build).
fn input_default(yml: &str, name: &str) -> Option<String> {
    let lines: Vec<&str> = yml.lines().collect();
    let start = lines.iter().position(|l| *l == format!("  {name}:"))?;
    lines[start + 1..]
        .iter()
        .take_while(|l| l.starts_with("    "))
        .find_map(|l| l.trim().strip_prefix("default:").map(|v| v.trim().to_string()))
}

// 23. GitHub Action surface. (a) The PR report is rendered by tested Rust:
//     hostile ids and residue from a fork PR stay inside escaped inline code, so
//     they cannot break the table, ping users, or inject links; it carries the
//     marker the action uses to update its comment, provenance, and the
//     not-a-PROVEN_RESIDUE disclaimer. (b) action.yml declares the specified
//     inputs and outputs, never interpolates ${{ }} into a shell line, maps exit
//     codes so an ERROR is never a verdict, and keeps the comment best-effort.
#[test]
fn action_report_and_manifest_are_sound() {
    let outcomes = vec![
        ContractOutcome {
            id: "evil|id`@org".into(),
            kind: "schema".into(),
            residue: vec!["amount_cents".into(), "a|b".into(), "@someone".into()],
            provenance: vec![Fetched {
                reference: "github:o/r@main:client.js".into(),
                url: "https://raw.githubusercontent.com/o/r/main/client.js".into(),
                bytes: 229,
                blake3_hex: "ecd0a60aa8fd380e574fae70e75810da57f3b43c744ec03d492bc3b21c576f69".into(),
            }],
            // Hostile witness values (file names come from the scanned repo).
            witnesses: vec![Witness {
                residue: "a|b".into(),
                sites: vec!["x|y`.js:1".into()],
                missing_from: Some("@org|s.json /properties".into()),
            }],
        },
        ContractOutcome { id: "ok".into(), kind: "event".into(), residue: vec![], provenance: vec![], witnesses: vec![] },
    ];
    let md = render_report_md(&outcomes);
    assert!(md.starts_with(REPORT_MARKER));
    assert!(md.contains("VIOLATED (1 of 2)"));
    for row in md.lines().filter(|l| l.starts_with("| `")) {
        assert_eq!(unescaped_pipes(row), 5, "row must keep 4 columns: {row}");
    }
    assert!(md.contains("`evil\\|id'@org`"), "backtick neutralized, pipe escaped");
    assert!(md.contains("`@someone`"), "mention rendered inert inside code");
    assert!(md.contains("`a\\|b`"));
    assert!(md.contains("ecd0a60aa8fd380e574fae70e75810da57f3b43c744ec03d492bc3b21c576f69"));
    assert!(md.contains("not a PROVEN_RESIDUE verdict"));
    let hold = render_report_md(&outcomes[1..]);
    assert!(hold.contains("ALL HOLD (1 contract(s))"));

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let yml = std::fs::read_to_string(root.join("action.yml")).expect("action.yml at repo root");
    assert!(yml.contains("  using: composite"));
    assert_eq!(input_default(&yml, "config").as_deref(), Some("invar.toml"));
    assert_eq!(input_default(&yml, "base_ref").as_deref(), Some("${{ github.base_ref }}"));
    assert_eq!(input_default(&yml, "github_token").as_deref(), Some("${{ github.token }}"));
    assert_eq!(input_default(&yml, "post_comment").as_deref(), Some("'true'"));
    for out in ["exit_code", "verdict", "report"] {
        assert!(yml.contains(&format!("  {out}:")), "output {out}");
    }
    // Script-injection guard: no ${{ }} expression on a shell line.
    for l in yml.lines() {
        assert!(!(l.trim_start().starts_with("run:") && l.contains("${{")), "interpolated run line: {l}");
    }
    assert!(yml.contains("continue-on-error: true"), "PR comment must be best-effort");

    for script in ["install.sh", "run.sh", "comment.sh", "gate.sh"] {
        assert!(yml.contains(&format!("action/{script}")), "{script} referenced");
        assert!(root.join("action").join(script).is_file(), "{script} exists");
    }
    let run = std::fs::read_to_string(root.join("action/run.sh")).unwrap();
    assert!(run.contains("1) verdict=violated") && run.contains("*) verdict=error"));
    assert!(run.contains("--report-md"));
    let gate = std::fs::read_to_string(root.join("action/gate.sh")).unwrap();
    assert!(gate.contains("ERROR, not a verdict") && gate.contains("exit 3"));
}

// ---------------------------------------------------------------------------
// Test 24: every violation points at the line that causes it (witnesses).
// Witnesses explain residue and never change it; REAL-INCIDENT-001's residue
// and probe hash are untouched.
// ---------------------------------------------------------------------------
#[test]
fn witnesses_locate_each_violation() {
    let dir = std::env::temp_dir().join(format!("invar-witness-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |name: &str, body: &str| std::fs::write(dir.join(name), body).unwrap();

    // Schema: amount_cents read on lines 3 and 5; order_id is declared.
    w("schema.json", PRODUCER_SCHEMA_P1);
    w("client.js", "// c\nfunction f(payload) {\n  const a = payload.amount_cents;\n  const id = payload.order_id;\n  return payload.amount_cents / 100;\n}\n");
    // Event: handler reads `ghost` on line 2; producer emits only `id`.
    w("event.json", r#"{"id": 1}"#);
    w("handler.js", "on(payload => {\n  use(payload.ghost, payload.id);\n});\n");
    // Lock order: commit (line 2) precedes acquire (line 4); line 3 is a comment.
    w("trace.txt", "svc:begin:tx\nsvc:commit:row\n# noise\nsvc:acquire:row\n");
    // OpenAPI-style pointer: the witness names the pointer path.
    w("api.json", r#"{"components":{"schemas":{"Order":{"properties":{"order_id":{}}}}}}"#);
    let cfg = r#"
[[invariant]]
kind = "schema"
id = "orders"
producer = "schema.json"
consumer = "client.js"

[[invariant]]
kind = "event"
id = "events"
producer = "event.json"
consumer = "handler.js"

[[invariant]]
kind = "lock_order"
id = "locks"
before = "svc:acquire:row"
after = "svc:commit:row"
trace = "trace.txt"

[[invariant]]
kind = "schema"
id = "openapi"
producer = "api.json"
pointer = "/components/schemas/Order"
consumer = "client.js"
"#;
    let sections = parse_sections("invar.toml", cfg).unwrap();
    let out = evaluate(&sections, &Resolver::local(&dir)).unwrap();

    let s = &out[0];
    assert_eq!(s.residue, vec!["amount_cents".to_string()], "witnesses must not change residue");
    assert_eq!(s.witnesses.len(), 1);
    assert_eq!(s.witnesses[0].sites, vec!["client.js:3".to_string(), "client.js:5".to_string()]);
    assert_eq!(s.witnesses[0].missing_from.as_deref(), Some("schema.json /properties"));

    let e = &out[1];
    assert_eq!(e.residue, vec!["ghost".to_string()]);
    assert_eq!(e.witnesses[0].sites, vec!["handler.js:2".to_string()]);
    assert_eq!(e.witnesses[0].missing_from.as_deref(), Some("event.json (top-level keys)"));

    let l = &out[2];
    assert_eq!(l.residue.len(), 1);
    assert!(l.residue[0].starts_with("ORDER_INVERSION"));
    assert_eq!(l.witnesses[0].sites, vec!["trace.txt:2".to_string(), "trace.txt:4".to_string()]);
    assert_eq!(l.witnesses[0].missing_from, None);

    let o = &out[3];
    assert_eq!(o.witnesses[0].missing_from.as_deref(), Some("api.json /components/schemas/Order/properties"));

    let md = render_report_md(&out);
    assert!(md.contains("`amount_cents` read at `client.js:3`, `client.js:5` · not in `schema.json /properties`"), "{md}");
    assert!(md.contains(" at `trace.txt:2`, `trace.txt:4`"), "{md}");
    for row in md.lines().filter(|l| l.starts_with("| `")) {
        assert_eq!(unescaped_pipes(row), 5, "row must keep 4 columns: {row}");
    }

    // REAL-INCIDENT-001 bytes: residue and probe hash unchanged by this work.
    assert_eq!(
        derive_probe_hash(&differential_residue(PRODUCER_SCHEMA_P1, CONSUMER_CLIENT_C1))
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        "829547e08158a232667735aafbc41ae2b1919fb822f8f55b83905ec88a045ab5"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Test 25: a producer that is not a JSON object is an ERROR, never a verdict.
// Before this, a YAML producer read as "defines nothing": HOLDS when the
// consumer had no recognised reads, invented residue when it had some.
// ---------------------------------------------------------------------------
#[test]
fn non_json_producer_is_an_error_not_a_verdict() {
    let dir = std::env::temp_dir().join(format!("invar-nonjson-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("spec.yaml"), "openapi: 3.0.0\nproperties:\n  a: {}\n").unwrap();
    std::fs::write(dir.join("list.json"), "[1, 2]").unwrap();
    std::fs::write(dir.join("c.js"), "f(payload.a)\n").unwrap();
    for (kind, producer) in [("schema", "spec.yaml"), ("event", "spec.yaml"), ("schema", "list.json")] {
        let cfg = format!(
            "[[invariant]]\nkind = \"{kind}\"\nid = \"x\"\nproducer = \"{producer}\"\nconsumer = \"c.js\"\n"
        );
        let sections = parse_sections("invar.toml", &cfg).unwrap();
        let err = evaluate(&sections, &Resolver::local(&dir)).expect_err("must not be a verdict");
        assert!(format!("{err:#}").contains("is not a JSON object"), "{kind}/{producer}: {err:#}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

