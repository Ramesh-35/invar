//! JUDGE-1 (docs/falsification/JUDGE-SPEC.md r1): `invar judge`, the
//! declared-data-contract verdict. The PRE contract is read from the base
//! ref via `git archive`, the way SIB-3 reads trust: the base tree, never
//! the working tree. The POST contract is the working tree. A property
//! removed or retyped is VIOLATED with a dotted-path witness; what r0 does
//! not judge is STATED in every output as `unjudged_constraints`, never
//! silently passed. The text format speaks to a person in sentences; the
//! JSON format carries the same facts, deterministically ordered, for
//! machines and agents.

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

/// The constraint classes r0 does not judge, stated in every receipt.
/// What r1 does not judge, stated in every output (JUDGE-SUBTYPE r1).
/// `$ref` is a gap r0 had and never stated: references are not followed.
const UNJUDGED: [&str; 5] = ["$ref", "allOf", "anyOf", "oneOf", "items"];

/// Lower bounds tighten when raised; upper bounds tighten when lowered.
const LOWER: [&str; 4] = ["minimum", "exclusiveMinimum", "minLength", "minItems"];
const UPPER: [&str; 4] = ["maximum", "exclusiveMaximum", "maxLength", "maxItems"];

pub struct Witness {
    pub path: String,
    /// removed | retyped | narrowed | tightened | escalated | constraint_changed
    pub kind: &'static str,
    pub pre_type: String,
    pub post_type: Option<String>,
    pub pre_line: Option<u64>,
    pub post_line: Option<u64>,
    /// The constraint keyword for the r1 kinds; None for removed and retyped.
    pub keyword: Option<&'static str>,
    pub pre_value: Option<String>,
    pub post_value: Option<String>,
    /// JUDGE-SARIF S-D1: where a removed field used to live in the POST
    /// file (its parent's key line). Not part of the JSON surface.
    pub anchor_line: Option<u64>,
}

enum BaseRead {
    Found(Vec<u8>),
    NoRef,
    NoFile,
}

/// Judge the working-tree contract against the base ref. Returns the exit
/// code (0 HOLDS, 1 VIOLATED, 4 NOT EVALUATED); real failures (not a git
/// repository, unwritable receipt) surface as errors and exit 3 upstream.
pub fn run(root: &Path, contract: &str, base: &str, format: &str, receipt_out: Option<&str>) -> Result<i32> {
    // Bytes are read raw and decoded lossily: malformed encoding is a
    // contract that cannot be parsed (NOT EVALUATED unparsable, exit 4),
    // never a process error. Only a missing or unreadable FILE is an error.
    let post_raw = match std::fs::read(root.join(contract)) {
        Ok(b) => b,
        Err(e) => anyhow::bail!("reading {contract} from the working tree: {e}"),
    };
    let post_bytes = String::from_utf8_lossy(&post_raw).into_owned();
    let read = base_bytes(root, base, contract)?;
    let base_b3 = match &read {
        BaseRead::Found(b) => Some(blake3::hash(b).to_hex().to_string()),
        _ => None,
    };
    let (verdict, reason, witnesses, pre_text) = evaluate(read, &post_bytes);
    let _ = pre_text; // pre bytes are consumed by line lookup in evaluate
    let body = body_json(verdict, reason, contract, base, &witnesses);
    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&body).unwrap_or_default()),
        "sarif" => println!("{}", serde_json::to_string_pretty(&render_sarif(verdict, reason, contract, base, &witnesses)).unwrap_or_default()),
        "github" => print!("{}", render_github(verdict, reason, contract, base, &witnesses)),
        _ => print!("{}", render_text(verdict, reason, contract, base, &witnesses)),
    }
    if let Some(p) = receipt_out {
        let inputs = ReceiptInputs {
            base_ref: base,
            base_commit: resolve_commit(root, base),
            base_contract_blake3: base_b3,
            contract,
            post_contract_blake3: blake3::hash(&post_raw).to_hex().to_string(),
        };
        let receipt = receipt_body(&inputs, body);
        std::fs::write(p, crate::receipt::render(&receipt)).with_context(|| format!("writing receipt {p}"))?;
    }
    Ok(match verdict {
        "HOLDS" => 0,
        "VIOLATED" => 1,
        _ => 4,
    })
}

