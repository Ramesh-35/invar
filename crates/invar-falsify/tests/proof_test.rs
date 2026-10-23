//! INVAR-PROOF-001 r3 fixtures (docs/architecture/INVAR-PROOF-001.md, rulings
//! D1 to D9 and I1 to I3). GATE 1: written BEFORE any proof engine code, per
//! the fixtures-first mandate, and run against the unchanged engine to record
//! the failure map. Synthetic repositories in temporary directories, each a
//! real git repository, run through the real binary.
//!
//! Declared Gate 1 outcomes against the unchanged engine:
//! - FAIL (no `--emit-proof` / `--proof` flag exists; clap rejects the flag):
//!   PRF-F1 to F4, V1 to V3, E1 to E5, U1 to U11, M1, M2, M5.
//! - FAIL (src/proof.rs does not exist): PRF-M4.
//! - FAIL (named stub): PRF-M6 needs to reseal a proof with a foreign
//!   rule_fingerprint, which requires proof::canonical_digest (Class 4);
//!   Cargo.toml is Class 1, so a dev-dependency on blake3 is not available.
//!   The stub panics with that reason and gains its real body at Gate 2.
//! - PASS (invariance of the unchanged engine, must STAY green): PRF-N1,
//!   PRF-N2, PRF-N3, PRF-M3.
//!
//! Interpretations carried from the spec, for the Gate 5 ledger:
//! - P-INT-1: `[producer] repo` comes from an optional `repo` key on the
//!   egress invariant block ("declared name; NOT verified"); absent, "none".
//! - P-INT-2: R-P-4's "+1 per emit" is read as +1 per emitted CHANGE; an
//!   emit at an unchanged source state is a no-op that leaves the file
//!   byte-identical (otherwise R-P-1 / PRF-M1 is unsatisfiable).
//! - P-INT-3: with `--proof` and at least one [[proof.import]] table, an
//!   invar.toml with zero [[invariant]] blocks is not an error; plain
//!   `check` keeps today's behavior exactly (PRF-N3).

use std::path::{Path, PathBuf};
use std::process::Command;

// ------------------------------------------------------------- helpers

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_prf_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (rel, body) in files {
        put(&dir, rel, body);
    }
    dir
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_invar")
}

/// Run the binary in `dir`; return (exit code, stdout + stderr).
fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    invar_env(dir, args, &[])
}

