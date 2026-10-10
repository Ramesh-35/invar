//! A2 evidence receipts (spec: docs/architecture/A2-RECEIPTS.md, ratified
//! 2026-10-10).
//!
//! A receipt states: these exact input bytes, evaluated by this exact engine
//! version on this stated date, produced this exact verdict. The file is the
//! canonical body (sorted keys, two-space indent, LF, no floats, no wall
//! clock, no machine state) followed by a one-line footer carrying the body's
//! BLAKE3. `verify-receipt` re-runs the evaluation and must reproduce the
//! body byte for byte.
//!
//! Deliberately absent from the body: the engine binary hash (differs per
//! platform; the gate requires identical receipts across platforms), absolute
//! paths, hostnames, and the ledger (machine-local state). `result.exit` is
//! therefore the contract verdict's exit code, never the ledger's.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::check::ContractOutcome;
use crate::resolver::{FetchRequest, Transport};

pub const RECEIPT_FORMAT: u64 = 1;
/// Rule generations active in this engine, stated so a verifier knows what
/// the verdict means.
pub const RULES: [&str; 4] = ["A1", "D1", "E1", "F1"];

fn status(o: &ContractOutcome) -> &'static str {
    if o.skipped.is_some() {
        "skipped"
    } else if o.violated() {
        "violated"
    } else if !o.unevaluated.is_empty() {
        "not_fully_evaluated"
    } else if !o.waived.is_empty() {
        "holds_waived"
    } else {
        "holds"
    }
}

fn contract_json(o: &ContractOutcome) -> Value {
    json!({
        "expired_waivers": o.expired.iter().map(|(f, owner)| json!({"field": f, "owner": owner})).collect::<Vec<_>>(),
        "id": o.id,
        "kind": o.kind,
        "notes": o.notes,
        "residue": o.witnesses.iter().map(|w| json!({
            "field": w.residue,
            "missing_from": w.missing_from,
            "read_as": w.read_as,
            "read_at": w.sites,
            "type_change": w.type_change.as_ref().map(|(a, b, at)| json!({"from": a, "in": at, "to": b})),
        })).collect::<Vec<_>>(),
        "skipped_reason": o.skipped,
        "status": status(o),
        "unevaluated": o.unevaluated,
        "waived": o.waived.iter().map(|(f, owner, reason, left)| json!({
            "days_remaining": left, "field": f, "owner": owner, "reason": reason
        })).collect::<Vec<_>>(),
    })
}

/// The exit code the contract verdict carries (0 hold, 1 violated, 4 not
/// fully evaluated). The ledger's exit 2 is machine-local and never recorded.
pub fn contracts_exit(outcomes: &[ContractOutcome]) -> i32 {
    let violations = outcomes.iter().filter(|o| o.violated()).count();
    let stated = outcomes
        .iter()
        .filter(|o| o.skipped.is_some() || (!o.violated() && !o.unevaluated.is_empty()))
        .count();
    if violations > 0 {
        1
    } else if stated > 0 {
        4
    } else {
        0
    }
}

/// Build the canonical body. `as_of` is the evaluation date (an input, so
/// waiver arithmetic reproduces forever); `config_name` is a basename, never
/// a path.
pub fn body(
    engine_version: &str,
    as_of: &str,
    config_name: &str,
    config_blake3: &str,
    lock_blake3: Option<&str>,
    outcomes: &[ContractOutcome],
) -> Value {
    let mut artifacts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for o in outcomes {
        for f in &o.provenance {
            artifacts.insert((f.reference.clone(), f.blake3_hex.clone()), f.bytes);
        }
    }
    let mut contracts: Vec<&ContractOutcome> = outcomes.iter().collect();
    contracts.sort_by(|a, b| a.id.cmp(&b.id));
    let exit = contracts_exit(outcomes);
    json!({
        "artifacts": artifacts.iter().map(|((r, h), b)| json!({
            "blake3": h, "bytes": b, "ref": r
        })).collect::<Vec<_>>(),
        "as_of": as_of,
        "config": {"blake3": config_blake3, "name": config_name},
        "contracts": contracts.iter().map(|o| contract_json(o)).collect::<Vec<_>>(),
        "engine": {"rules": RULES, "version": engine_version},
        "lock": lock_blake3.map(|h| json!({"blake3": h})),
        "receipt_format": RECEIPT_FORMAT,
        "result": {
            "exit": exit,
            "verdict": match exit { 0 => "all_hold", 1 => "violated", _ => "not_fully_evaluated" },
        },
    })
}

/// Canonical bytes: pretty JSON (two-space indent, sorted keys by
/// construction), one LF, then the footer line with the body's BLAKE3.
pub fn render(body: &Value) -> String {
    let b = serde_json::to_string_pretty(body).unwrap_or_default();
    let h = blake3::hash(b.as_bytes()).to_hex().to_string();
    format!("{b}\n{{\"receipt_blake3\":\"{h}\"}}\n")
}

pub struct Parsed {
    pub body: Value,
    pub body_text: String,
}