/// The judgment proper: base contract (as read from git) against the
/// working-tree bytes. Pure: the same inputs give the same verdict,
/// reason and witnesses, which is what lets a receipt be reproduced.
#[allow(clippy::type_complexity)]
fn evaluate(read: BaseRead, post_bytes: &str) -> (&'static str, Option<&'static str>, Vec<Witness>, Option<String>) {
    match read {
            BaseRead::NoRef => ("NOT EVALUATED", Some("unreadable_base"), Vec::new(), None),
            BaseRead::NoFile => ("NOT EVALUATED", Some("new_contract"), Vec::new(), None),
            BaseRead::Found(bytes) => {
                let pre_text = String::from_utf8_lossy(&bytes).into_owned();
                match (serde_json::from_str::<Value>(&pre_text), serde_json::from_str::<Value>(post_bytes)) {
                    (Err(_), _) | (_, Err(_)) => ("NOT EVALUATED", Some("unparsable"), Vec::new(), Some(pre_text)),
                    (Ok(pre), Ok(post)) => {
                        let mut w = Vec::new();
                        walk("", &pre, &post, &mut w);
                        w.sort_by(|a, b| (a.path.as_str(), a.kind, a.keyword).cmp(&(b.path.as_str(), b.kind, b.keyword)));
                        for x in &mut w {
                            let last = x.path.rsplit('.').next().unwrap_or(&x.path).to_string();
                            x.pre_line = find_line(&pre_text, &last);
                            if x.kind != "removed" {
                                x.post_line = find_line(post_bytes, &last);
                            } else {
                                let parent = x.path.rsplit('.').nth(1).unwrap_or("properties");
                                x.anchor_line = find_line(post_bytes, parent);
                            }
                        }
                        let v = if w.is_empty() { "HOLDS" } else { "VIOLATED" };
                        (v, None, w, Some(pre_text))
                    }
                }
            }
    }
}

/// What a judge receipt names as judged (JUDGE-RECEIPT r0).
struct ReceiptInputs<'a> {
    base_ref: &'a str,
    base_commit: Option<String>,
    base_contract_blake3: Option<String>,
    contract: &'a str,
    post_contract_blake3: String,
}