fn invar_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, String) {
    let mut c = Command::new(bin());
    c.args(no_reuse(args)).current_dir(dir);
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn git(dir: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(["-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

/// git init + add + commit everything currently in the directory.
fn ginit(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

fn commit_all(dir: &Path, msg: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", msg]);
}

// ------------------------------------------------------- producer repos

const SCHEMAS: &str = "import { z } from \"zod\";\nexport const OrderResponse = z.object({\n  id: z.string(),\n  total_minor: z.number(),\n  currency_code: z.string(),\n  nested: z.object({ inner_ref: z.string() }),\n});\n";

const ROUTE: &str = "import { OrderResponse } from \"../../../../lib/schemas\";\nexport async function GET() {\n  const order = load();\n  return Response.json(OrderResponse.parse(order));\n}\n";

const PRODUCER_TOML: &str = "[[invariant]]\nkind = \"egress\"\nid = \"egress:OrderResponse\"\nproducer = \"lib/schemas.ts\"\nshape = \"OrderResponse\"\nboundary = \"app/api/v1/orders/route.ts\"\neligibility = \"versioned-path\"\nanchor = \"GET /v1/orders\"\n";

/// A committed, locked producer repository ready to emit.
fn producer(tag: &str) -> PathBuf {
    let d = ws(tag, &[("lib/schemas.ts", SCHEMAS), ("app/api/v1/orders/route.ts", ROUTE), ("invar.toml", PRODUCER_TOML)]);
    ginit(&d);
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "producer lock failed: {out}");
    d
}

fn emit(dir: &Path, extra: &[&str]) -> (i32, String) {
    let mut args = vec!["lock", "--emit-proof", "egress:OrderResponse"];
    args.extend_from_slice(extra);
    invar(dir, &args)
}

/// Accept a producer change: rewrite the lock from the live producer.
fn relock(dir: &Path) {
    let (c, out) = invar(dir, &["lock", "--update"]);
    assert_eq!(c, 0, "lock --update failed: {out}");
}

// ----------------------------------------------------- proof file helpers

fn proof_get(text: &str, key: &str) -> String {
    for l in text.lines() {
        if let Some(rest) = l.strip_prefix(&format!("{key} = \"")) {
            return rest.trim_end_matches('"').to_string();
        }
    }
    panic!("proof has no {key} line:\n{text}");
}

fn read_proof(dir: &Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap()
}

// ------------------------------------------------------- consumer repos

/// One [[proof.import]] table. `accepted`: Some((epoch, digest)).
fn import_block(proof: &str, contract: &str, anchor: &str, fields: &str, accepted: Option<(&str, &str)>, extra: &str) -> String {
    let mut s = format!(
        "[[proof.import]]\nproof = \"{proof}\"\ncontract_id = \"{contract}\"\nanchor = \"{anchor}\"\nfields = \"{fields}\"\n"
    );
    if let Some((e, dg)) = accepted {
        s.push_str(&format!("accepted_epoch = \"{e}\"\naccepted_digest = \"{dg}\"\n"));
    }
    s.push_str(extra);
    s
}

/// A consumer repository holding one proof copy and one import table.
fn consumer(tag: &str, proof_bytes: &str, fields: &str, accepted: Option<(&str, &str)>, extra: &str) -> PathBuf {
    let toml = import_block("vendor/orders.proof", "egress:OrderResponse", "GET /v1/orders", fields, accepted, extra);
    ws(tag, &[("vendor/orders.proof", proof_bytes), ("invar.toml", &toml)])
}

fn check_proof(dir: &Path) -> (i32, String) {
    let (c1, o1) = invar(dir, &["check", "--proof"]);
    let (c2, o2) = invar(dir, &["check", "--proof"]);
    assert_eq!((c1, &o1), (c2, &o2), "check --proof is not deterministic");
    (c1, o1)
}

/// Emit at the default path and return the accepted pair (epoch, digest).
fn emitted(dir: &Path) -> (String, String) {
    let (c, out) = emit(dir, &[]);
    assert_eq!(c, 0, "{out}");
    let p = read_proof(dir, "invar.proof");
    (proof_get(&p, "epoch"), proof_get(&p, "digest"))
}

// ------------------------------------------------------------- F: positives

#[test]
fn prf_f1_emit_epoch_1_and_consumer_check_holds() {
    let d = producer("f1");
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 0, "{out}");
    let p = read_proof(&d, "invar.proof");
    assert_eq!(proof_get(&p, "format"), "1");
    assert_eq!(proof_get(&p, "level"), "0");
    assert_eq!(proof_get(&p, "epoch"), "1");
    assert_eq!(proof_get(&p, "parent_digest"), "none");
    assert_eq!(proof_get(&p, "names"), "currency_code id nested total_minor");
    assert_eq!(proof_get(&p, "contract"), "egress:OrderResponse");
    assert_eq!(proof_get(&p, "anchor"), "GET /v1/orders");
    let dg = proof_get(&p, "digest");
    assert_eq!(dg.len(), 64, "digest is not 64 hex: {dg}");
    assert!(dg.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_eq!(proof_get(&p, "commit").len(), 40);
    assert_eq!(proof_get(&p, "source_blob").len(), 40);
    assert!(out.contains("invar.proof") && out.contains(&dg), "emit must print path, epoch and digest: {out}");

    let cons = consumer("f1c", &p, "id total_minor currency_code", Some(("1", &dg)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("[HOLDS]") && out.contains("egress:OrderResponse"), "{out}");
}

#[test]
fn prf_f2_added_field_chains_to_epoch_2_and_still_holds() {
    let d = producer("f2");
    let (_, dg1) = emitted(&d);
    put(&d, "lib/schemas.ts", &SCHEMAS.replace("  nested:", "  status: z.string(),\n  nested:"));
    commit_all(&d, "add status");
    relock(&d);
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 0, "{out}");
    let p = read_proof(&d, "invar.proof");
    assert_eq!(proof_get(&p, "epoch"), "2");
    assert_eq!(proof_get(&p, "parent_digest"), dg1);
    let dg2 = proof_get(&p, "digest");
    assert_ne!(dg1, dg2);

    // The consumer still accepts epoch 1: a chained higher epoch passes, and
    // the added field is noted, never alarmed.
    let cons = consumer("f2c", &p, "id total_minor currency_code", Some(("1", &dg1)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("[HOLDS]"), "{out}");
    assert!(out.contains("status"), "the added field is noted: {out}");
    // After recording epoch 2, exit 0 again.
    let cons2 = consumer("f2c2", &p, "id total_minor currency_code", Some(("2", &dg2)), "");
    let (c, out) = check_proof(&cons2);
    assert_eq!(c, 0, "{out}");
}

const SCHEMAS2: &str = "import { z } from \"zod\";\nexport const InvoiceResponse = z.object({\n  invoice_number: z.string(),\n  due_date: z.string(),\n  tax_total: z.number(),\n});\n";

#[test]
fn prf_f3_two_contracts_in_one_proof_directory_resolve_independently() {
    let toml = format!(
        "{PRODUCER_TOML}\n[[invariant]]\nkind = \"egress\"\nid = \"egress:InvoiceResponse\"\nproducer = \"lib/invoice.ts\"\nshape = \"InvoiceResponse\"\nboundary = \"app/api/v1/invoices/route.ts\"\neligibility = \"versioned-path\"\nanchor = \"GET /v1/invoices\"\n"
    );
    let d = ws("f3", &[("lib/schemas.ts", SCHEMAS), ("lib/invoice.ts", SCHEMAS2), ("app/api/v1/orders/route.ts", ROUTE), ("invar.toml", &toml)]);
    ginit(&d);
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = emit(&d, &["--out", ".invar/proofs/orders.proof"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = invar(&d, &["lock", "--emit-proof", "egress:InvoiceResponse", "--out", ".invar/proofs/invoices.proof"]);
    assert_eq!(c, 0, "{out}");
    let pa = read_proof(&d, ".invar/proofs/orders.proof");
    let pb = read_proof(&d, ".invar/proofs/invoices.proof");

    let b1 = import_block("vendor/proofs", "egress:OrderResponse", "GET /v1/orders", "id total_minor", Some(("1", &proof_get(&pa, "digest"))), "");
    let b2 = import_block("vendor/proofs", "egress:InvoiceResponse", "GET /v1/invoices", "invoice_number tax_total", Some(("1", &proof_get(&pb, "digest"))), "");
    let cons = ws("f3c", &[("vendor/proofs/a.proof", &pa), ("vendor/proofs/b.proof", &pb), ("invar.toml", &format!("{b1}\n{b2}"))]);
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 0, "{out}");
    assert_eq!(out.matches("[HOLDS]").count(), 2, "{out}");
}

#[test]
fn prf_f4_layout_rule_root_or_dot_invar_proofs_only() {
    let d = producer("f4");
    let (c, out) = emit(&d, &["--out", ".invar/proofs/orders.proof"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = emit(&d, &["--out", "invar.proof"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = emit(&d, &["--out", "somewhere/else.proof"]);
    assert_eq!(c, 3, "an out path outside the D6 layout is exit 3: {out}");
    assert!(!d.join("somewhere/else.proof").exists(), "no file is written on refusal");
    let (c, _) = emit(&d, &["--out", ".invar/proofs/bad name.proof"]);
    assert_eq!(c, 3, "a proof name outside [A-Za-z0-9_-]+ is exit 3");
}

// ------------------------------------------------------------ V: violations

#[test]
fn prf_v1_removed_relied_on_field_is_violated() {
    let d = producer("v1");
    let (_, dg1) = emitted(&d);
    put(&d, "lib/schemas.ts", &SCHEMAS.replace("  currency_code: z.string(),\n", ""));
    commit_all(&d, "drop currency_code");
    relock(&d);
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 0, "{out}");
    let p = read_proof(&d, "invar.proof");
    let cons = consumer("v1c", &p, "id total_minor currency_code", Some(("1", &dg1)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 1, "{out}");
    assert!(out.contains("[VIOLATED]") && out.contains("currency_code"), "{out}");
    assert!(out.contains("epoch 2") || out.contains("epoch \"2\"") || out.contains("epoch=2"), "the violation names the epoch: {out}");
    assert!(out.contains("orders.proof"), "the violation names the proof path: {out}");
}

#[test]
fn prf_v2_renamed_field_is_violated_under_its_old_name() {
    let d = producer("v2");
    let (_, dg1) = emitted(&d);
    put(&d, "lib/schemas.ts", &SCHEMAS.replace("total_minor", "total_cents"));
    commit_all(&d, "rename");
    relock(&d);
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 0, "{out}");
    let p = read_proof(&d, "invar.proof");
    let cons = consumer("v2c", &p, "id total_minor", Some(("1", &dg1)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 1, "{out}");
    assert!(out.contains("total_minor"), "{out}");
}

#[test]
fn prf_v3_field_the_proof_never_listed_is_violated() {
    let d = producer("v3");
    let (_, dg1) = emitted(&d);
    let p = read_proof(&d, "invar.proof");
    let cons = consumer("v3c", &p, "id owner", Some(("1", &dg1)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 1, "{out}");
    assert!(out.contains("owner") && !out.contains("[HOLDS]"), "{out}");
}

// ------------------------------------- E: authoring refusals (exit 3, no file)

#[test]
fn prf_e1_no_lock_entry_refuses() {
    let d = ws("e1", &[("lib/schemas.ts", SCHEMAS), ("invar.toml", PRODUCER_TOML)]);
    ginit(&d);
    // No `invar lock` was run: there is no invar.lock entry.
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 3, "{out}");
    assert!(!d.join("invar.proof").exists(), "no file on refusal");
    // An unknown contract id refuses too.
    let d2 = producer("e1b");
    let (c, out) = invar(&d2, &["lock", "--emit-proof", "egress:Nope"]);
    assert_eq!(c, 3, "{out}");
}

#[test]
fn prf_e2_dynamic_shape_refuses() {
    let d = producer("e2");
    put(&d, "lib/schemas.ts", "import { z } from \"zod\";\nconst extra = { memo: z.string() };\nexport const OrderResponse = z.object({\n  id: z.string(),\n  ...extra,\n});\n");
    commit_all(&d, "dynamic");
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 3, "{out}");
    assert!(!d.join("invar.proof").exists());
}

#[test]
fn prf_e3_unlocked_drift_refuses_and_points_at_lock_update() {
    let d = producer("e3");
    put(&d, "lib/schemas.ts", &SCHEMAS.replace("  nested:", "  status: z.string(),\n  nested:"));
    commit_all(&d, "drift");
    // The lock still holds the old key set: emit must refuse, not re-lock.
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 3, "{out}");
    assert!(out.contains("lock --update"), "the refusal names the sanctioned next action: {out}");
    assert!(!d.join("invar.proof").exists());
}

#[test]
fn prf_e4_uncommitted_shape_edit_refuses() {
    let d = producer("e4");
    let mut s = String::from(SCHEMAS);
    s.push_str("// trailing note\n");
    put(&d, "lib/schemas.ts", &s);
    // Not committed: the working tree no longer equals the committed blob.
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 3, "{out}");
    assert!(out.contains("uncommitted") || out.contains("committed"), "{out}");
    assert!(!d.join("invar.proof").exists());
}

#[test]
fn prf_e5_undeclared_anchor_refuses() {
    let d = ws("e5", &[("lib/schemas.ts", SCHEMAS), ("app/api/v1/orders/route.ts", ROUTE), ("invar.toml", &PRODUCER_TOML.replace("anchor = \"GET /v1/orders\"\n", ""))]);
    ginit(&d);
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 3, "{out}");
    assert!(out.contains("anchor"), "{out}");
    assert!(!d.join("invar.proof").exists());
}

#[test]
fn prf_e6_emit_refuses_to_overwrite_a_proof_that_fails_verification() {
    // Added at Gate 3 so mutation 13 dies by a named assertion; the behavior
    // itself is R-P-4 ("an existing proof that fails verification is exit 3").
    let d = producer("e6");
    let (_, _dg) = emitted(&d);
    let corrupted = read_proof(&d, "invar.proof").replace("epoch = \"1\"", "epoch = \"7\"");
    put(&d, "invar.proof", &corrupted);
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 3, "{out}");
    assert!(out.contains("refus"), "the refusal is stated: {out}");
    assert_eq!(read_proof(&d, "invar.proof"), corrupted, "the failing proof is never overwritten");
}

// --------------------------------------- U: abstentions and errors (exact)

#[test]
fn prf_u1_anchor_mismatch_is_scope_mismatch() {
    let d = producer("u1");
    let (_, dg) = emitted(&d);
    let p = read_proof(&d, "invar.proof");
    let toml = import_block("vendor/orders.proof", "egress:OrderResponse", "GET /v2/orders", "id", Some(("1", &dg)), "");
    let cons = ws("u1c", &[("vendor/orders.proof", &p), ("invar.toml", &toml)]);
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("ABSTAIN PROOF-SCOPE-MISMATCH invar.toml:"), "{out}");
}

#[test]
fn prf_u2_two_proofs_for_one_contract_in_a_directory_is_ambiguous() {
    let d = producer("u2");
    let (_, dg) = emitted(&d);
    let p = read_proof(&d, "invar.proof");
    let toml = import_block("vendor/proofs", "egress:OrderResponse", "GET /v1/orders", "id", Some(("1", &dg)), "");
    let cons = ws("u2c", &[("vendor/proofs/a.proof", &p), ("vendor/proofs/b.proof", &p), ("invar.toml", &toml)]);
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("ABSTAIN PROOF-AMBIG-SCHEMA invar.toml:"), "{out}");
}

#[test]
fn prf_u3_epoch_below_accepted_is_stale() {
    let d = producer("u3");
    let (_, dg1) = emitted(&d);
    let p1 = read_proof(&d, "invar.proof");
    put(&d, "lib/schemas.ts", &SCHEMAS.replace("  nested:", "  status: z.string(),\n  nested:"));
    commit_all(&d, "v2");
    relock(&d);
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 0, "{out}");
    let dg2 = proof_get(&read_proof(&d, "invar.proof"), "digest");
    // The consumer accepted epoch 2, then an old epoch-1 proof is replayed.
    let cons = consumer("u3c", &p1, "id", Some(("2", &dg2)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("ABSTAIN STALE-PROOF-COMMIT invar.toml:"), "{out}");
    let _ = dg1;
}

#[test]
fn prf_u4_equal_epoch_with_different_digest_is_a_fork() {
    let d = producer("u4a");
    let (_, dg_a) = emitted(&d);
    // An independent history at the same epoch: same keys, different bytes.
    let mut s2 = String::from("// fork lineage\n");
    s2.push_str(SCHEMAS);
    let d2 = ws("u4b", &[("lib/schemas.ts", &s2), ("app/api/v1/orders/route.ts", ROUTE), ("invar.toml", PRODUCER_TOML)]);
    ginit(&d2);
    let (c, out) = invar(&d2, &["lock"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = emit(&d2, &[]);
    assert_eq!(c, 0, "{out}");
    let p_b = read_proof(&d2, "invar.proof");
    assert_ne!(proof_get(&p_b, "digest"), dg_a, "the two lineages must differ");
    let cons = consumer("u4c", &p_b, "id", Some(("1", &dg_a)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 3, "{out}");
    assert!(out.contains("PROOF-FORK"), "{out}");
}

#[test]
fn prf_u5_no_accepted_state_is_unanchored_never_holds() {
    let d = producer("u5");
    let (_, _dg) = emitted(&d);
    let p = read_proof(&d, "invar.proof");
    let cons = consumer("u5c", &p, "id", None, "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("ABSTAIN TEMPORAL-STALENESS-UNANCHORED invar.toml:"), "{out}");
    assert!(!out.contains("[HOLDS]"), "HOLDS is not offered unanchored: {out}");
}

#[test]
fn prf_u6_opt_in_age_window_exceeded_is_clock_dependent_abstention() {
    let d = producer("u6");
    let (_, dg) = emitted(&d);
    // generated_at is informational and outside the digest: aging the proof
    // by hand does not tamper it.
    let p = read_proof(&d, "invar.proof");
    let old = regen_at(&p, "2020-01-01T00:00:00Z");
    let cons = consumer("u6c", &old, "id", Some(("1", &dg)), "max_age_days = \"30\"\n");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("ABSTAIN TEMPORAL-STALENESS-UNANCHORED invar.toml:"), "{out}");
    assert!(out.contains("clock"), "the result says it is clock-dependent: {out}");
}

/// Replace the generated_at value (outside the digest by R-P-1).
fn regen_at(p: &str, when: &str) -> String {
    let cur = proof_get(p, "generated_at");
    p.replace(&format!("generated_at = \"{cur}\""), &format!("generated_at = \"{when}\""))
}

#[test]
fn prf_u7_non_egress_kind_is_unsupported() {
    let d = producer("u7");
    let (_, dg) = emitted(&d);
    let p = read_proof(&d, "invar.proof");
    let toml = import_block("vendor/orders.proof", "strings:OrderIn@const:SELECT_FIELDS", "GET /v1/orders", "id", Some(("1", &dg)), "");
    let cons = ws("u7c", &[("vendor/orders.proof", &p), ("invar.toml", &toml)]);
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("ABSTAIN PROOF-UNSUPPORTED-KIND invar.toml:"), "{out}");
}

#[test]
fn prf_u8_edited_names_with_stale_digest_is_tampered() {
    let d = producer("u8");
    let (_, dg) = emitted(&d);
    let p = read_proof(&d, "invar.proof").replace(
        "names = \"currency_code id nested total_minor\"",
        "names = \"currency_code id nested owner total_minor\"",
    );
    let cons = consumer("u8c", &p, "id owner", Some(("1", &dg)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 3, "{out}");
    assert!(out.contains("PROOF-TAMPERED"), "{out}");
    assert!(!out.contains("[VIOLATED]"), "tamper is an error, never a verdict: {out}");
}

#[test]
fn prf_u9_unchained_parent_at_accepted_plus_one_is_a_fork() {
    // Lineage A, accepted at epoch 1.
    let d = producer("u9a");
    let (_, dg_a) = emitted(&d);
    // Lineage B advances to epoch 2 with a parent that is not A.
    let mut s2 = String::from("// lineage b\n");
    s2.push_str(SCHEMAS);
    let d2 = ws("u9b", &[("lib/schemas.ts", &s2), ("app/api/v1/orders/route.ts", ROUTE), ("invar.toml", PRODUCER_TOML)]);
    ginit(&d2);
    assert_eq!(invar(&d2, &["lock"]).0, 0);
    assert_eq!(emit(&d2, &[]).0, 0);
    put(&d2, "lib/schemas.ts", &s2.replace("  nested:", "  status: z.string(),\n  nested:"));
    commit_all(&d2, "b2");
    relock(&d2);
    assert_eq!(emit(&d2, &[]).0, 0);
    let p_c = read_proof(&d2, "invar.proof");
    assert_eq!(proof_get(&p_c, "epoch"), "2");
    let cons = consumer("u9c", &p_c, "id", Some(("1", &dg_a)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 3, "{out}");
    assert!(out.contains("PROOF-FORK"), "{out}");
}

#[test]
fn prf_u10_missing_proof_path_is_not_found() {
    let toml = import_block("vendor/missing.proof", "egress:OrderResponse", "GET /v1/orders", "id", Some(("1", "0".repeat(64).as_str())), "");
    let cons = ws("u10", &[("invar.toml", &toml)]);
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("ABSTAIN PROOF-NOT-FOUND invar.toml:1 "), "the location is the table's line: {out}");
    // A directory holding no proof for the contract is NOT-FOUND too.
    let d = producer("u10b");
    let (_, dg) = emitted(&d);
    let p = read_proof(&d, "invar.proof");
    let toml2 = import_block("vendor/proofs", "egress:SomethingElse", "GET /v1/x", "id", Some(("1", &dg)), "");
    let cons2 = ws("u10c", &[("vendor/proofs/a.proof", &p), ("invar.toml", &toml2)]);
    let (c, out) = check_proof(&cons2);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("PROOF-NOT-FOUND"), "{out}");
}

#[test]
fn prf_u11_unknown_binding_key_is_exit_3_naming_the_key() {
    let d = producer("u11");
    let (_, dg) = emitted(&d);
    let p = read_proof(&d, "invar.proof");
    let cons = consumer("u11c", &p, "id", Some(("1", &dg)), "strict = \"true\"\n");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 3, "a typo in a trust binding is never silently ignored: {out}");
    assert!(out.contains("strict"), "{out}");
}

// --------------------------------------------- N: negative controls (PASS)

#[test]
fn prf_n1_a_proof_file_changes_nothing_for_plain_check() {
    let d = producer("n1");
    put(&d, "invar.proof", "[proof]\nformat = \"1\"\n");
    let (c1, o1) = invar(&d, &["check"]);
    std::fs::remove_file(d.join("invar.proof")).unwrap();
    let (c2, o2) = invar(&d, &["check"]);
    assert_eq!((c1, o1), (c2, o2), "plain check must ignore proof artifacts");
}

#[test]
fn prf_n2_init_proposes_nothing_from_a_proof_file() {
    let files: &[(&str, &str)] = &[
        ("lib/order.ts", "import { z } from \"zod\";\nexport const OrderIn = z.object({\n  total_minor: z.number(),\n  currency_code: z.string(),\n  unread_flag: z.boolean(),\n});\n"),
        ("src/consume.js", "import { OrderIn } from \"../lib/order.js\";\nexport function f(row) {\n  return [row.total_minor, row.currency_code];\n}\n"),
    ];
    let run_init = |tag: &str, with_proof: bool| -> String {
        let d = ws(tag, files);
        if with_proof {
            put(&d, "invar.proof", "[proof]\nformat = \"1\"\n\n[fields]\nnames = \"a b c\"\n");
        }
        let o = Command::new(bin()).args(["init", "--stdout", "--root"]).arg(&d).output().unwrap();
        assert_eq!(o.status.code(), Some(0));
        String::from_utf8(o.stdout).unwrap()
    };
    // The walk honestly counts the extra file; everything else, above all the
    // proposal set, must be byte-identical, and the proof file never appears.
    let norm = |s: String| {
        s.lines().filter(|l| !l.starts_with("files_scanned")).collect::<Vec<_>>().join("\n")
    };
    let without = run_init("n2a", false);
    let with = run_init("n2b", true);
    assert!(!with.contains("invar.proof"), "init must never mention the proof file: {with}");
    assert_eq!(norm(without), norm(with), "init must propose nothing from invar.proof");
}

#[test]
fn prf_n3_proof_import_tables_change_nothing_for_plain_check() {
    let d = producer("n3");
    let (c1, o1) = invar(&d, &["check"]);
    let mut toml = std::fs::read_to_string(d.join("invar.toml")).unwrap();
    toml.push('\n');
    toml.push_str(&import_block("vendor/orders.proof", "egress:OrderResponse", "GET /v1/orders", "id", Some(("1", "a")), ""));
    put(&d, "invar.toml", &toml);
    let (c2, o2) = invar(&d, &["check"]);
    assert_eq!((c1, o1), (c2, o2), "plain check must not evaluate [[proof.import]]");
}

// ------------------------------------------------------------- M: shared

#[test]
fn prf_m1_reemit_at_unchanged_state_is_byte_identical() {
    let d = producer("m1");
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 0, "{out}");
    let first = read_proof(&d, "invar.proof");
    let (c, out) = emit(&d, &[]);
    assert_eq!(c, 0, "{out}");
    let second = read_proof(&d, "invar.proof");
    assert_eq!(first, second, "re-emit at the same source state must be byte-identical (generated_at preserved, epoch unchanged)");
    assert_eq!(proof_get(&second, "epoch"), "1");
}

#[test]
fn prf_m2_default_check_reads_no_clock() {
    let d = producer("m2");
    let (_, dg) = emitted(&d);
    // A proof aged by hand, with NO max_age_days: default verification must
    // not look at generated_at at all.
    let p = regen_at(&read_proof(&d, "invar.proof"), "2020-01-01T00:00:00Z");
    let cons = consumer("m2c", &p, "id", Some(("1", &dg)), "");
    let (c1, o1) = invar_env(&cons, &["check", "--proof"], &[("TZ", "UTC")]);
    let (c2, o2) = invar_env(&cons, &["check", "--proof"], &[("TZ", "Asia/Ho_Chi_Minh")]);
    assert_eq!(c1, 0, "{o1}");
    assert_eq!((c1, &o1), (c2, &o2), "default check output varies with the clock environment");
    assert!(!o1.to_lowercase().contains("stale") && !o1.to_lowercase().contains("age"), "no age text by default: {o1}");
}

#[test]
fn prf_m3_no_new_dependency() {
    let toml = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let deps: Vec<&str> = toml
        .lines()
        .skip_while(|l| l.trim() != "[dependencies]")
        .skip(1)
        .take_while(|l| !l.trim().starts_with('['))
        .filter_map(|l| l.split('=').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(deps, ["clap", "serde", "serde_json", "blake3", "rusqlite", "thiserror", "anyhow"], "the dependency set is frozen");
}

#[test]
fn prf_m4_proof_module_has_no_network_capability() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/proof.rs"))
        .expect("src/proof.rs must exist (Class 4)");
    for needle in ["std::net", "TcpStream", "UdpSocket", "reqwest", "curl", "CurlTransport", "http://", "https://"] {
        assert!(!src.contains(needle), "proof.rs must not reference {needle}");
    }
}

#[test]
fn prf_m5_rule_fingerprint_is_stable_across_contracts_and_runs() {
    let d = producer("m5");
    let (_, _dg) = emitted(&d);
    let f1 = proof_get(&read_proof(&d, "invar.proof"), "rule_fingerprint");
    assert_eq!(f1.len(), 64, "{f1}");
    let d2 = producer("m5b");
    let (_, _dg2) = emitted(&d2);
    let f2 = proof_get(&read_proof(&d2, "invar.proof"), "rule_fingerprint");
    assert_eq!(f1, f2, "one engine has one rule fingerprint");
}

#[test]
fn prf_m6_foreign_fingerprint_is_a_note_never_a_verdict_change() {
    // A proof sealed by a different engine build carries a different
    // rule_fingerprint. One test run has one engine, so the foreign proof is
    // constructed by editing the fingerprint and resealing with the
    // library's own canonical digest (the fingerprint is inside the digest,
    // R-P-10; Cargo.toml is Class 1, so no dev-dependency on blake3).
    let d = producer("m6");
    let (_, _dg) = emitted(&d);
    let p = read_proof(&d, "invar.proof");
    let fp = proof_get(&p, "rule_fingerprint");
    let foreign = p.replace(&fp, &"f".repeat(64));
    let idx = foreign.find("\n[attestation]\n").unwrap();
    let canonical = &foreign[..idx];
    let resealed_digest = invar_falsify::proof::canonical_digest(canonical);
    let old_digest = proof_get(&foreign, "digest");
    let resealed = foreign.replace(&format!("digest = \"{old_digest}\""), &format!("digest = \"{resealed_digest}\""));
    let cons = consumer("m6c", &resealed, "id", Some(("1", &resealed_digest)), "");
    let (c, out) = check_proof(&cons);
    assert_eq!(c, 0, "a foreign fingerprint never changes the verdict: {out}");
    assert!(out.contains("[HOLDS]"), "{out}");
    assert!(out.contains("NOTE") && out.contains("rule_fingerprint"), "the mismatch is stated: {out}");
}

/// CHECK-INCR-1 (K-2): these tests exercise evaluation, so `check` runs with
/// `--no-reuse`; reuse is tested in check_incr1_test.rs.
fn no_reuse<'a>(args: &[&'a str]) -> Vec<&'a str> {
    let mut v = args.to_vec();
    if v.first() == Some(&"check") && !v.contains(&"--no-reuse") {
        v.push("--no-reuse");
    }
    v
}