/// Split and check the footer. Any failure here means a corrupted or foreign
/// file: the caller reports ERROR (exit 3), not a verdict.
pub fn parse(text: &str) -> Result<Parsed> {
    let marker = "\n{\"receipt_blake3\":\"";
    let idx = text
        .rfind(marker)
        .ok_or_else(|| anyhow!("no receipt footer; not a receipt file"))?;
    let body_text = &text[..idx];
    let footer: Value = serde_json::from_str(text[idx..].trim())
        .map_err(|e| anyhow!("unreadable receipt footer: {e}"))?;
    let claimed = footer
        .get("receipt_blake3")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("footer has no receipt_blake3"))?;
    let actual = blake3::hash(body_text.as_bytes()).to_hex().to_string();
    if claimed != actual {
        return Err(anyhow!(
            "corrupted receipt: footer blake3 {claimed} does not match body {actual}"
        ));
    }
    let body: Value =
        serde_json::from_str(body_text).map_err(|e| anyhow!("unreadable receipt body: {e}"))?;
    Ok(Parsed {
        body,
        body_text: body_text.to_string(),
    })
}

/// Offline artifact source for `verify-receipt --artifacts <dir>`: files
/// named by their BLAKE3 hex. Tokenless `github:` URLs are inverted back to
/// refs and looked up in the receipt's artifact list; anything unknown is an
/// error, never a guess.
pub struct DirTransport {
    pub dir: std::path::PathBuf,
    /// ref -> blake3 hex, from the receipt's `artifacts`.
    pub by_ref: BTreeMap<String, String>,
}

impl DirTransport {
    pub fn from_receipt(dir: &std::path::Path, body: &Value) -> Self {
        let mut by_ref = BTreeMap::new();
        for a in body
            .get("artifacts")
            .and_then(Value::as_array)
            .unwrap_or(&Vec::new())
        {
            if let (Some(r), Some(h)) = (
                a.get("ref").and_then(Value::as_str),
                a.get("blake3").and_then(Value::as_str),
            ) {
                by_ref.insert(r.to_string(), h.to_string());
            }
        }
        DirTransport {
            dir: dir.to_path_buf(),
            by_ref,
        }
    }
}

impl Transport for DirTransport {
    fn get(&self, req: &FetchRequest) -> Result<Vec<u8>, String> {
        let rest = req
            .url
            .strip_prefix("https://raw.githubusercontent.com/")
            .ok_or_else(|| format!("offline verify: unexpected url {}", req.url))?;
        let mut it = rest.splitn(4, '/');
        let (owner, repo, git_ref, path) = (
            it.next().unwrap_or(""),
            it.next().unwrap_or(""),
            it.next().unwrap_or(""),
            it.next().unwrap_or(""),
        );
        let reference = format!("github:{owner}/{repo}@{git_ref}:{path}");
        let hash = self.by_ref.get(&reference).ok_or_else(|| {
            format!("offline verify: {reference} is not in the receipt's artifacts")
        })?;
        std::fs::read(self.dir.join(hash))
            .map_err(|e| format!("offline verify: artifact {hash} ({reference}): {e}"))
    }
}

/// A2.1: the receipt body wrapped as an in-toto Statement v1.0, an ADDITIVE
/// export (CONFLICTS ruling 3). Format 1 stays canonical; this is a view of
/// it for in-toto consumers. Subjects are the content-addressed inputs; the
/// predicate is the format 1 body verbatim.
pub fn to_in_toto(body: &Value) -> Value {
    let mut subjects = Vec::new();
    if let (Some(name), Some(h)) = (
        body.pointer("/config/name").and_then(Value::as_str),
        body.pointer("/config/blake3").and_then(Value::as_str),
    ) {
        subjects.push(json!({"digest": {"blake3": h}, "name": name}));
    }
    if let Some(h) = body.pointer("/lock/blake3").and_then(Value::as_str) {
        subjects.push(json!({"digest": {"blake3": h}, "name": "invar.lock"}));
    }
    for a in body
        .get("artifacts")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
    {
        if let (Some(r), Some(h)) = (
            a.get("ref").and_then(Value::as_str),
            a.get("blake3").and_then(Value::as_str),
        ) {
            subjects.push(json!({"digest": {"blake3": h}, "name": r}));
        }
    }
    json!({
        "_type": "https://in-toto.io/Statement/v1",
        "predicate": body,
        "predicateType": "https://github.com/Ramesh-35/invar-platform/receipt/v1",
        "subject": subjects,
    })
}

/// Verify a detached OpenSSH signature over `payload` by shelling out to
/// ssh-keygen (same standing as the curl subprocess). Ok(true) verified,
/// Ok(false) signature rejected, Err only when ssh-keygen cannot run.
pub fn verify_ssh(
    payload: &[u8],
    sig_file: &std::path::Path,
    signers_file: &std::path::Path,
    principal: &str,
    namespace: &str,
) -> Result<bool> {
    use std::io::Write as _;
    let mut child = std::process::Command::new("ssh-keygen")
        .arg("-Y")
        .arg("verify")
        .arg("-f")
        .arg(signers_file)
        .arg("-I")
        .arg(principal)
        .arg("-n")
        .arg(namespace)
        .arg("-s")
        .arg(sig_file)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| anyhow!("running ssh-keygen: {e}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("ssh-keygen stdin unavailable"))?
        .write_all(payload)?;
    Ok(child.wait()?.success())
}
