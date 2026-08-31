//! Acceptance tests for the Phase-1 falsification instrument.
//!
//! Exactly six tests, split between the classifier's epistemic honesty and the
//! bi-temporal ledger's integrity. These are the ground truth the crate is
//! written to satisfy.

use invar_falsify::evaluator::{classify, LegState, LegVerdict};
use invar_falsify::ledger::{Ledger, LedgerError};
use invar_falsify::manifest::{
    EpistemicStatus, EvidenceTier, FailureType, TestCase,
};

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