/// The commit a ref resolves to, or None when it does not resolve.
fn resolve_commit(root: &Path, base: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", &format!("{base}^{{commit}}")])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// JUDGE-RECEIPT r0 body: the inputs judged, and the `--format json`
/// document unchanged as `result`. Sorted keys, no wall clock.
fn receipt_body(i: &ReceiptInputs, result: Value) -> Value {
    json!({
        "engine": {"version": env!("CARGO_PKG_VERSION")},
        "inputs": {
            "base_commit": i.base_commit,
            "base_contract_blake3": i.base_contract_blake3,
            "base_ref": i.base_ref,
            "contract": i.contract,
            "post_contract_blake3": i.post_contract_blake3,
        },
        "receipt_format": crate::receipt::RECEIPT_FORMAT,
        "receipt_kind": "judge",
        "result": result,
    })
}

/// JUDGE-RECEIPT r0 verification, after the footer, format, engine version
/// and optional signature have been checked upstream: the inputs must be
/// the recorded bytes, and re-judging them must reproduce the recorded
/// body byte for byte. 0 reproduced, 1 drift or not reproduced; a base
/// commit this repository lacks is an error (exit 3 upstream).
pub fn verify_receipt(root: &Path, body: &Value, body_text: &str) -> Result<i32> {
    let s = |k: &str| body.pointer(&format!("/inputs/{k}")).and_then(Value::as_str).map(str::to_string);
    let contract = s("contract").ok_or_else(|| anyhow::anyhow!("judge receipt has no inputs.contract"))?;
    let base_ref = s("base_ref").ok_or_else(|| anyhow::anyhow!("judge receipt has no inputs.base_ref"))?;
    let base_commit = s("base_commit");
    let recorded_post = s("post_contract_blake3").unwrap_or_default();
    let recorded_base = s("base_contract_blake3");

    let post_raw = std::fs::read(root.join(&contract)).with_context(|| format!("reading {contract} from the working tree"))?;
    let post_b3 = blake3::hash(&post_raw).to_hex().to_string();
    if post_b3 != recorded_post {
        println!("input drift: {contract} is not the contract the receipt judged");
        println!("result: receipt NOT reproduced");
        return Ok(1);
    }
    let read = match base_commit.as_deref() {
        Some(base_commit) => match base_bytes(root, base_commit, &contract)? {
            BaseRead::NoRef => anyhow::bail!("base commit {base_commit} is not in this repository; fetch it, then verify"),
            other => other,
        },
        None => BaseRead::NoRef,
    };
    let base_b3 = match &read {
        BaseRead::Found(b) => Some(blake3::hash(b).to_hex().to_string()),
        _ => None,
    };
    if base_b3.as_deref() != recorded_base.as_deref() {
        println!("input drift: the base contract at {} is not the one the receipt judged", base_commit.as_deref().unwrap_or("(no commit)"));
        println!("result: receipt NOT reproduced");
        return Ok(1);
    }
    let post_bytes = String::from_utf8_lossy(&post_raw).into_owned();
    let (verdict, reason, witnesses, _) = evaluate(read, &post_bytes);
    let inputs = ReceiptInputs {
        base_ref: &base_ref,
        base_commit,
        base_contract_blake3: base_b3,
        contract: &contract,
        post_contract_blake3: post_b3,
    };
    let rederived = receipt_body(&inputs, body_json(verdict, reason, &contract, &base_ref, &witnesses));
    let rendered = serde_json::to_string_pretty(&rederived).unwrap_or_default();
    if rendered != body_text {
        println!("re-judging the recorded inputs gives a different result ({verdict})");
        println!("result: receipt NOT reproduced");
        return Ok(1);
    }
    println!("inputs ok: {contract} and its base at {}", inputs.base_commit.as_deref().unwrap_or("(no commit)"));
    println!("result: receipt reproduced ({verdict})");
    Ok(0)
}

/// Read the contract from the base ref via `git archive`: the base tree,
/// never the working tree (JUDGE-SPEC r1, the SIB-3 trust discipline).
fn base_bytes(root: &Path, base: &str, contract: &str) -> Result<BaseRead> {
    let ok = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", &format!("{base}^{{commit}}")])
        .current_dir(root)
        .output()
        .context("running git rev-parse")?;
    if !ok.status.success() {
        return Ok(BaseRead::NoRef);
    }
    let out = Command::new("git")
        .args(["archive", "--format=tar", base, "--", contract])
        .current_dir(root)
        .output()
        .context("running git archive")?;
    if !out.status.success() {
        return Ok(BaseRead::NoFile);
    }
    match untar_single(&out.stdout, contract) {
        Some(bytes) => Ok(BaseRead::Found(bytes)),
        None => Ok(BaseRead::NoFile),
    }
}

/// Minimal tar reader for the one file `git archive` was asked for:
/// 512-byte headers, name at 0..100 with optional prefix at 345..500,
/// size in octal at 124..136, typeflag at 156. Pax headers are skipped.
fn untar_single(tar: &[u8], want: &str) -> Option<Vec<u8>> {
    let mut i = 0;
    while i + 512 <= tar.len() {
        let hdr = &tar[i..i + 512];
        if hdr.iter().all(|&b| b == 0) {
            break;
        }
        let name = cstr(&hdr[0..100]);
        let prefix = cstr(&hdr[345..500]);
        let full = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
        let size = usize::from_str_radix(cstr(&hdr[124..136]).trim(), 8).unwrap_or(0);
        let typeflag = hdr[156];
        let data_start = i + 512;
        if full == want && (typeflag == b'0' || typeflag == 0) && data_start + size <= tar.len() {
            return Some(tar[data_start..data_start + size].to_vec());
        }
        i = data_start + size.div_ceil(512) * 512;
    }
    None
}

fn cstr(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}

/// Recursive diff over `properties`: presence and `type`, dotted paths.
/// A nested subtree the post drops entirely reports every pre field in it.
fn walk(prefix: &str, pre: &Value, post: &Value, out: &mut Vec<Witness>) {
    let empty = serde_json::Map::new();
    let pp = pre.get("properties").and_then(Value::as_object).unwrap_or(&empty);
    let qp = post.get("properties").and_then(Value::as_object).unwrap_or(&empty);
    for name in newly_required(pre, post) {
        let path = if prefix.is_empty() { name.clone() } else { format!("{prefix}.{name}") };
        let ty = pp.get(&name).or_else(|| qp.get(&name)).map(type_str).unwrap_or_default();
        out.push(Witness {
            path,
            kind: "escalated",
            pre_type: ty,
            post_type: None,
            pre_line: None,
            post_line: None,
            keyword: Some("required"),
            pre_value: None,
            post_value: None,
            anchor_line: None,
        });
    }
    for (k, pv) in pp {
        let dotted = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
        match qp.get(k) {
            None => out.push(Witness {
                path: dotted,
                kind: "removed",
                pre_type: type_str(pv),
                post_type: None,
                pre_line: None,
                post_line: None,
                keyword: None,
                pre_value: None,
                post_value: None,
                anchor_line: None,
            }),
            Some(qv) => {
                if type_str(pv) != type_str(qv) {
                    out.push(Witness {
                        path: dotted.clone(),
                        kind: "retyped",
                        pre_type: type_str(pv),
                        post_type: Some(type_str(qv)),
                        pre_line: None,
                        post_line: None,
                        keyword: None,
                        pre_value: None,
                        post_value: None,
                        anchor_line: None,
                    });
                }
                constraints(&dotted, pv, qv, out);
                walk(&dotted, pv, qv, out);
            }
        }
    }
}

/// Names newly present in an object's `required[]` (JUDGE-SUBTYPE JS-D4).
fn newly_required(pre: &Value, post: &Value) -> Vec<String> {
    let names = |v: &Value| -> Vec<String> {
        v.get("required")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    let before = names(pre);
    let mut added: Vec<String> = names(post).into_iter().filter(|n| !before.contains(n)).collect();
    added.sort();
    added.dedup();
    added
}

/// True when post's `enum` accepts less than pre's (JS-D1): a value lost,
/// or an enum added where there was none. Dropping the enum widens.
fn narrowed_enum(pv: &Value, qv: &Value) -> bool {
    match (pv.get("enum").and_then(Value::as_array), qv.get("enum").and_then(Value::as_array)) {
        (_, None) => false,
        (None, Some(_)) => true,
        (Some(a), Some(b)) => a.iter().any(|x| !b.contains(x)),
    }
}

/// A constraint value as people read it: strings bare, everything else JSON.
fn disp(v: Option<&Value>) -> Option<String> {
    v.map(|x| x.as_str().map(str::to_string).unwrap_or_else(|| x.to_string()))
}

fn constraint_witness(path: &str, kind: &'static str, keyword: &'static str, pv: &Value, a: Option<&Value>, b: Option<&Value>) -> Witness {
    Witness {
        path: path.to_string(),
        kind,
        pre_type: type_str(pv),
        post_type: None,
        pre_line: None,
        post_line: None,
        keyword: Some(keyword),
        pre_value: disp(a),
        post_value: disp(b),
        anchor_line: None,
    }
}

/// The JUDGE-SUBTYPE r1 validators on one property present at both refs:
/// enum narrowing, bound tightening, pattern and format changes. A schema
/// is read as a validation contract: narrowing what it accepts is VIOLATED.
fn constraints(path: &str, pv: &Value, qv: &Value, out: &mut Vec<Witness>) {
    if narrowed_enum(pv, qv) {
        out.push(constraint_witness(path, "narrowed", "enum", pv, pv.get("enum"), qv.get("enum")));
    }
    let bounds = LOWER.iter().map(|k| (*k, true)).chain(UPPER.iter().map(|k| (*k, false)));
    for (kw, is_lower) in bounds {
        let (ra, rb) = (pv.get(kw), qv.get(kw));
        if ra == rb || rb.is_none() {
            continue; // unchanged, or the bound was removed: a loosening
        }
        let numeric_a = !matches!(ra, Some(v) if !v.is_number());
        match (ra.and_then(Value::as_f64), rb.and_then(Value::as_f64)) {
            (a, Some(b)) if numeric_a => {
                let tightened = match a {
                    None => true,
                    Some(a) => (is_lower && b > a) || (!is_lower && b < a),
                };
                if tightened {
                    out.push(constraint_witness(path, "tightened", kw, pv, ra, rb));
                }
            }
            // A non-numeric bound (draft-04 boolean exclusiveMinimum, say)
            // changed: flagged as changed, never silently passed.
            _ => out.push(constraint_witness(path, "constraint_changed", kw, pv, ra, rb)),
        }
    }
    for kw in ["pattern", "format"] {
        if pv.get(kw) != qv.get(kw) && qv.get(kw).is_some() {
            out.push(constraint_witness(path, "constraint_changed", kw, pv, pv.get(kw), qv.get(kw)));
        }
    }
}

/// A schema `type` as people write it: "integer|null" for ["integer","null"].
fn type_str(v: &Value) -> String {
    match v.get("type") {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| x.as_str().unwrap_or("?").to_string())
            .collect::<Vec<_>>()
            .join("|"),
        _ => "unspecified".to_string(),
    }
}

/// 1-based line of the first `"key":` occurrence in the byte stream. The
/// first occurrence is the stated r0 grain; a path that repeats a key name
/// at several depths reports the earliest, never none.
fn find_line(text: &str, key: &str) -> Option<u64> {
    let needle = format!("\"{key}\"");
    let mut from = 0;
    while let Some(pos) = text[from..].find(&needle) {
        let abs = from + pos;
        let rest = text[abs + needle.len()..].trim_start();
        if rest.starts_with(':') {
            return Some(text[..abs].bytes().filter(|&b| b == b'\n').count() as u64 + 1);
        }
        from = abs + needle.len();
    }
    None
}

/// One JSON document, deterministically ordered (sorted keys), the same
/// facts the text states: the machine surface for agents and CI.
fn body_json(verdict: &str, reason: Option<&str>, contract: &str, base: &str, w: &[Witness]) -> Value {
    json!({
        "verdict": verdict,
        "reason": reason,
        "contract": contract,
        "base": base,
        "judged": "property presence and type, enum, numeric, length and item-count bounds, required, pattern and format, recursive",
        "unjudged_constraints": UNJUDGED,
        "witnesses": w.iter().map(|x| json!({
            "path": x.path,
            "kind": x.kind,
            "pre_type": x.pre_type,
            "post_type": x.post_type,
            "contract": contract,
            "pre_line": x.pre_line,
            "post_line": x.post_line,
            "keyword": x.keyword,
            "pre_value": x.pre_value,
            "post_value": x.post_value,
        })).collect::<Vec<_>>(),
    })
}

/// The sentence the text format prints for one witness, without the
/// indent or newline. Every encoding (text, SARIF, GitHub) carries this
/// exact sentence: one voice (JUDGE-SARIF r0).
fn witness_sentence(x: &Witness, base: &str) -> String {
    match x.kind {
        "removed" => {
            let at = x.pre_line.map(|l| format!(" (line {l} at {base})")).unwrap_or_default();
            format!(
                "removed: {} was {}{at}. Consumers reading this field will break; keep the field, or ship this as a new major version and migrate them.",
                x.path, x.pre_type
            )
        }
        "narrowed" => {
            let post = x.post_value.as_deref().unwrap_or("[]");
            match x.pre_value.as_deref() {
                Some(pre) => {
                    let parse = |s: &str| serde_json::from_str::<Vec<Value>>(s).unwrap_or_default();
                    let (a, b) = (parse(pre), parse(post));
                    let lost: Vec<String> = a.iter().filter(|v| !b.contains(v)).map(|v| v.to_string()).collect();
                    format!(
                        "narrowed: {} no longer accepts {}. Senders using a removed value will be rejected; keep the value, or version the change.",
                        x.path,
                        lost.join(", ")
                    )
                }
                None => format!(
                    "narrowed: {} now accepts only {post}. Values outside that set, valid before, will be rejected; drop the enum, or version the change.",
                    x.path
                ),
            }
        }
        "tightened" => {
            let kw = x.keyword.unwrap_or("bound");
            let to = x.post_value.as_deref().unwrap_or("?");
            let change = match x.pre_value.as_deref() {
                None => format!("{kw} added at {to}"),
                Some(from) if LOWER.contains(&kw) => format!("{kw} raised from {from} to {to}"),
                Some(from) => format!("{kw} lowered from {from} to {to}"),
            };
            format!(
                "tightened: {} {change}. Values valid before can now be rejected; restore the bound, or version the change.",
                x.path
            )
        }
        "escalated" => format!(
            "escalated: {} is now required. Payloads that omit it, valid before, will be rejected; keep it optional, or version the change.",
            x.path
        ),
        "constraint_changed" => format!(
            "changed: {} {} was {}, is now {}. Whether this narrows what is accepted is not decided here; review it, or version the change.",
            x.path,
            x.keyword.unwrap_or("constraint"),
            x.pre_value.as_deref().unwrap_or("absent"),
            x.post_value.as_deref().unwrap_or("absent")
        ),
        _ => {
            let to = x.post_type.as_deref().unwrap_or("unspecified");
            format!(
                "retyped: {} was {}, is now {to}. Consumers parsing the old type can misread values; restore the type, or version the change.",
                x.path, x.pre_type
            )
        }
    }
}

/// The NOT EVALUATED sentence for a reason code, shared by every format.
fn not_evaluated_sentence(reason: Option<&str>, contract: &str, base: &str) -> String {
    match reason {
        Some("new_contract") => format!(
            "NOT EVALUATED (new_contract): {contract} does not exist at {base}; nothing was promised before. The next change to this contract will be judged against this version."
        ),
        Some("unreadable_base") => format!(
            "NOT EVALUATED (unreadable_base): the ref {base} cannot be read in this repository. Fetch it, or pass --base a ref that exists."
        ),
        _ => format!(
            "NOT EVALUATED (unparsable): could not read {contract} as JSON (checked the working tree and {base}). Fix the JSON before judging."
        ),
    }
}

/// The human surface, frozen as part of the contract (JUDGE-SPEC r1,
/// Voice): sentences that name the field, the change, and a next step.
fn render_text(verdict: &str, reason: Option<&str>, contract: &str, base: &str, w: &[Witness]) -> String {
    let mut s = String::new();
    match verdict {
        "HOLDS" => s.push_str(&format!("HOLDS: {contract} keeps every promise it made at {base}.\n")),
        "VIOLATED" => {
            let n = w.len();
            let noun = if n == 1 { "promise" } else { "promises" };
            s.push_str(&format!("VIOLATED: {contract} breaks {n} {noun} made at {base}.\n"));
            for x in w {
                s.push_str(&format!("  {}\n", witness_sentence(x, base)));
            }
        }
        _ => s.push_str(&format!("{}\n", not_evaluated_sentence(reason, contract, base))),
    }
    s.push_str(&format!(
        "judged: property presence and type, enum, numeric, length and item-count bounds, required, pattern and format, recursively. not judged: {} (stated, never skipped).\n",
        UNJUDGED.join(", ")
    ));
    s
}

/// JUDGE-SARIF S-D1: the post-file line for a witness; a removed field
/// anchors to its parent's key line in the post file; 1 when none.
fn location_line(x: &Witness) -> u64 {
    x.post_line.or(x.anchor_line).unwrap_or(1)
}

const RULES: [(&str, &str); 6] = [
    ("removed", "A property the base ref declared is gone."),
    ("retyped", "A property's declared type changed."),
    ("narrowed", "An enum accepts fewer values than at the base ref."),
    ("tightened", "A numeric, length or item-count bound rejects values valid at the base ref."),
    ("escalated", "A property became required."),
    ("constraint_changed", "A pattern, format or non-numeric bound changed; whether it narrows is not decided."),
];

/// JUDGE-SARIF r0: a SARIF 2.1.0 log. Sorted keys, no timestamps, no
/// absolute paths: byte-stable. NOT EVALUATED is a failed execution with
/// a warning notification, never an empty log that reads as clean (S-D2).
fn render_sarif(verdict: &str, reason: Option<&str>, contract: &str, base: &str, w: &[Witness]) -> Value {
    let results: Vec<Value> = w
        .iter()
        .map(|x| {
            json!({
                "ruleId": format!("invar.judge.{}", x.kind),
                "level": "error",
                "message": { "text": witness_sentence(x, base) },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": contract },
                        "region": { "startLine": location_line(x) },
                    }
                }],
            })
        })
        .collect();
    let notifications: Vec<Value> = match verdict {
        "HOLDS" | "VIOLATED" => Vec::new(),
        _ => vec![json!({
            "level": "warning",
            "descriptor": { "id": reason.unwrap_or("unparsable") },
            "message": { "text": not_evaluated_sentence(reason, contract, base) },
        })],
    };
    let rules: Vec<Value> = RULES
        .iter()
        .map(|(k, d)| json!({ "id": format!("invar.judge.{k}"), "shortDescription": { "text": d } }))
        .collect();
    json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": {
                "name": "invar",
                "version": env!("CARGO_PKG_VERSION"),
                "informationUri": "https://github.com/Ramesh-35/invar",
                "rules": rules,
            }},
            "invocations": [{
                "executionSuccessful": verdict != "NOT EVALUATED",
                "toolExecutionNotifications": notifications,
            }],
            "results": results,
            "properties": {
                "unjudged_constraints": UNJUDGED,
                "verdict": verdict,
                "base": base,
            },
        }],
    })
}

/// GitHub workflow-command escaping: message data escapes %, CR and LF;
/// property values (file, title) also escape `:` and `,`.
fn gh_data(s: &str) -> String {
    s.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A")
}

fn gh_prop(s: &str) -> String {
    gh_data(s).replace(':', "%3A").replace(',', "%2C")
}

/// JUDGE-SARIF r0: one `::error` per witness, located on the post line;
/// one `::warning` when NOT EVALUATED. HOLDS prints nothing.
fn render_github(verdict: &str, reason: Option<&str>, contract: &str, base: &str, w: &[Witness]) -> String {
    let file = gh_prop(contract);
    match verdict {
        "HOLDS" => String::new(),
        "VIOLATED" => w
            .iter()
            .map(|x| {
                format!(
                    "::error file={file},line={},title={}::{}\n",
                    location_line(x),
                    gh_prop(&format!("invar.judge.{}", x.kind)),
                    gh_data(&witness_sentence(x, base))
                )
            })
            .collect(),
        _ => format!(
            "::warning file={file},title=invar.judge.not_evaluated::{}\n",
            gh_data(&not_evaluated_sentence(reason, contract, base))
        ),
    }
}
