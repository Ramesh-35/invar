//! INVAR-PROOF-001 r3: cross-repository egress attestation (`invar.proof`).
//!
//! Provider side: `invar lock --emit-proof <contract-id> [--out <path>]`
//! (exit 0 or 3 only; R-P-3 preconditions checked in order, no file written
//! on refusal). Consumer side: `invar check --proof` evaluates every
//! `[[proof.import]]` table in invar.toml (exit 0, 1, 3 or 4; never 2).
//!
//! HONEST BOUNDS (spec section 0): a proof attests what the provider's
//! SOURCE declared at a commit, never what its runtime sends (B1/B3); the
//! level-0 digest proves integrity, not identity (B2); bindings are explicit
//! and nothing here infers a pairing (B4). Zero network: emit invokes only
//! the local `git` subprocess; verification reads only the proof file and
//! the manifest (R-P-9).

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::check::Sections;
use crate::lock;

/// R-P-10: the engine's rule table, serialized as sorted `key=value` lines
/// and digested with blake3 (ruling I2: blake3 everywhere, no new crate).
/// Informational and traceability only: a mismatch at verification prints a
/// NOTE and never changes a verdict (fixture PRF-M6, mutation 15).
pub fn rule_fingerprint() -> String {
    let lines = [
        format!("engine_version={}", crate::report::ENGINE_VERSION),
        format!("min_fields={}", crate::codepairs::MIN_FIELDS),
        "proof_format=1".to_string(),
        "proof_level=0".to_string(),
        format!("response_markers={}", crate::valuepairs::RESPONSE_MARKERS.join(" ")),
        format!("stoplist={}", crate::codepairs::STOPLIST.join(" ")),
    ];
    // The keys above are already in sorted order; keep them so.
    canonical_digest(&(lines.join("\n") + "\n"))
}

/// The blake3 hex digest of a canonical byte string (R-P-1).
pub fn canonical_digest(canonical: &str) -> String {
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

// ------------------------------------------------------------ proof file

/// A parsed, digest-verified proof file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub fingerprint: String,
    pub repo: String,
    pub commit: String,
    pub source_blob: String,
    pub boundary: String,
    pub shape_file: String,
    pub shape: String,
    pub contract: String,
    pub anchor: String,
    pub names: Vec<String>,
    pub epoch: u64,
    pub parent_digest: String,
    pub digest: String,
    pub generated_at: String,
}

impl Proof {
    /// The identity of what is attested, ignoring the chain position. Two
    /// emits with an equal identity are the same attestation (P-INT-2).
    fn identity(&self) -> (String, String, String, String, String, String, String, String, String, String) {
        (
            self.fingerprint.clone(),
            self.repo.clone(),
            self.commit.clone(),
            self.source_blob.clone(),
            self.boundary.clone(),
            self.shape_file.clone(),
            self.shape.clone(),
            self.contract.clone(),
            self.anchor.clone(),
            self.names.join(" "),
        )
    }
}

/// R-P-1 canonical form: the exact bytes of the proof, producer, fields and
/// chain sections, fixed key order, LF, sorted names.
fn render_canonical(p: &Proof) -> String {
    format!(
        "[proof]\nformat = \"1\"\nlevel = \"0\"\nrule_fingerprint = \"{}\"\n\n[producer]\nrepo = \"{}\"\ncommit = \"{}\"\nsource_blob = \"{}\"\nboundary = \"{}\"\nshape_file = \"{}\"\nshape = \"{}\"\ncontract = \"{}\"\nanchor = \"{}\"\n\n[fields]\nnames = \"{}\"\n\n[chain]\nepoch = \"{}\"\nparent_digest = \"{}\"\n",
        p.fingerprint,
        p.repo,
        p.commit,
        p.source_blob,
        p.boundary,
        p.shape_file,
        p.shape,
        p.contract,
        p.anchor,
        p.names.join(" "),
        p.epoch,
        p.parent_digest,
    )
}

