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
    let missing = CurlTransport {
        program: "/nonexistent/invar-no-curl".into(),
        ..CurlTransport::default()
    };
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
                type_change: Some(("string`|".into(), "@int".into(), "x|y.json".into())),
                read_as: vec!["a`|b".into()],
            }],
            skipped: None,
            ..Default::default()
        },
        ContractOutcome { id: "ok".into(), kind: "event".into(), ..Default::default() },
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
// Test 25: producers are JSON or YAML objects; anything else is an ERROR,
// never a verdict. (Before R1 batch 1, a YAML producer was silently read as
// "defines nothing".)
// ---------------------------------------------------------------------------
#[test]
fn producer_formats_yaml_ok_unreadable_is_an_error() {
    let dir = std::env::temp_dir().join(format!("invar-formats-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();
    w("spec.yaml", "openapi: 3.0.0\ncomponents:\n  schemas:\n    Order:\n      properties:\n        id: {type: string}\n        total:\n          type: integer\n");
    w("c.js", "f(payload.id, payload.gone)\n");
    w("list.json", "[1, 2]");
    w("prose.txt", "just a sentence, not a spec\n");
    w("anchors.yaml", "a: &x 1\nb: *x\n");

    // YAML producer with a pointer: evaluated like JSON.
    let ok = "[[invariant]]\nkind = \"schema\"\nid = \"y\"\nproducer = \"spec.yaml\"\npointer = \"/components/schemas/Order\"\nconsumer = \"c.js\"\n";
    let out = evaluate(&parse_sections("invar.toml", ok).unwrap(), &Resolver::local(&dir)).unwrap();
    assert_eq!(out[0].residue, vec!["gone".to_string()]);

    for (kind, producer, needle) in [
        ("schema", "list.json", "is not a JSON object"),
        ("event", "list.json", "is not a JSON object"),
        ("schema", "prose.txt", "is not a JSON object"),
        ("schema", "anchors.yaml", "anchors and aliases"),
    ] {
        let cfg = format!(
            "[[invariant]]\nkind = \"{kind}\"\nid = \"x\"\nproducer = \"{producer}\"\nconsumer = \"c.js\"\n"
        );
        let err = evaluate(&parse_sections("invar.toml", &cfg).unwrap(), &Resolver::local(&dir))
            .expect_err("must not be a verdict");
        assert!(format!("{err:#}").contains(needle), "{kind}/{producer}: {err:#}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Test 26: change-scoped schema contracts. With a base version, residue is
// the fields removed or retyped between base and head that the consumer
// reads by any recognised pattern; unrelated property access is not residue.
// Producers may be a spec (JSON/YAML) and/or code (Go, Python, TS, Rust).
// ---------------------------------------------------------------------------
#[test]
fn change_scoped_contracts_find_removed_and_retyped_fields() {
    use invar_falsify::surface::{broad_reads, code_fields, field_names};
    let dir = std::env::temp_dir().join(format!("invar-change-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();

    // Spec: `legacy` removed, `size` retyped, `extra` added.
    w("base.yaml", "properties:\n  id: {type: string}\n  legacy: {type: string}\n  size: {type: string}\n");
    w("head.yaml", "properties:\n  id: {type: string}\n  size: {type: integer}\n  extra: {type: string}\n");
    // Consumer reads through several styles, plus lots of unrelated access.
    w(
        "client.py",
        "def f(data):\n    a = data[\"legacy\"]\n    b = data.get('size')\n    n = len(data.items())\n    return data.id, a.upper(), b\n",
    );
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"s\"\nproducer = \"head.yaml\"\nproducer_base = \"base.yaml\"\nconsumer = \"client.py\"\n";
    let out = evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();
    assert_eq!(out[0].residue, vec!["legacy".to_string(), "size".to_string()], "only changed fields the consumer reads");
    let size = out[0].witnesses.iter().find(|w| w.residue == "size").unwrap();
    assert_eq!(size.sites, vec!["client.py:3".to_string()]);
    assert_eq!(size.type_change.as_ref().map(|t| (t.0.as_str(), t.1.as_str())), Some(("string", "integer")));
    let md = render_report_md(&out);
    assert!(md.contains("`legacy` read at `client.py:2` · not in `head.yaml`"), "{md}");
    assert!(md.contains("· type `string` -> `integer` in `head.yaml`"), "{md}");

    // Code producer (Go): tag rename is a removal of the old JSON name.
    w("old.go", "type Report struct {\n\tWarnings []string `json:\"warnings\"`\n\tCount int `json:\"count,omitempty\"`\n\tName string\n}\n");
    w("new.go", "type Report struct {\n\tFindings []string `json:\"findings\"`\n\tCount string `json:\"count\"`\n\tName string\n}\n");
    w("consumer.ts", "const r = JSON.parse(out);\nfor (const x of r.warnings) use(x);\nlet c: number = r.count;\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"g\"\nproducer = \"new.go\"\nproducer_base = \"old.go\"\nconsumer = \"consumer.ts\"\n";
    let out = evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();
    assert_eq!(out[0].residue, vec!["count".to_string(), "warnings".to_string()]);

    // Dual source: spec unchanged, code changed. The code catches it.
    w("same.yaml", "properties:\n  events: {type: array}\n");
    w("v1.rs", "pub struct R {\n    #[serde(rename = \"events\")]\n    pub evs: Vec<E>,\n}\n");
    w("v2.rs", "pub struct R {\n    #[serde(rename = \"final_events\")]\n    pub evs: Vec<E>,\n}\n");
    w("reader.js", "const e = data.events || [];\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"d\"\nproducer = \"same.yaml\"\nproducer_base = \"same.yaml\"\nproducer_code = \"v2.rs\"\nproducer_code_base = \"v1.rs\"\nconsumer = \"reader.js\"\n";
    let out = evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();
    assert_eq!(out[0].residue, vec!["events".to_string()]);

    // Extraction units.
    let py = field_names(&code_fields("m.py", "class M(Base):\n    is_ok: bool = field(alias=\"isOk\")\n    n: int\n").unwrap());
    assert!(py.contains("isOk") && py.contains("n"));
    let reads: Vec<String> = broad_reads("x = a?.b + c['d'] + e.get(\"f\")\nG string `json:\"g\"`\nh: int\n")
        .into_iter()
        .map(|(f, _)| f)
        .collect();
    for want in ["b", "d", "f", "g", "h"] {
        assert!(reads.iter().any(|r| r == want), "missing {want} in {reads:?}");
    }

    // Half a code pair is a configuration ERROR, never a verdict.
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"h\"\nproducer = \"same.yaml\"\nproducer_code = \"v2.rs\"\nconsumer = \"reader.js\"\n";
    assert!(evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}


// ---------------------------------------------------------------------------
// Test 27 (alpha.4): naming forms match across case and `_`/`-`; payloads
// assembled at runtime (dict/map/object literal keys, keyed assignments) are
// part of a code producer's surface; the remote size limit is configurable
// within a hard ceiling.
// ---------------------------------------------------------------------------
#[test]
fn naming_forms_runtime_payloads_and_size_limit() {
    use invar_falsify::resolver::{curl_args_max, HARD_MAX_REMOTE_BYTES, MAX_REMOTE_BYTES};
    use invar_falsify::surface::{code_fields, field_names, name_key};
    let dir = std::env::temp_dir().join(format!("invar-alpha4-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();

    assert_eq!(name_key("is_food"), name_key("isFood"));
    assert_eq!(name_key("IsFood"), name_key("is-food"));
    assert_ne!(name_key("is_food"), name_key("is_foods"));

    // Python producer drops a dict key; consumer reads it in camelCase.
    w("v1.py", "def stats():\n    out = {\"uptime\": 1, \"cpu_temp\": t()}\n    out[\"disk_free\"] = d()\n    return out\n");
    w("v2.py", "def stats():\n    out = {\"uptime\": 1}\n    out[\"disk_free\"] = d()\n    return out\n");
    w("ui.js", "const t = data.cpuTemp;\nconst u = data.uptime;\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"p\"\nproducer = \"v2.py\"\nproducer_base = \"v1.py\"\nconsumer = \"ui.js\"\n";
    let out = evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();
    assert_eq!(out[0].residue, vec!["cpu_temp".to_string()]);
    assert_eq!(out[0].witnesses[0].read_as, vec!["cpuTemp".to_string()]);
    let md = render_report_md(&out);
    assert!(md.contains("`cpu_temp` read at `ui.js:1` · not in `v2.py` · naming differs, read as `cpuTemp`"), "{md}");

    // Runtime assembly in JS and Go.
    let js = field_names(&code_fields("e.js", "res.json({ status: ok, \"total\": n });\nbody.items = list;\n").unwrap());
    for k in ["status", "total", "items"] {
        assert!(js.contains(k), "js missing {k}: {js:?}");
    }
    let go = field_names(&code_fields("h.go", "m := map[string]any{\n\t\"grid_power\": p,\n}\n").unwrap());
    assert!(go.contains("grid_power"), "{go:?}");

    // Unrelated reads still do not become residue (control).
    w("ui2.js", "const n = data.length; x.map(f);\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"q\"\nproducer = \"v2.py\"\nproducer_base = \"v1.py\"\nconsumer = \"ui2.js\"\n";
    let out = evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();
    assert!(out[0].residue.is_empty());

    // Size limit: default 5 MB, explicit value reaches curl, hard ceiling 50 MB.
    assert_eq!(MAX_REMOTE_BYTES, 5_000_000);
    assert_eq!(HARD_MAX_REMOTE_BYTES, 50_000_000);
    let req = github_request(&parse_ref("github:o/r@main:a.json", None).unwrap(), None);
    let args = curl_args_max(&req, 12_000_000);
    let i = args.iter().position(|a| a == "--max-filesize").unwrap();
    assert_eq!(args[i + 1], "12000000");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Test 28 (alpha.4): fields are bound to their declaring scope, so same-named
// fields of different types in different structs do not collide, and a
// function parameter cannot mask a removed field; contracts touching an
// unsupported language are SKIPPED, never a pass.
// ---------------------------------------------------------------------------
#[test]
fn scoped_fields_and_skipped_languages() {
    let dir = std::env::temp_dir().join(format!("invar-scope-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();

    // Go: `state` keeps its type in PullRequest; an unrelated Option struct
    // has a `state` of another type. Only the removed `head_branch` is residue.
    w("old.go", "type PullRequest struct {\n\tState string `json:\"state\"`\n\tHeadBranch string `json:\"head_branch\"`\n}\n");
    w("new.go", "type PullRequest struct {\n\tState string `json:\"state\"`\n}\ntype EditOption struct {\n\tState *int `json:\"state\"`\n}\n");
    w("bot.go", "type Hook struct {\n\tState string `json:\"state\"`\n\tHead string `json:\"head_branch\"`\n}\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"g\"\nproducer = \"new.go\"\nproducer_base = \"old.go\"\nconsumer = \"bot.go\"\n";
    let out = evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();
    assert_eq!(out[0].residue, vec!["head_branch".to_string()], "no cross-struct type collision");

    // Python: a dict key is removed; a same-named method parameter remains.
    w("e1.py", "class E:\n    def __init__(\n        self,\n        line_no: int = 0,\n    ):\n        self.line_no = line_no\n\n    def info(self):\n        return {\"line_no\": self.line_no}\n");
    w("e2.py", "class E:\n    def __init__(\n        self,\n        line_no: int = 0,\n    ):\n        self.line_no = line_no\n\n    def to_dict(self):\n        return {\"start_line_no\": self.line_no}\n");
    w("view.vim", "let l:n = l:violation.line_no\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"p\"\nproducer = \"e2.py\"\nproducer_base = \"e1.py\"\nconsumer = \"view.vim\"\n";
    let out = evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();
    assert_eq!(out[0].residue, vec!["line_no".to_string()], "parameter must not mask the removed key");

    // Unsupported language on either side: SKIPPED, nothing fetched.
    for (producer, consumer) in [("rpc.cpp", "ui.js"), ("new.go", "App.swift"), ("Svc.cs", "ui.js"), ("svc.php", "ui.js"), ("Order.java", "ui.js"), ("new.go", "Client.kt"), ("api.rb", "ui.js")] {
        let cfg = format!(
            "[[invariant]]\nkind = \"schema\"\nid = \"s\"\nproducer = \"{producer}\"\nproducer_base = \"{producer}\"\nconsumer = \"{consumer}\"\n"
        );
        let out = evaluate(&parse_sections("invar.toml", &cfg).unwrap(), &Resolver::local(&dir)).unwrap();
        let why = out[0].skipped.as_deref().unwrap_or("");
        assert!(why.starts_with("language not supported"), "{producer}/{consumer}: {why}");
        assert!(out[0].residue.is_empty() && out[0].provenance.is_empty());
        let md = render_report_md(&out);
        assert!(md.contains("NOT FULLY EVALUATED (1 of 1 not evaluated)") && md.contains("| SKIPPED |"), "{md}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 29. R2 invar.lock: the locked slice replaces producer_base; Type.field
// residue; waivers show days left and fail once expired; dates are strict;
// the file round-trips and re-locking an unchanged producer rewrites nothing.
// ---------------------------------------------------------------------------
#[test]
fn lockfile_slice_waivers_and_roundtrip() {
    use invar_falsify::check::evaluate_with;
    use invar_falsify::lock::{self, parse_date, Lock, Waiver};
    let dir = std::env::temp_dir().join(format!("invar-lock-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();
    w("api.json", r#"{"components":{"schemas":{"Order":{"properties":{"amount_cents":{"type":"integer"},"currency":{"type":"string"},"note":{"type":"string"}}}}}}"#);
    w("order.go", "type Order struct {\n\tAmountCents int64 `json:\"amount_cents\"`\n}\ntype Refund struct {\n\tAmountCents string `json:\"amount_cents\"`\n}\n");
    w("client.js", "const a = order.amount_cents;\nconst c = order.currency;\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"orders\"\nproducer = \"api.json\"\npointer = \"/components/schemas/Order\"\nproducer_code = \"order.go\"\nconsumer = \"client.js\"\n";
    let sections = parse_sections("invar.toml", cfg).unwrap();
    let r = Resolver::local(&dir);
    let day = parse_date("2026-10-08").unwrap();

    let l = lock::compute(&sections, &r, &Lock::default(), day).unwrap();
    let shown: Vec<String> = l.contracts[0].bindings.iter().map(|b| format!("{} {}", b.source, b.display())).collect();
    assert_eq!(shown, ["code Order.amount_cents", "code Refund.amount_cents", "spec Order.amount_cents", "spec Order.currency"], "unread `note` is not bound");
    let text = lock::render(&l).unwrap();
    assert_eq!(lock::parse("invar.lock", &text).unwrap(), l, "round-trip");
    let again = lock::compute(&sections, &r, &l, day + 3).unwrap();
    assert_eq!(again, l, "unchanged producer: same t_r and t_v");
    assert!(lock::diff(&l, &again).is_empty());

    // Unchanged producer holds. Producer retypes Order.amount_cents (not Refund's).
    let out = evaluate_with(&sections, &r, Some(&l), day).unwrap();
    assert!(out[0].residue.is_empty() && out[0].notes[0].starts_with("producer unchanged"));
    w("order.go", "type Order struct {\n\tAmountCents string `json:\"amount_cents\"`\n}\ntype Refund struct {\n\tAmountCents string `json:\"amount_cents\"`\n}\n");
    let out = evaluate_with(&sections, &r, Some(&l), day).unwrap();
    assert_eq!(out[0].residue, ["Order.amount_cents"]);
    assert_eq!(out[0].witnesses[0].type_change.as_ref().map(|t| (t.0.as_str(), t.1.as_str())), Some(("int64", "string")));

    // Waiver: active -> WAIVED with days left, holds; expired -> violated.
    let mut lw = l.clone();
    lw.waivers.push(Waiver { contract: "orders".into(), field: "Order.amount_cents".into(), owner: "nikhil".into(), reason: "migration #42".into(), expires: "2026-11-01".into() });
    let out = evaluate_with(&sections, &r, Some(&lw), day).unwrap();
    assert!(!out[0].violated() && out[0].waived[0].3 == 24, "{:?}", out[0]);
    assert!(render_report_md(&out).contains("WAIVED `Order.amount_cents` (24 days remaining"));
    let out = evaluate_with(&sections, &r, Some(&lw), parse_date("2026-11-01").unwrap()).unwrap();
    assert!(out[0].violated() && out[0].expired.len() == 1, "expires day fails closed");
    assert!(render_report_md(&out).contains("WAIVER EXPIRED"));
    // 90-day cap: 91 days away is rejected (an ERROR), 90 is allowed.
    let mut capped = lw.clone();
    capped.waivers[0].expires = lock::format_date(day + 91);
    assert!(evaluate_with(&sections, &r, Some(&capped), day).is_err(), "over the 90-day cap");
    capped.waivers[0].expires = lock::format_date(day + 90);
    assert!(evaluate_with(&sections, &r, Some(&capped), day).is_ok());
    lw.waivers[0].contract = "nope".into();
    assert!(evaluate_with(&sections, &r, Some(&lw), day).is_err(), "waiver for unknown contract");

    // The lock replaces producer_base; a different producer in invar.toml is an error.
    let moved = cfg.replace("api.json", "api2.json");
    w("api2.json", "{}");
    assert!(evaluate_with(&parse_sections("invar.toml", &moved).unwrap(), &r, Some(&l), day).is_err());

    // Strict dates and file errors.
    for bad in ["2026-02-30", "2026-13-01", "26-10-08", "2026-10-8"] {
        assert!(parse_date(bad).is_none(), "{bad}");
    }
    assert_eq!(lock::format_date(day), "2026-10-08");
    assert!(lock::parse("x", "[lock]\nformat = \"9\"\n").is_err());
    assert!(lock::parse("x", "[lock]\nformat = \"1\"\n[[waiver]]\ncontract = \"a\"\nfield = \"f\"\nowner = \"o\"\nreason = \"\"\nexpires = \"2026-01-01\"\n").is_err(), "empty reason");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 30. R2.1/R2.2: `invar fix --suggest` proposes a patch only for a 1:1 rename
// in the same type; `invar mcp` answers check/contract read-only.
// ---------------------------------------------------------------------------
#[test]
fn fix_suggest_and_mcp_server() {
    use invar_falsify::lock::{self, parse_date, Lock};
    use invar_falsify::mcp::Server;
    let dir = std::env::temp_dir().join(format!("invar-fix-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();
    w("order.go", "type Order struct {\n\tAmountCents int64 `json:\"amount_cents\"`\n\tCurrency string `json:\"currency\"`\n}\n");
    w("client.js", "const t = 0; // amount_cents_total stays\nconst a = order.amount_cents;\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"orders\"\nproducer = \"order.go\"\nconsumer = \"client.js\"\n";
    w("invar.toml", cfg);
    let sections = parse_sections("invar.toml", cfg).unwrap();
    let r = Resolver::local(&dir);
    let day = parse_date("2026-10-08").unwrap();
    let l = lock::compute(&sections, &r, &Lock::default(), day).unwrap();
    std::fs::write(dir.join("invar.lock"), lock::render(&l).unwrap()).unwrap();

    let run = |go: &str| {
        w("order.go", go);
        let res = lock::check_contract(&l.contracts[0], "order.go", None, None, "client.js", &r, &mut Vec::new()).unwrap();
        let residue: Vec<(String, bool)> = res.witnesses.iter().map(|x| (x.residue.clone(), x.missing_from.is_some() && x.type_change.is_none())).collect();
        invar_falsify::fix::suggest(&l.contracts[0], &res.live_members, &residue, "client.js", &std::fs::read_to_string(dir.join("client.js")).unwrap())
    };
    let s = run("type Order struct {\n\tAmountMinor int64 `json:\"amount_minor\"`\n\tCurrency string `json:\"currency\"`\n}\n");
    assert_eq!(s[0].rename_to.as_deref(), Some("amount_minor"));
    let p = s[0].patch.as_deref().unwrap();
    assert!(p.contains("@@ -2 +2 @@\n-const a = order.amount_cents;\n+const a = order.amount_minor;"), "{p}");
    assert!(!p.contains("amount_cents_total stays\n+"), "whole identifiers only");
    let s = run("type Order struct {\n\tAmountMinor int64 `json:\"amount_minor\"`\n\tCurrency string `json:\"currency\"`\n\tTax int `json:\"tax\"`\n}\n");
    assert!(s[0].rename_to.is_none() && s[0].note.as_deref().unwrap().contains("not a 1:1 rename"), "two additions: ambiguous");
    let s = run("type Order struct {\n\tAmountCents string `json:\"amount_cents\"`\n\tCurrency string `json:\"currency\"`\n}\n");
    assert!(s[0].patch.is_none() && s[0].note.as_deref().unwrap().starts_with("type change"));

    // MCP: initialize, list, both tools, unknown method; notifications get no reply.
    let server = Server { config: dir.join("invar.toml"), transport: None, token: None, today: day };
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#, "\n",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#, "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#, "\n",
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"check"}}"#, "\n",
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"contract"}}"#, "\n",
        r#"{"jsonrpc":"2.0","id":5,"method":"nope"}"#, "\n",
    );
    let mut out = Vec::new();
    server.serve(input.as_bytes(), &mut out).unwrap();
    let replies: Vec<serde_json::Value> = String::from_utf8(out).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(replies.len(), 5, "no reply to a notification");
    assert_eq!(replies[0]["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(replies[1]["result"]["tools"].as_array().unwrap().len(), 2);
    let check = &replies[2]["result"]["structuredContent"];
    assert_eq!(check["result"], "violated", "{check}");
    assert_eq!(check["contracts"][0]["residue"][0]["field"], "Order.amount_cents");
    assert!(replies[3]["result"]["structuredContent"]["contracts"][0]["bindings"].as_array().unwrap().iter().any(|b| b["field"] == "Order.amount_cents"));
    assert_eq!(replies[4]["error"]["code"], -32601);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 31. A1 stated blind spots: a changed field the consumer only mentions is NOT
// EVALUATED (not HOLDS); a field moved between types is NOT EVALUATED;
// nullability is part of a spec type.
// ---------------------------------------------------------------------------
#[test]
fn a1_states_blind_spots() {
    let dir = std::env::temp_dir().join(format!("invar-a1-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();
    let run = |cfg: &str| evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();

    // Destructuring read of a removed field: not a recognised read, but stated.
    w("old.ts", "interface Info {\n  permit_join_timeout: number;\n  version: string;\n}\n");
    w("new.ts", "interface Info {\n  version: string;\n}\n");
    w("ui.tsx", "const { permit_join_timeout: t } = info;\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"d\"\nproducer = \"new.ts\"\nproducer_base = \"old.ts\"\nconsumer = \"ui.tsx\"\n");
    assert!(out[0].residue.is_empty() && !out[0].violated());
    assert_eq!(out[0].unevaluated.len(), 1, "{:?}", out[0]);
    assert!(out[0].unevaluated[0].contains("permit_join_timeout changed; mentioned at ui.tsx:1"));
    assert!(render_report_md(&out).contains("NOT EVALUATED"));

    // Moved between types: stated, not residue.
    w("m1.go", "type Meta struct {\n\tFiles []string `json:\"files\"`\n}\ntype Pkg struct {\n\tName string `json:\"name\"`\n}\n");
    w("m2.go", "type Meta struct {\n\tHash string `json:\"hash\"`\n}\ntype Pkg struct {\n\tName string `json:\"name\"`\n\tFiles []string `json:\"files\"`\n}\n");
    w("c.py", "x = lock[\"metadata\"][\"files\"]\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"m\"\nproducer = \"m2.go\"\nproducer_base = \"m1.go\"\nconsumer = \"c.py\"\n");
    assert!(out[0].residue.is_empty());
    assert!(out[0].unevaluated.iter().any(|u| u.starts_with("files moved from Meta to Pkg")), "{:?}", out[0].unevaluated);

    // Nullability is a type change.
    w("s1.json", r#"{"properties":{"limit":{"type":"integer"}}}"#);
    w("s2.json", r#"{"properties":{"limit":{"type":"integer","nullable":true}}}"#);
    w("r.py", "if data[\"limit\"] > 0:\n    pass\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"n\"\nproducer = \"s2.json\"\nproducer_base = \"s1.json\"\nconsumer = \"r.py\"\n");
    assert_eq!(out[0].residue, ["limit"]);
    let tc = out[0].witnesses[0].type_change.clone().unwrap();
    assert_eq!((tc.0.as_str(), tc.1.as_str()), ("integer", "integer|null"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 32. D1: a TypeScript type introduced at head that Omits a field removes it;
// Python unannotated class fields (`x = SomethingField()`) are fields.
// ---------------------------------------------------------------------------
#[test]
fn d1_omit_and_python_assigned_fields() {
    let dir = std::env::temp_dir().join(format!("invar-d1-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();
    let run = |cfg: &str| evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();

    let base = "type Modify<T, R> = Omit<T, keyof R> & R;\ninterface State31 {\n  isHealNetworkActive: boolean;\n  homeId: number;\n}\ntype Old = Omit<State31, \"homeId\">;\n";
    w("s1.ts", base);
    w("s2.ts", &format!("{base}type State32 = Omit<\n  Modify<State31, {{ isRebuildingRoutes: boolean }}>,\n  \"isHealNetworkActive\"\n>;\n"));
    w("c.py", "active = state.get(\"isHealNetworkActive\")\nhid = state[\"homeId\"]\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"o\"\nproducer = \"s2.ts\"\nproducer_base = \"s1.ts\"\nconsumer = \"c.py\"\n");
    assert_eq!(out[0].residue, ["isHealNetworkActive"], "only the Omit in the new type counts");
    assert!(out[0].witnesses[0].missing_from.as_deref().unwrap().contains("Omit in State32"));

    w("m1.py", "class DeviceSerializer(Serializer):\n    device_role = NestedDeviceRoleSerializer()\n    name = serializers.CharField(max_length=64)\n    MAX = Limit(3)\n");
    w("m2.py", "class DeviceSerializer(Serializer):\n    role = NestedDeviceRoleSerializer()\n    name = serializers.CharField(max_length=64)\n    MAX = Limit(3)\n");
    w("u.py", "dev = api_result[\"device_role\"]\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"p\"\nproducer = \"m2.py\"\nproducer_base = \"m1.py\"\nconsumer = \"u.py\"\n");
    assert_eq!(out[0].residue, ["device_role"]);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 33. E1: a key moved into a new envelope is residue unless the consumer
// already mentions the new envelope key.
// ---------------------------------------------------------------------------
#[test]
fn e1_envelope_relocation() {
    let dir = std::env::temp_dir().join(format!("invar-e1-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();
    let run = |cfg: &str| evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();
    w("a1.py", "def forward(msg):\n    send({\n        \"level\": msg.level,\n        \"message\": msg.text,\n    })\n");
    w("a2.py", "def forward(msg):\n    send({\n        \"type\": \"log_message\",\n        \"log_message\": {\n            \"level\": msg.level,\n            \"message\": msg.text,\n        },\n    })\n");
    w("old.ts", "lines.push(log.message);\n");
    w("new.ts", "lines.push(update.log_message.message);\n");
    let cfg = |c: &str| format!("[[invariant]]\nkind = \"schema\"\nid = \"e\"\nproducer = \"a2.py\"\nproducer_base = \"a1.py\"\nconsumer = \"{c}\"\n");
    let out = run(&cfg("old.ts"));
    assert_eq!(out[0].residue, ["message"], "{:?}", out[0]);
    assert!(out[0].witnesses[0].missing_from.as_deref().unwrap().contains("moved from the top level to log_message"));
    let out = run(&cfg("new.ts"));
    assert!(out[0].residue.is_empty(), "consumer already reads the envelope: {:?}", out[0]);

    // Keyed by a variable (Python): the new segment is "*".
    w("b1.py", "def info(d):\n    return {\"url\": d.url, \"channel\": d.channel}\n");
    w("b2.py", "def info(d, ext):\n    return {ext: {\"url\": d.url, \"channel\": d.channel}}\n");
    w("c.ts", "const u = info.url;\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"k\"\nproducer = \"b2.py\"\nproducer_base = \"b1.py\"\nconsumer = \"c.ts\"\n");
    assert_eq!(out[0].residue, ["url"]);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 34. F1: a default version/format selector that changed value turns a
// would-be HOLDS into NOT FULLY EVALUATED; it never claims VIOLATED, and
// adds nothing when there is residue.
// ---------------------------------------------------------------------------
#[test]
fn f1_default_switch_is_stated_never_silent() {
    let dir = std::env::temp_dir().join(format!("invar-f1-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();
    let run = |cfg: &str| evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();

    // Assignment form (r1-075 shape): the field survives, the default flips.
    w("w1.js", "const defaultLockfileVersion = 2\nmodule.exports = { dependencies: tree(), lockfileVersion }\n");
    w("w2.js", "const defaultLockfileVersion = 3\nmodule.exports = { dependencies: tree(), lockfileVersion }\n");
    w("r.js", "check(lock.dependencies)\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"f\"\nproducer = \"w2.js\"\nproducer_base = \"w1.js\"\nconsumer = \"r.js\"\n");
    assert!(out[0].residue.is_empty() && !out[0].violated(), "{:?}", out[0]);
    assert_eq!(out[0].unevaluated.len(), 1, "{:?}", out[0].unevaluated);
    assert!(out[0].unevaluated[0].contains("default selector 'defaultlockfileversion' changed 2 -> 3"), "{}", out[0].unevaluated[0]);

    // Quoted-key form (r1-099 shape): a second emitted version appears while
    // the old line survives.
    w("q1.rs", "pub struct D { pub rrname: String }\nfn log(js: &mut J) { js.set_uint(\"version\", 2); }\n");
    w("q2.rs", "pub struct D { pub rrname: String }\nfn log(js: &mut J) { js.set_uint(\"version\", 2); }\nfn log3(jb: &mut J) { jb.set_int(\"version\", 3); }\n");
    w("fmt.ts", "lines.push(dns.rrname);\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"q\"\nproducer = \"q2.rs\"\nproducer_base = \"q1.rs\"\nconsumer = \"fmt.ts\"\n");
    assert!(out[0].residue.is_empty());
    assert!(out[0].unevaluated[0].contains("'version' changed 2 -> 3"), "{:?}", out[0].unevaluated);

    // defaultValue form (r1-100 shape), boolean flip across pattern kinds.
    w("v1.ts", "interface Q { state: string }\nconst q = { v8format: schema.boolean({ defaultValue: false }) };\n");
    w("v2.ts", "interface Q { state: string }\nconst { v8format = true } = req.query;\n");
    w("t.go", "s.State = payload.state\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"v\"\nproducer = \"v2.ts\"\nproducer_base = \"v1.ts\"\nconsumer = \"t.go\"\n");
    assert!(out[0].unevaluated.iter().any(|u| u.contains("'v8format' changed false -> true")), "{:?}", out[0].unevaluated);

    // Control: no selector change, no F1 noise.
    w("c1.js", "const defaultLockfileVersion = 3\nexports.total = 1\n");
    w("c2.js", "const defaultLockfileVersion = 3\nexports.total = 2\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"c\"\nproducer = \"c2.js\"\nproducer_base = \"c1.js\"\nconsumer = \"r.js\"\n");
    assert!(out[0].unevaluated.is_empty() && out[0].residue.is_empty(), "{:?}", out[0]);

    // With residue, F1 adds nothing: VIOLATED stands alone.
    w("x1.py", "class S:\n    api_version = 1\n    total: int\n");
    w("x2.py", "class S:\n    api_version = 2\n");
    w("u.py", "t = d[\"total\"]\n");
    let out = run("[[invariant]]\nkind = \"schema\"\nid = \"x\"\nproducer = \"x2.py\"\nproducer_base = \"x1.py\"\nconsumer = \"u.py\"\n");
    assert_eq!(out[0].residue, ["total"]);
    assert!(out[0].unevaluated.is_empty(), "{:?}", out[0].unevaluated);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 35. A2: receipts are byte-deterministic; the body hash seals the file; the
// recorded verdict and artifact hashes are what verification reproduces.
// ---------------------------------------------------------------------------
#[test]
fn a2_receipt_is_canonical_and_sealed() {
    use invar_falsify::receipt;
    let dir = std::env::temp_dir().join(format!("invar-a2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();
    w("s.json", r#"{"properties":{"total":{"type":"integer"}}}"#);
    w("c.py", "t = d[\"total\"]\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"a\"\nproducer = \"s.json\"\nconsumer = \"c.py\"\n";
    let out = evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();

    // Determinism: same inputs, same bytes, twice.
    let b1 = receipt::body("0.0.0-test", "2026-10-10", "invar.toml", "cfg-hash", None, &out);
    let b2 = receipt::body("0.0.0-test", "2026-10-10", "invar.toml", "cfg-hash", None, &out);
    let (r1, r2) = (receipt::render(&b1), receipt::render(&b2));
    assert_eq!(r1, r2, "two renders of the same evaluation must be identical");
    assert!(r1.ends_with("\"}\n") && r1.contains("\"receipt_format\": 1"));

    // The footer seals the body: parse succeeds, tampering fails.
    let parsed = receipt::parse(&r1).unwrap();
    assert_eq!(parsed.body.pointer("/result/verdict").and_then(|v| v.as_str()), Some("all_hold"));
    let tampered = r1.replace("all_hold", "violated");
    assert!(receipt::parse(&tampered).is_err(), "a flipped verdict without a recomputed footer is corruption");

    // A recomputed footer over a flipped verdict parses, but the body no
    // longer matches a re-evaluation (what verify-receipt compares).
    let flipped = receipt::render(&serde_json::from_str::<serde_json::Value>(
        &parsed.body_text.replace("all_hold", "violated")).unwrap());
    let reparsed = receipt::parse(&flipped).unwrap();
    assert_ne!(reparsed.body_text, parsed.body_text);

    // contracts_exit mirrors the check exit codes (0/1/4), never the ledger.
    assert_eq!(receipt::contracts_exit(&out), 0);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 36. A2.1: the in-toto export wraps the format 1 body verbatim; the SSH
// verify helper accepts a good signature and rejects a bad one (skipped
// with a stated reason when ssh-keygen is absent).
// ---------------------------------------------------------------------------
#[test]
fn a21_in_toto_export_and_ssh_verify() {
    use invar_falsify::receipt;
    let dir = std::env::temp_dir().join(format!("invar-a21-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = |n: &str, b: &str| std::fs::write(dir.join(n), b).unwrap();
    w("s.json", r#"{"properties":{"total":{"type":"integer"}}}"#);
    w("c.py", "t = d[\"total\"]\n");
    let cfg = "[[invariant]]\nkind = \"schema\"\nid = \"a\"\nproducer = \"s.json\"\nconsumer = \"c.py\"\n";
    let out = evaluate(&parse_sections("invar.toml", cfg).unwrap(), &Resolver::local(&dir)).unwrap();
    let body = receipt::body("0.0.0-test", "2026-10-10", "invar.toml", "cfg-hash", Some("lock-hash"), &out);
    let st = receipt::to_in_toto(&body);
    assert_eq!(st.get("_type").and_then(|v| v.as_str()), Some("https://in-toto.io/Statement/v1"));
    assert_eq!(st.get("predicate"), Some(&body), "the predicate is the format 1 body verbatim");
    let subj = st.get("subject").and_then(|v| v.as_array()).unwrap();
    assert!(subj.iter().any(|s| s.get("name").and_then(|v| v.as_str()) == Some("invar.lock")));
    assert!(subj.iter().any(|s| s.get("name").and_then(|v| v.as_str()) == Some("invar.toml")));

    // SSH round-trip, only where ssh-keygen exists (CI has it; the gate is
    // the real proof, this is the unit check).
    if std::process::Command::new("ssh-keygen").arg("-h").output().is_err() {
        eprintln!("SKIPPED ssh round-trip: ssh-keygen not on PATH");
        let _ = std::fs::remove_dir_all(&dir);
        return;
    }
    let key = dir.join("k");
    let runs = std::process::Command::new("ssh-keygen")
        .args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(&key).status().unwrap().success();
    assert!(runs);
    w("payload.txt", "locked bytes\n");
    let ok = std::process::Command::new("ssh-keygen")
        .args(["-Y", "sign", "-n", "invar-lock", "-f"]).arg(&key).arg(dir.join("payload.txt"))
        .status().unwrap().success();
    assert!(ok);
    let signers = dir.join("signers");
    let pubkey = std::fs::read_to_string(key.with_extension("pub")).unwrap();
    std::fs::write(&signers, format!("release@invar {}", pubkey)).unwrap();
    let good = receipt::verify_ssh(b"locked bytes\n", &dir.join("payload.txt.sig"), &signers, "release@invar", "invar-lock").unwrap();
    assert!(good, "a valid signature verifies");
    let bad = receipt::verify_ssh(b"tampered bytes\n", &dir.join("payload.txt.sig"), &signers, "release@invar", "invar-lock").unwrap();
    assert!(!bad, "tampered payload is rejected");
    let wrong_ns = receipt::verify_ssh(b"locked bytes\n", &dir.join("payload.txt.sig"), &signers, "release@invar", "invar-receipt").unwrap();
    assert!(!wrong_ns, "namespace separation holds");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 37. A4: the dependents graph answers "who depends on Type.field" from
// local lockfiles only, deterministically; zero matches is an answer.
// ---------------------------------------------------------------------------
#[test]
fn a4_dependents_graph_over_local_locks() {
    use invar_falsify::dependents;
    use invar_falsify::lock::{self, parse_date, Lock};
    let root = std::env::temp_dir().join(format!("invar-a4-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let day = parse_date("2026-10-09").unwrap();
    // Two consumer repos locking the same producer field, one unrelated.
    for (repo, reads) in [
        ("team-a/billing", "const a = order.amount_cents;\n"),
        ("team-b/reports", "total += order.amountCents;\nconst c = order.currency;\n"),
        ("team-c/other", "const n = user.name;\n"),
    ] {
        let dir = root.join(repo);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("api.json"), r#"{"components":{"schemas":{"Order":{"properties":{"amount_cents":{"type":"integer"},"currency":{"type":"string"}}},"User":{"properties":{"name":{"type":"string"}}}}}"#.to_string()+"}").unwrap();
        std::fs::write(dir.join("c.js"), reads).unwrap();
        let ptr = if repo.contains("other") { "/components/schemas/User" } else { "/components/schemas/Order" };
        let cfg = format!("[[invariant]]\nkind = \"schema\"\nid = \"x\"\nproducer = \"api.json\"\npointer = \"{ptr}\"\nconsumer = \"c.js\"\n");
        let sections = parse_sections("invar.toml", &cfg).unwrap();
        let l = lock::compute(&sections, &Resolver::local(&dir), &Lock::default(), day).unwrap();
        std::fs::write(dir.join("invar.lock"), lock::render(&l).unwrap()).unwrap();
    }
    // Scoped query matches by naming form across both consumers.
    let a = dependents::find(&root, "Order.amountCents", None).unwrap();
    assert_eq!(a.lockfiles_scanned, 3);
    assert_eq!(a.lockfiles_matched, 2, "{:?}", a.dependents);
    let consumers: Vec<&str> = a.dependents.iter().map(|d| d.consumer.as_str()).collect();
    assert_eq!(consumers, ["team-a/billing/invar.lock", "team-b/reports/invar.lock"], "sorted and relative");
    // Bare field and producer filter.
    assert_eq!(dependents::find(&root, "currency", None).unwrap().dependents.len(), 1);
    assert!(dependents::find(&root, "amount_cents", Some("nonexistent")).unwrap().dependents.is_empty());
    // Zero matches is an answer; deterministic JSON.
    let none = dependents::find(&root, "Order.gone", None).unwrap();
    assert!(none.dependents.is_empty());
    let j1 = dependents::render_json("Order.amountCents", None, &a);
    let j2 = dependents::render_json("Order.amountCents", None, &dependents::find(&root, "Order.amountCents", None).unwrap());
    assert_eq!(j1, j2, "byte-identical for identical inputs");
    assert!(j1.contains("\"lockfiles_scanned\": 3"));
    // A directory with no lockfiles is an error, not an empty answer.
    let empty = root.join("empty"); std::fs::create_dir_all(&empty).unwrap();
    assert!(dependents::find(&empty, "x", None).is_err());
    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------
// 38. Observed reality with presence witnesses: shapes derive
// deterministically; format 3 roundtrips and format 2 reads as one
// observation; the verdict thresholds are exactly the frozen spec's.
// ---------------------------------------------------------------------------
#[test]
fn observed_reality_witnesses_formats_and_thresholds() {
    use invar_falsify::lock::{self, merge_observation, payload_shape, pii_suspects, witness_drift, Sample};
    let sh = |j: &str| payload_shape(&serde_json::from_str(j).unwrap());
    let s1 = sh(r#"{"total": 3, "user": {"email": "x@y.z", "age": null}, "items": [{"id": 1}, {"id": "a"}]}"#);
    assert_eq!(s1.iter().find(|(q, _)| q == "items[].id").map(|(_, t)| t.as_str()), Some("integer|string"));
    assert_eq!(pii_suspects(&s1), ["user.email"]);

    // Merge three observations: `coupon` appears only in the second.
    let mut sm = Sample { contract: "orders".into(), blake3: String::new(), taken: String::new(), window_start: String::new(), observations: 0, witnesses: Vec::new() };
    let base = sh(r#"{"total": 1, "state": "ok"}"#);
    merge_observation(&mut sm, &base, "2026-10-01", "aa");
    merge_observation(&mut sm, &sh(r#"{"total": 2, "state": "ok", "coupon": "X"}"#), "2026-10-05", "bb");
    merge_observation(&mut sm, &base, "2026-10-11", "cc");
    assert_eq!(sm.observations, 3);
    let g = |p: &str| sm.witnesses.iter().find(|w| w.path == p).unwrap();
    assert_eq!((g("total").present, g("total").absent), (3, 0));
    assert_eq!((g("coupon").present, g("coupon").absent), (1, 2), "absent before first sighting counts");
    assert_eq!((sm.window_start.as_str(), sm.taken.as_str()), ("2026-10-01", "2026-10-11"));

    // Thresholds: always-present-now-gone is residue; optional-absent and
    // additions are stated or ignored, never residue.
    let fresh = sh(r#"{"state": "ok", "extra": true}"#);
    let (residue, stated) = witness_drift(&sm, &fresh, "fresh.json");
    assert_eq!(residue.len(), 1, "{residue:?}");
    assert_eq!(residue[0].0, "total");
    assert!(stated.iter().any(|x| x.contains("coupon") && x.contains("[not-evaluated:optional-field-absent]")), "{stated:?}");
    assert!(!stated.iter().any(|x| x.contains("extra")), "additions are never drift");

    // Below three observations: too few to accuse, stated.
    let mut young = Sample { contract: "o".into(), blake3: String::new(), taken: String::new(), window_start: String::new(), observations: 0, witnesses: Vec::new() };
    merge_observation(&mut young, &base, "2026-10-01", "aa");
    merge_observation(&mut young, &base, "2026-10-02", "bb");
    let (r2, s2) = witness_drift(&young, &sh(r#"{"state": "ok"}"#), "f.json");
    assert!(r2.is_empty());
    assert!(s2.iter().any(|x| x.contains("[not-evaluated:witness-count-low]")), "{s2:?}");

    // Retype of an always-present field at N>=3 is residue.
    let (r3, _) = witness_drift(&sm, &sh(r#"{"total": "3", "state": "ok"}"#), "f.json");
    assert!(r3.iter().any(|(p, _)| p == "total"), "{r3:?}");

    // Format 3 roundtrips; format 2 parses as one observation; 4 refused.
    let mut l = lock::Lock::default();
    l.samples.push(sm.clone());
    let text = lock::render(&l).unwrap();
    assert!(text.contains("format = \"3\"") && text.contains("observations = \"3\""));
    assert_eq!(lock::parse("invar.lock", &text).unwrap().samples, l.samples);
    let f2 = "[lock]\nformat = \"2\"\n\n[[sample]]\ncontract = \"orders\"\nblake3 = \"aa\"\ntaken = \"2026-10-01\"\nfield = \"total integer\"\nfield = \"age null\"\n";
    let back = lock::parse("l", f2).unwrap();
    assert_eq!(back.samples[0].observations, 1);
    assert_eq!(back.samples[0].witnesses[1].null, 1, "format 2 null type counts as one null witness");
    assert!(lock::parse("l", "[lock]\nformat = \"4\"\n").is_err());
    assert!(lock::render(&lock::Lock::default()).unwrap().contains("format = \"1\""));
}
