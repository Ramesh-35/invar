//! REAL-INCIDENT-001 end-to-end evaluation (GAP-D, run-time).
//!
//! Usage: real_incident <producer_schema.json> <consumer_source>
//!
//! Operates on bytes that were fetched and externally resolved out of band:
//! the two commit SHAs below were confirmed to resolve via raw.githubusercontent
//! (HTTP 200) with a negative control (a bogus SHA returns 404). This program
//! then runs the committed INVAR logic over those real bytes and prints the
//! verdict it produces. It does not decide the verdict; `classify` does.

use std::time::{SystemTime, UNIX_EPOCH};

use invar_falsify::evaluator::{admit_reality, classify, LegState, LegVerdict};
use invar_falsify::ledger::Ledger;
use invar_falsify::manifest::{EvidenceTier, FailureType, TestCase};
use invar_falsify::reality::{derive_probe_hash, differential_residue, verify_real_tier, RealArtifactProof};

const PRODUCER_REPO: &str = "github.com/Ramesh-35/invar-demo-producer";
const PRODUCER_COMMIT: &str = "7c0855709970092b4f913093f0b57a70923e0b1a";
const CONSUMER_REPO: &str = "github.com/Ramesh-35/invar-demo-consumer";
const CONSUMER_COMMIT: &str = "fe73889bf250275711339d065ecc37f21879fc72";
const CATCH_SIGNATURE: &str = "CROSS_REPO_FIELD_DROP:amount_cents";

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn now_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let producer = std::fs::read_to_string(&args[1]).expect("read producer schema");
    let consumer = std::fs::read_to_string(&args[2]).expect("read consumer source");

    // 1. Differential probe over the real bytes.
    let residue = differential_residue(&producer, &consumer);
    let probe_hash = derive_probe_hash(&residue);

    // 2. Real artifact proof, with the probe hash DERIVED from the fetched diff.
    let proof = RealArtifactProof {
        producer_repo: PRODUCER_REPO.to_string(),
        producer_commit: PRODUCER_COMMIT.to_string(),
        consumer_repo: CONSUMER_REPO.to_string(),
        consumer_commit: CONSUMER_COMMIT.to_string(),
        diff_probe_hash: probe_hash,
    };

    // 3. The real composite-baseline leg: semgrep executed on each repo in
    //    isolation and emitted only advisory token findings (none matched the
    //    cross-repo catch signature). OPA / OpenTelemetry / property SKIPPED.
    let legs = vec![
        LegVerdict {
            tool: "semgrep".to_string(),
            state: LegState::FAIL,
            findings: 3,
            caught_designated: false,
            summary: "semgrep 1.179.0 ran per-repo: consumer payload.amount_cents x2, producer token x1; no finding isolates the cross-repo contract".to_string(),
        },
        LegVerdict { tool: "opa".to_string(), state: LegState::SKIPPED, findings: 0, caught_designated: false, summary: "no policy input document".to_string() },
        LegVerdict { tool: "opentelemetry".to_string(), state: LegState::SKIPPED, findings: 0, caught_designated: false, summary: "no live collector/workload".to_string() },
        LegVerdict { tool: "property_testing".to_string(), state: LegState::SKIPPED, findings: 0, caught_designated: false, summary: "no bound runner".to_string() },
    ];

    // 4. The reality-admission gate, binding the proof to the recorded residue.
    let admitted = match verify_real_tier(EvidenceTier::REAL, Some(&proof), Some(probe_hash)) {
        Ok(()) => "ADMITTED",
        Err(_) => "REFUSED",
    };
    let gated_legs = admit_reality(&legs, EvidenceTier::REAL, Some(&proof), Some(probe_hash));

    // 5. The case and the verdict (classify decides, not us).
    let tc = TestCase {
        id: "REAL-INCIDENT-001".to_string(),
        description: "producer dropped amount_cents from defined properties; consumer still requires it".to_string(),
        cross_repository: true,
        failure_type: FailureType::AST_SCHEMA_DESYNC,
        evidence_tier: EvidenceTier::REAL,
        target_path: "./corpus/real".to_string(),
        catch_signature: Some(CATCH_SIGNATURE.to_string()),
        expected_status: None,
    };
    let (status, rationale) = classify(&tc, &gated_legs);

    // 6. Persist the verdict to a bi-temporal ledger and re-verify.
    //    NOTE: authoritative commit timestamps were not retrievable (the commit
    //    API was gated), so valid-time is recorded as the evaluation time. This
    //    is a stated limitation, not a hidden assumption.
    let db = std::env::temp_dir().join(format!("real_incident_{}.db", std::process::id()));
    let content = format!(
        "verdict={status:?}; residue={residue:?}; probe_hash={}; producer={}@{}; consumer={}@{}",
        hex(&probe_hash), PRODUCER_REPO, PRODUCER_COMMIT, CONSUMER_REPO, CONSUMER_COMMIT
    );
    let ledger = Ledger::open(db.to_str().unwrap()).expect("open ledger");
    let node = ledger.append_at(now_us(), now_us(), "real_verdict", &tc.id, &content).expect("append");
    let verified = ledger.verify().expect("verify");

    println!("== REAL-INCIDENT-001 ==");
    println!("producer: {PRODUCER_REPO}@{PRODUCER_COMMIT}");
    println!("consumer: {CONSUMER_REPO}@{CONSUMER_COMMIT}");
    println!("differential residue (consumer requires, producer does not define): {residue:?}");
    println!("derived BLAKE3 probe hash: {}", hex(&probe_hash));
    println!("reality-admission gate: {admitted}");
    println!("composite legs: semgrep=FAIL(3 advisory, caught_designated=false), opa/otel/property=SKIPPED");
    println!("VERDICT: {status:?}");
    println!("rationale: {rationale}");
    println!("ledger: node id={} lineage={} ({} node verified)", node.id, hex(&node.lineage_hash), verified);
    let _ = std::fs::remove_file(&db);
    let _ = std::fs::remove_file(db.with_extension("db-wal"));
    let _ = std::fs::remove_file(db.with_extension("db-shm"));
}