fn render_file(p: &Proof) -> String {
    format!(
        "{}\n[attestation]\ndigest = \"{}\"\ngenerated_at = \"{}\"\n",
        render_canonical(p),
        p.digest,
        p.generated_at
    )
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Parse and digest-verify a proof file. Err is a PROOF-TAMPERED reason: a
/// malformed file, unknown format or level, or a digest mismatch (exit 3 at
/// verification; a stated error, never a verdict).
pub fn parse_proof(name: &str, text: &str) -> std::result::Result<Proof, String> {
    let Some(split) = text.find("\n[attestation]\n") else {
        return Err("malformed proof: no [attestation] section".to_string());
    };
    let canonical = &text[..split];
    let sections = crate::config::parse_sections(name, text).map_err(|e| format!("malformed proof: {e}"))?;
    let get = |sec: &str, key: &str| -> std::result::Result<String, String> {
        sections
            .iter()
            .find(|(n, _)| n == sec)
            .and_then(|(_, kv)| kv.iter().find(|(k, _)| k == key))
            .map(|(_, v)| v.clone())
            .ok_or_else(|| format!("malformed proof: missing {sec}.{key}"))
    };
    if get("proof", "format")? != "1" || get("proof", "level")? != "0" {
        return Err("unknown proof format or level (this engine verifies format 1, level 0)".to_string());
    }
    let digest = get("attestation", "digest")?;
    if !is_hex64(&digest) {
        return Err("malformed proof: digest is not 64 hex characters".to_string());
    }
    let recomputed = canonical_digest(canonical);
    if recomputed != digest {
        return Err("digest mismatch: the file was edited after emission".to_string());
    }
    let epoch: u64 = get("chain", "epoch")?
        .parse()
        .ok()
        .filter(|e| *e >= 1)
        .ok_or_else(|| "malformed proof: epoch is not a positive integer".to_string())?;
    let mut names: Vec<String> = get("fields", "names")?.split_whitespace().map(str::to_string).collect();
    names.sort();
    Ok(Proof {
        fingerprint: get("proof", "rule_fingerprint")?,
        repo: get("producer", "repo")?,
        commit: get("producer", "commit")?,
        source_blob: get("producer", "source_blob")?,
        boundary: get("producer", "boundary")?,
        shape_file: get("producer", "shape_file")?,
        shape: get("producer", "shape")?,
        contract: get("producer", "contract")?,
        anchor: get("producer", "anchor")?,
        names,
        epoch,
        parent_digest: get("chain", "parent_digest")?,
        digest,
        generated_at: get("attestation", "generated_at").unwrap_or_default(),
    })
}

// ------------------------------------------------------------- provider

fn getk(b: &[(String, String)], key: &str) -> Option<String> {
    b.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

fn git_out(dir: &Path, args: &[&str]) -> Result<String> {
    let o = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .context("git is not available; a proof binds a committed source state (R-P-3 e)")?;
    if !o.status.success() {
        bail!("git {}: {}", args.join(" "), String::from_utf8_lossy(&o.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// R-P-2a (ruling D6): `invar.proof` at the root, or `.invar/proofs/<name>.proof`.
fn valid_out(out: &str) -> bool {
    if out == "invar.proof" {
        return true;
    }
    out.strip_prefix(".invar/proofs/")
        .and_then(|rest| rest.strip_suffix(".proof"))
        .map(|name| !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'))
        .unwrap_or(false)
}

/// `invar lock --emit-proof <contract-id>`: R-P-3 preconditions in order,
/// any failure exit 3 (via the caller's error path) with no file written.
pub fn emit(base: &Path, sections: &Sections, contract_id: &str, out: Option<&str>) -> Result<String> {
    // (a) the contract exists, is kind egress and has an invar.lock entry.
    let block = sections
        .iter()
        .filter(|(n, _)| n == "invariant")
        .map(|(_, kv)| kv)
        .find(|kv| getk(kv, "id").as_deref() == Some(contract_id))
        .with_context(|| format!("no [[invariant]] block with id {contract_id}"))?;
    let kind = getk(block, "kind").unwrap_or_default();
    if kind != "egress" {
        bail!("{contract_id} is kind {kind}; --emit-proof attests egress contracts only in v1 (ruling D5)");
    }
    let lock_path = base.join("invar.lock");
    if !lock_path.exists() {
        bail!("no invar.lock; run `invar lock` first, then emit (a proof attests a locked contract)");
    }
    let lockf = lock::parse(&lock_path.display().to_string(), &std::fs::read_to_string(&lock_path)?)?;
    let entry = lockf
        .contracts
        .iter()
        .find(|c| c.id == contract_id)
        .with_context(|| format!("no invar.lock entry for {contract_id}; run `invar lock` first"))?;

    // (b) the shape is statically extractable right now.
    let producer = getk(block, "producer").with_context(|| format!("{contract_id}: the block has no producer"))?;
    if producer.contains("://") || producer.starts_with("github:") {
        bail!("{contract_id}: --emit-proof attests local producer files only (zero network, R-P-9)");
    }
    let shape = getk(block, "shape").with_context(|| format!("{contract_id}: the block has no shape"))?;
    let text = std::fs::read_to_string(base.join(&producer)).with_context(|| format!("reading {producer}"))?;
    let mut current = match crate::valuepairs::read_shape(&text, &shape) {
        crate::valuepairs::ShapeRead::Found(f) => f,
        crate::valuepairs::ShapeRead::Dynamic(why) => {
            bail!("{contract_id}: {shape} in {producer} is not statically extractable ({why}); nothing is attested on a guess")
        }
        crate::valuepairs::ShapeRead::NotFound => {
            bail!("{contract_id}: no `export const {shape} = z.object(...)` in {producer}")
        }
    };
    current.sort();

    // (c) the current direct keys equal the locked set (no unlocked drift).
    let locked: Vec<String> = entry
        .members
        .iter()
        .find(|(_, scope, _)| scope == &shape)
        .map(|(_, _, f)| f.clone())
        .unwrap_or_else(|| entry.bindings.iter().map(|b| b.field.clone()).collect());
    if locked != current {
        bail!(
            "{contract_id}: the current {shape} keys differ from the locked set (locked: {}; current: {}); review the change and run `invar lock --update`, then emit again",
            locked.join(" "),
            current.join(" ")
        );
    }

    // (d) the block declares the receiver anchor (ruling D3).
    let anchor = getk(block, "anchor").with_context(|| {
        format!("{contract_id}: the [[invariant]] block declares no anchor; add anchor = \"<METHOD /path>\" (required, ruling D3)")
    })?;

    // (e) git is available and the working tree equals the committed blob.
    let commit = git_out(base, &["rev-parse", "HEAD"])?;
    let blob = git_out(base, &["rev-parse", &format!("HEAD:{producer}")])
        .with_context(|| format!("{producer} is not committed; the proof binds the committed blob (R-P-2)"))?;
    // SECURITY-SPEC S3-2: no filter program named by the repository runs;
    // the proof binds the exact committed bytes.
    let worktree = git_out(base, &["hash-object", "--no-filters", &producer])?;
    if worktree != blob {
        bail!("{producer} differs from its committed blob (uncommitted edits, or a git filter such as line-ending conversion applies to it); commit the shape file before emitting (the proof binds the exact committed bytes, not the working tree)");
    }

    // Layout (R-P-2a), then chain (R-P-4).
    let out = out.unwrap_or("invar.proof");
    if !valid_out(out) {
        bail!("--out must be invar.proof or .invar/proofs/<name>.proof with <name> in [A-Za-z0-9_-]+ (ruling D6)");
    }
    let mut proof = Proof {
        fingerprint: rule_fingerprint(),
        repo: getk(block, "repo").unwrap_or_else(|| "none".to_string()),
        commit,
        source_blob: blob,
        boundary: getk(block, "boundary").unwrap_or_else(|| "none".to_string()),
        shape_file: producer.clone(),
        shape,
        contract: contract_id.to_string(),
        anchor,
        names: current,
        epoch: 1,
        parent_digest: "none".to_string(),
        digest: String::new(),
        generated_at: lock::now_rfc3339(),
    };
    let out_path = base.join(out);
    if out_path.exists() {
        let prev_text = std::fs::read_to_string(&out_path)?;
        let prev = parse_proof(out, &prev_text)
            .map_err(|e| anyhow::anyhow!("the existing proof at {out} fails verification ({e}); refusing to overwrite it"))?;
        if prev.identity() == proof.identity() {
            // R-P-1 / P-INT-2: an emit at an unchanged source state is a
            // no-op; the file stays byte-identical, generated_at preserved.
            return Ok(format!("unchanged: {out} epoch {} digest {}", prev.epoch, prev.digest));
        }
        proof.epoch = prev.epoch + 1;
        proof.parent_digest = prev.digest;
    }
    proof.digest = canonical_digest(&render_canonical(&proof));
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out_path, render_file(&proof))?;
    Ok(format!("wrote {out} epoch {} digest {}", proof.epoch, proof.digest))
}

// ------------------------------------------------------------- consumer

const ALLOWED_KEYS: &[&str] = &["proof", "contract_id", "anchor", "fields", "accepted_epoch", "accepted_digest", "max_age_days"];

/// Exit precedence: 3 over 2 over 1 over 4 over 0 (R-P-6; exit 2 never
/// arises from proofs and keeps its ledger meaning on the plain side).
pub fn worse(a: i32, b: i32) -> i32 {
    let rank = |c: i32| match c {
        3 => 5,
        2 => 4,
        1 => 3,
        4 => 2,
        _ => 1,
    };
    if rank(b) > rank(a) {
        b
    } else {
        a
    }
}

/// Evaluate every `[[proof.import]]` table (R-P-6). Returns the lines to
/// print and the worst exit code over all imports. Reads no clock unless a
/// table opts into `max_age_days` (R-P-8); `today` is passed in by the
/// caller so this function itself stays hermetic.
pub fn verify(base: &Path, raw_toml: &str, sections: &Sections, today: i64) -> (Vec<String>, i32) {
    let header_lines: Vec<usize> = raw_toml
        .lines()
        .enumerate()
        .filter(|(_, l)| l.trim() == "[[proof.import]]")
        .map(|(i, _)| i + 1)
        .collect();
    let imports: Vec<&Vec<(String, String)>> =
        sections.iter().filter(|(n, _)| n == "proof.import").map(|(_, kv)| kv).collect();
    let engine_fp = rule_fingerprint();
    let mut lines = Vec::new();
    let mut code = 0;
    for (i, kv) in imports.iter().enumerate() {
        let loc = format!("invar.toml:{}", header_lines.get(i).copied().unwrap_or(0));
        let (ls, c) = verify_one(base, kv, &loc, today, &engine_fp);
        lines.extend(ls);
        code = worse(code, c);
    }
    lines.push(format!("proof imports evaluated: {}", imports.len()));
    (lines, code)
}

fn verify_one(base: &Path, kv: &[(String, String)], loc: &str, today: i64, engine_fp: &str) -> (Vec<String>, i32) {
    let abstain = |code: &str, reason: String| format!("ABSTAIN {code} {loc} {reason}");
    // Step 0: the binding itself. A typo in a trust binding is never ignored.
    for (k, _) in kv {
        if !ALLOWED_KEYS.contains(&k.as_str()) {
            return (
                vec![abstain("PROOF-TAMPERED", format!("unknown key `{k}` in this [[proof.import]] table; allowed keys: {}", ALLOWED_KEYS.join(" ")))],
                3,
            );
        }
    }
    let get = |k: &str| getk(kv, k);
    let (Some(proofv), Some(contract_id), Some(anchor), Some(fieldsv)) =
        (get("proof"), get("contract_id"), get("anchor"), get("fields"))
    else {
        return (
            vec![abstain("PROOF-TAMPERED", "this [[proof.import]] table is missing a required key (proof, contract_id, anchor, fields)".to_string())],
            3,
        );
    };

    // Step 2 (binding side): v1 verifies egress contracts only (ruling D5).
    let bkind = contract_id.split(':').next().unwrap_or("");
    if bkind != "egress" {
        return (vec![abstain("PROOF-UNSUPPORTED-KIND", format!("the binding names kind {bkind}; v1 verifies egress contracts only"))], 4);
    }

    // Step 0/1: resolve and parse (digest verified inside parse_proof).
    let p = base.join(&proofv);
    let (ppath, proof) = if p.is_dir() {
        let mut entries: Vec<String> = match std::fs::read_dir(&p) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.ends_with(".proof"))
                .collect(),
            Err(e) => return (vec![abstain("PROOF-NOT-FOUND", format!("{proofv} is not readable: {e}"))], 4),
        };
        entries.sort();
        let mut matches: Vec<(String, Proof)> = Vec::new();
        for name in &entries {
            let full = format!("{proofv}/{name}");
            let text = match std::fs::read_to_string(p.join(name)) {
                Ok(t) => t,
                Err(e) => return (vec![abstain("PROOF-TAMPERED", format!("proof {full}: unreadable: {e}"))], 3),
            };
            match parse_proof(&full, &text) {
                Ok(pr) => {
                    if pr.contract == contract_id {
                        matches.push((full, pr));
                    }
                }
                Err(e) => return (vec![abstain("PROOF-TAMPERED", format!("proof {full}: {e}"))], 3),
            }
        }
        match matches.len() {
            0 => return (vec![abstain("PROOF-NOT-FOUND", format!("no *.proof in {proofv} attests {contract_id}"))], 4),
            1 => matches.remove(0),
            n => {
                let names: Vec<String> = matches.iter().map(|(f, _)| f.clone()).collect();
                return (
                    vec![abstain("PROOF-AMBIG-SCHEMA", format!("{n} proofs in {proofv} attest {contract_id}: {}; the binding must resolve to one", names.join(", ")))],
                    4,
                );
            }
        }
    } else if !p.exists() {
        return (vec![abstain("PROOF-NOT-FOUND", format!("{proofv} does not exist"))], 4);
    } else {
        let text = match std::fs::read_to_string(&p) {
            Ok(t) => t,
            Err(e) => return (vec![abstain("PROOF-TAMPERED", format!("proof {proofv}: unreadable: {e}"))], 3),
        };
        match parse_proof(&proofv, &text) {
            Ok(pr) => (proofv.clone(), pr),
            Err(e) => return (vec![abstain("PROOF-TAMPERED", format!("proof {proofv}: {e}"))], 3),
        }
    };

    // Step 2 (proof side) and step 3: kind, then scope.
    let pkind = proof.contract.split(':').next().unwrap_or("");
    if pkind != "egress" {
        return (vec![abstain("PROOF-UNSUPPORTED-KIND", format!("proof {ppath} attests kind {pkind}; v1 verifies egress contracts only"))], 4);
    }
    if proof.contract != contract_id {
        return (
            vec![abstain("PROOF-SCOPE-MISMATCH", format!("the binding names {contract_id} but proof {ppath} attests {}", proof.contract))],
            4,
        );
    }
    if proof.anchor != anchor {
        return (
            vec![abstain("PROOF-SCOPE-MISMATCH", format!("the anchor differs: binding \"{anchor}\", proof \"{}\" ({ppath})", proof.anchor))],
            4,
        );
    }

    // Step 5: freshness relative to the consumer's recorded acceptance (B1,
    // ruling D3: mandatory; HOLDS is never offered unanchored).
    let mut notes: Vec<String> = Vec::new();
    let (acc_e, acc_d) = match (get("accepted_epoch"), get("accepted_digest")) {
        (Some(e), Some(d)) => (e, d),
        _ => {
            return (
                vec![abstain("TEMPORAL-STALENESS-UNANCHORED", "no accepted_epoch/accepted_digest recorded; freshness cannot be established and HOLDS is not offered (record the reviewed epoch and digest)".to_string())],
                4,
            )
        }
    };
    let Ok(acc_epoch) = acc_e.parse::<u64>() else {
        return (vec![abstain("PROOF-TAMPERED", format!("accepted_epoch \"{acc_e}\" is not an integer"))], 3);
    };
    if proof.epoch < acc_epoch {
        return (
            vec![abstain("STALE-PROOF-COMMIT", format!("proof {ppath} is epoch {} but epoch {acc_epoch} was already accepted; an older proof was replayed or rolled back", proof.epoch))],
            4,
        );
    }
    if proof.epoch == acc_epoch && proof.digest != acc_d {
        return (
            vec![abstain("PROOF-FORK", format!("proof {ppath} carries epoch {} with a digest that differs from the accepted one; two proofs claim the same epoch", proof.epoch))],
            3,
        );
    }
    if proof.epoch == acc_epoch + 1 && proof.parent_digest != acc_d {
        return (
            vec![abstain("PROOF-FORK", format!("proof {ppath} is epoch {} but its parent_digest does not chain to the accepted digest of epoch {acc_epoch}", proof.epoch))],
            3,
        );
    }
    if proof.epoch > acc_epoch + 1 {
        notes.push(format!("  note: epoch gap ({acc_epoch} accepted, {} presented); the intermediate chain cannot be verified offline (stated, not alarmed)", proof.epoch));
    }

    // R-P-8: the one opt-in, clock-dependent step.
    if let Some(window) = get("max_age_days") {
        let Ok(days) = window.parse::<i64>() else {
            return (vec![abstain("PROOF-TAMPERED", format!("max_age_days \"{window}\" is not an integer"))], 3);
        };
        let Some(gen_day) = proof.generated_at.get(..10).and_then(lock::parse_date) else {
            return (vec![abstain("PROOF-TAMPERED", format!("generated_at \"{}\" is not parsable", proof.generated_at))], 3);
        };
        if today - gen_day > days {
            return (
                vec![abstain("TEMPORAL-STALENESS-UNANCHORED", format!("proof {ppath} generated_at {} exceeds max_age_days = {days} (clock-dependent, opt-in check)", proof.generated_at))],
                4,
            );
        }
    }

    // Step 6: every relied-on field must be present; extras are tolerated.
    let mut relied: Vec<String> = fieldsv.split_whitespace().map(str::to_string).collect();
    relied.sort();
    relied.dedup();
    let missing: Vec<&String> = relied.iter().filter(|f| !proof.names.contains(f)).collect();
    let mut out = Vec::new();
    if proof.fingerprint != engine_fp {
        out.push(format!(
            "NOTE (informational, never a verdict): proof {ppath} carries rule_fingerprint {} but this engine's is {}",
            &proof.fingerprint[..12.min(proof.fingerprint.len())],
            &engine_fp[..12]
        ));
    }
    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
        out.push(format!(
            "[VIOLATED] proof {contract_id}: relied-on field(s) missing from the proof: {} (proof {ppath}, epoch {})",
            names.join(" "),
            proof.epoch
        ));
        out.extend(notes);
        return (out, 1);
    }
    out.push(format!("[HOLDS] proof {contract_id} epoch {} ({ppath})", proof.epoch));
    let extras: Vec<&String> = proof.names.iter().filter(|n| !relied.contains(n)).collect();
    if !extras.is_empty() {
        let names: Vec<&str> = extras.iter().map(|s| s.as_str()).collect();
        out.push(format!("  note: the provider also lists field(s) this repository does not rely on: {}", names.join(" ")));
    }
    out.extend(notes);
    (out, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_layout_rule() {
        assert!(valid_out("invar.proof"));
        assert!(valid_out(".invar/proofs/orders.proof"));
        assert!(valid_out(".invar/proofs/a-b_C9.proof"));
        assert!(!valid_out("somewhere/else.proof"));
        assert!(!valid_out(".invar/proofs/bad name.proof"));
        assert!(!valid_out(".invar/proofs/.proof"));
        assert!(!valid_out(".invar/proofs/nested/x.proof"));
    }

    #[test]
    fn precedence_is_3_2_1_4_0() {
        assert_eq!(worse(0, 4), 4);
        assert_eq!(worse(4, 1), 1);
        assert_eq!(worse(1, 2), 2);
        assert_eq!(worse(2, 3), 3);
        assert_eq!(worse(3, 0), 3);
    }

    #[test]
    fn verification_budget_under_2ms() {
        // R-P-9, measured in release at Gate 4: one verification, averaged
        // over 200 runs against a real file and manifest in a temp dir.
        let mut p = Proof {
            fingerprint: rule_fingerprint(),
            repo: "none".into(),
            commit: "a".repeat(40),
            source_blob: "b".repeat(40),
            boundary: "app/api/v1/x/route.ts".into(),
            shape_file: "lib/s.ts".into(),
            shape: "X".into(),
            contract: "egress:X".into(),
            anchor: "GET /v1/x".into(),
            names: vec!["a".into(), "b".into(), "c".into()],
            epoch: 3,
            parent_digest: "c".repeat(64),
            digest: String::new(),
            generated_at: "2026-10-18T00:00:00Z".into(),
        };
        p.digest = canonical_digest(&render_canonical(&p));
        let dir = std::env::temp_dir().join(format!("invar_prf_budget_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("x.proof"), render_file(&p)).unwrap();
        let raw = format!(
            "[[proof.import]]\nproof = \"x.proof\"\ncontract_id = \"egress:X\"\nanchor = \"GET /v1/x\"\nfields = \"a b\"\naccepted_epoch = \"3\"\naccepted_digest = \"{}\"\n",
            p.digest
        );
        let sections = crate::config::parse_sections("invar.toml", &raw).unwrap();
        let (lines, code) = verify(&dir, &raw, &sections, lock::today_utc());
        assert_eq!(code, 0, "{lines:?}");
        if cfg!(debug_assertions) {
            return; // the budget is a release-build bound (R-P-9)
        }
        let t = std::time::Instant::now();
        for _ in 0..200 {
            let _ = verify(&dir, &raw, &sections, 0);
        }
        let per = t.elapsed() / 200;
        assert!(per < std::time::Duration::from_millis(2), "verification took {per:?} per proof (budget 2 ms)");
    }

    #[test]
    fn canonical_roundtrip_and_tamper() {
        let mut p = Proof {
            fingerprint: rule_fingerprint(),
            repo: "none".into(),
            commit: "a".repeat(40),
            source_blob: "b".repeat(40),
            boundary: "app/api/v1/x/route.ts".into(),
            shape_file: "lib/s.ts".into(),
            shape: "X".into(),
            contract: "egress:X".into(),
            anchor: "GET /v1/x".into(),
            names: vec!["a".into(), "b".into()],
            epoch: 1,
            parent_digest: "none".into(),
            digest: String::new(),
            generated_at: "2026-10-18T00:00:00Z".into(),
        };
        p.digest = canonical_digest(&render_canonical(&p));
        let text = render_file(&p);
        let back = parse_proof("t.proof", &text).unwrap();
        assert_eq!(back, p);
        let tampered = text.replace("names = \"a b\"", "names = \"a b c\"");
        assert!(parse_proof("t.proof", &tampered).unwrap_err().contains("digest mismatch"));
        // generated_at is outside the digest: editing it never tampers.
        let aged = text.replace("2026-10-18T00:00:00Z", "2020-01-01T00:00:00Z");
        assert!(parse_proof("t.proof", &aged).is_ok());
    }
}
