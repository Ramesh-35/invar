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
    /// COUNTEREXAMPLE r0: an instance valid at base and rejected now,
    /// checked before it is set; or why none was built.
    pub counterexample: Option<Value>,
    pub counterexample_reason: Option<String>,
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
    let read = base_bytes(root, base, contract)?;
    let base_b3 = match &read {
        BaseRead::Found(b) => Some(blake3::hash(b).to_hex().to_string()),
        _ => None,
    };
    let One { verdict, reason, witnesses, stated, .. } = judge_one(contract, read, Some(&post_raw));
    let body = body_json(verdict, reason, contract, base, &witnesses, &stated);
    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&body).unwrap_or_default()),
        "sarif" => println!("{}", serde_json::to_string_pretty(&render_sarif(verdict, reason, contract, base, &witnesses, &stated)).unwrap_or_default()),
        "github" => print!("{}", render_github(verdict, reason, contract, base, &witnesses)),
        _ => print!("{}", render_text(verdict, reason, contract, base, &witnesses, &stated)),
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

/// JUDGE-FILEPAIR r0: judge two files directly, no git anywhere. The
/// first path is the base (the promise), the second the head (the
/// change), and the paths as given are the labels every surface prints.
/// The engine is the repository mode's by construction: the same
/// judge_one, the same renderers. Only how the base bytes arrive
/// differs, so the two modes cannot diverge on the same bytes.
pub fn run_files(base_path: &str, head_path: &str, format: &str) -> Result<i32> {
    let base_raw = std::fs::read(base_path).map_err(|e| anyhow::anyhow!("reading {base_path}: {e}"))?;
    let head_raw = std::fs::read(head_path).map_err(|e| anyhow::anyhow!("reading {head_path}: {e}"))?;
    // Ad-hoc files that do not parse are operator errors (exit 3), unlike
    // a repository contract, whose unparsable bytes are a change under
    // judgment (NOT EVALUATED, exit 4). Ruled in JUDGE-FILEPAIR r0.
    serde_json::from_slice::<Value>(&base_raw).map_err(|e| anyhow::anyhow!("{base_path} is not JSON: {e}"))?;
    serde_json::from_slice::<Value>(&head_raw).map_err(|e| anyhow::anyhow!("{head_path} is not JSON: {e}"))?;
    let One { verdict, reason, witnesses, stated, .. } = judge_one(head_path, BaseRead::Found(base_raw), Some(&head_raw));
    let (c, b) = (head_path, base_path);
    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&body_json(verdict, reason, c, b, &witnesses, &stated)).unwrap_or_default()),
        "sarif" => println!("{}", serde_json::to_string_pretty(&render_sarif(verdict, reason, c, b, &witnesses, &stated)).unwrap_or_default()),
        "github" => print!("{}", render_github(verdict, reason, c, b, &witnesses)),
        _ => print!("{}", render_text(verdict, reason, c, b, &witnesses, &stated)),
    }
    Ok(match verdict {
        "HOLDS" => 0,
        "VIOLATED" => 1,
        _ => 4,
    })
}

/// One contract's judgment, as every surface renders it.
struct One {
    contract: String,
    verdict: &'static str,
    reason: Option<&'static str>,
    witnesses: Vec<Witness>,
    stated: Stated,
}

/// Judge one contract from its base read and its working-tree bytes
/// (None: the file is gone from the working tree).
fn judge_one(contract: &str, read: BaseRead, post_raw: Option<&[u8]>) -> One {
    let (verdict, reason, witnesses, stated) = match (post_raw, read) {
        // JUDGE-CONTRACTS M-D3: deleting a contract withdraws everything
        // it promised; stated, never a pass.
        (None, BaseRead::Found(_)) => ("NOT EVALUATED", Some("contract_deleted"), Vec::new(), Stated::default()),
        (None, BaseRead::NoRef) => ("NOT EVALUATED", Some("unreadable_base"), Vec::new(), Stated::default()),
        (None, BaseRead::NoFile) => ("NOT EVALUATED", Some("unparsable"), Vec::new(), Stated::default()),
        (Some(raw), read) => {
            let post_bytes = String::from_utf8_lossy(raw).into_owned();
            let (v, r, w, _pre, st) = evaluate(read, &post_bytes);
            (v, r, w, st)
        }
    };
    One { contract: contract.to_string(), verdict, reason, witnesses, stated }
}

/// The judgment proper: base contract (as read from git) against the
/// working-tree bytes. Pure: the same inputs give the same verdict,
/// reason and witnesses, which is what lets a receipt be reproduced.
#[allow(clippy::type_complexity)]
fn evaluate(read: BaseRead, post_bytes: &str) -> (&'static str, Option<&'static str>, Vec<Witness>, Option<String>, Stated) {
    match read {
            BaseRead::NoRef => ("NOT EVALUATED", Some("unreadable_base"), Vec::new(), None, Stated::default()),
            BaseRead::NoFile => ("NOT EVALUATED", Some("new_contract"), Vec::new(), None, Stated::default()),
            BaseRead::Found(bytes) => {
                let pre_text = String::from_utf8_lossy(&bytes).into_owned();
                match (serde_json::from_str::<Value>(&pre_text), serde_json::from_str::<Value>(post_bytes)) {
                    (Err(_), _) | (_, Err(_)) => ("NOT EVALUATED", Some("unparsable"), Vec::new(), Some(pre_text), Stated::default()),
                    (Ok(pre), Ok(post)) => {
                        let mut w = Vec::new();
                        ROOTS.with(|r| *r.borrow_mut() = Some([pre.clone(), post.clone()]));
                        walk("", &pre, &post, &mut w);
                        ROOTS.with(|r| *r.borrow_mut() = None);
                        let stated = take_stated(&mut w);
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
                        for x in &mut w {
                            let (c, why) = crate::counterexample::build(x, &pre, &post);
                            x.counterexample = c;
                            x.counterexample_reason = why;
                        }
                        let (v, r) = if !w.is_empty() {
                            ("VIOLATED", None)
                        } else if !stated.not_evaluated.is_empty() {
                            ("NOT EVALUATED", Some("unjudged_construct"))
                        } else {
                            ("HOLDS", None)
                        };
                        (v, r, w, Some(pre_text), stated)
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
    let (verdict, reason, witnesses, _, stated) = evaluate(read, &post_bytes);
    let inputs = ReceiptInputs {
        base_ref: &base_ref,
        base_commit,
        base_contract_blake3: base_b3,
        contract: &contract,
        post_contract_blake3: post_b3,
    };
    let rederived = receipt_body(&inputs, body_json(verdict, reason, &contract, &base_ref, &witnesses, &stated));
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
            counterexample: None,
            counterexample_reason: None,
        });
    }
    // D2 (r2): a removal breaks validation only when the object it left is
    // closed at the post ref; from an open object it is a stated note.
    let closed = is_closed(post);
    // J1 (JUDGE-R3 r0): a pattern may still admit a key that left
    // `properties`; this judge does not evaluate patterns, so it states the
    // path instead of accusing.
    let patterned = post.get("patternProperties").is_some();
    for (k, pv) in pp {
        let dotted = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
        // D4 (r2 Amendment A): local references resolved before comparing.
        let (pv_resolved, p_refs) = resolve_ref(0, pv);
        let pv = &pv_resolved;
        let q_pair = qp.get(k).map(|q| resolve_ref(1, q));
        let q_refs = q_pair.as_ref().map(|(_, r)| r.clone()).unwrap_or_default();
        let qv_resolved = q_pair.map(|(v, _)| v);
        ACTIVE.with(|a| {
            let mut a = a.borrow_mut();
            a[0].extend(p_refs.iter().cloned());
            a[1].extend(q_refs.iter().cloned());
        });
        match qv_resolved.as_ref() {
            None if closed && patterned => out.push(stated_witness(&dotted, "not_evaluated", pv)),
            None => out.push(Witness {
                path: dotted,
                kind: if closed { "removed" } else { "removed_open" },
                pre_type: type_str(pv),
                post_type: None,
                pre_line: None,
                post_line: None,
                keyword: None,
                pre_value: None,
                post_value: None,
                anchor_line: None,
                counterexample: None,
                counterexample_reason: None,
            }),
            Some(qv) if has_combinator(pv) || has_combinator(qv) => {
                // D1 (r2): a node under anyOf/oneOf/allOf/$ref is not followed,
                // so it is never accused: a changed one is stated, not judged.
                if pv != qv {
                    out.push(stated_witness(&dotted, "not_evaluated", pv));
                }
            }
            Some(qv) => {
                if type_str(pv) == "unspecified" && type_made_explicit(pv, qv) {
                    // D5 (r2 Amendment B): the type its own keywords already
                    // implied, now written down: a stated note, not a break.
                    out.push(stated_witness(&dotted, "type_explicit", qv));
                } else if type_str(pv) == "unspecified" && enum_covered(pv, qv) {
                    // D9 (JUDGE-D9 r0): the enum (or const) already limited
                    // the field to values of the new type.
                    out.push(Witness { post_type: Some(type_str(qv)), ..stated_witness(&dotted, "type_enum_covered", pv) });
                } else if type_str(pv) != type_str(qv) && widens(pv, qv) {
                    // D7 (JUDGE-WIDEN r0): every value valid before is still
                    // valid; a stated note, never a break.
                    out.push(Witness { kind: "retyped_widened", post_type: Some(type_str(qv)), ..stated_witness(&dotted, "retyped_widened", pv) });
                } else if type_str(pv) != type_str(qv) {
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
                        counterexample: None,
                        counterexample_reason: None,
                    });
                }
                constraints(&dotted, pv, qv, out);
                walk(&dotted, pv, qv, out);
            }
        }
        ACTIVE.with(|a| {
            let mut a = a.borrow_mut();
            let (n0, n1) = (a[0].len() - p_refs.len(), a[1].len() - q_refs.len());
            a[0].truncate(n0);
            a[1].truncate(n1);
        });
    }
}

thread_local! {
    /// The base and post documents of the judgment in progress, so a local
    /// `$ref` resolves against its own side (D4). Set and cleared by
    /// `evaluate`; nothing else reads it.
    static ROOTS: std::cell::RefCell<Option<[Value; 2]>> = const { std::cell::RefCell::new(None) };
    /// References being expanded along the current walk path, per side: a
    /// reference already on the path is a cycle through `properties`, and
    /// stays a `$ref` (stated by D1) instead of recursing forever.
    static ACTIVE: std::cell::RefCell<[Vec<String>; 2]> = const { std::cell::RefCell::new([Vec::new(), Vec::new()]) };
}

/// D4 (r2 Amendment A): resolve a node's local `$ref` chain against the
/// document of `side` (0 base, 1 post). Keywords beside a `$ref` override
/// the target's. At most 8 hops; a cycle, an external or missing target,
/// or a non-object target leaves the `$ref` in place, so D1 states it.
fn resolve_ref(side: usize, v: &Value) -> (Value, Vec<String>) {
    let active = ACTIVE.with(|a| a.borrow()[side].clone());
    ROOTS.with(|r| match r.borrow().as_ref() {
        Some(roots) => deref(&roots[side], v, active),
        None => (v.clone(), Vec::new()),
    })
}

fn deref(root: &Value, v: &Value, on_path: Vec<String>) -> (Value, Vec<String>) {
    let mut cur = v.clone();
    let mut seen: Vec<String> = on_path;
    let mut followed: Vec<String> = Vec::new();
    for _ in 0..8 {
        let Some(r) = cur.get("$ref").and_then(Value::as_str).map(str::to_string) else {
            return (cur, followed);
        };
        if !r.starts_with('#') || seen.contains(&r) {
            return (cur, followed);
        }
        let Some(mut merged) = root.pointer(&r[1..]).cloned() else {
            return (cur, followed);
        };
        let (Some(m), Some(o)) = (merged.as_object_mut(), cur.as_object()) else {
            return (cur, followed);
        };
        for (k, val) in o {
            if k != "$ref" {
                m.insert(k.clone(), val.clone());
            }
        }
        seen.push(r.clone());
        followed.push(r);
        cur = merged;
    }
    (cur, followed)
}

/// D1 (r2): does this node carry a construct the judge does not follow?
fn has_combinator(v: &Value) -> bool {
    ["anyOf", "oneOf", "allOf", "$ref"].iter().any(|k| v.get(*k).is_some())
}

/// D5 (r2 Amendment B): is every type `post` declares already implied by
/// the keywords `pre` carried while it declared none?
fn type_made_explicit(pre: &Value, post: &Value) -> bool {
    const IMPLIED: [(&str, &[&str]); 4] = [
        ("object", &["properties", "additionalProperties", "patternProperties", "required", "propertyNames", "minProperties", "maxProperties"]),
        ("array", &["items", "prefixItems", "contains", "minItems", "maxItems", "uniqueItems"]),
        ("string", &["minLength", "maxLength", "pattern", "format"]),
        ("number", &["minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum", "multipleOf"]),
    ];
    let implied: Vec<&str> = IMPLIED.iter().filter(|(_, kws)| kws.iter().any(|k| pre.get(*k).is_some())).map(|(t, _)| *t).collect();
    let post_type = type_str(post);
    let declared: Vec<&str> = post_type.split('|').map(|t| if t == "integer" { "number" } else { t }).collect();
    !implied.is_empty() && declared.iter().all(|t| implied.contains(t))
}

/// D7 (JUDGE-WIDEN r0): does `post` accept every type `pre` declared?
/// `integer` is contained in `number`; no `type` at post accepts all. A
/// field with no `type` at base is not a widening (D5 or VIOLATED decide).
fn widens(pre: &Value, post: &Value) -> bool {
    let (a, b) = (type_str(pre), type_str(post));
    if a == "unspecified" {
        return false;
    }
    if b == "unspecified" {
        return true;
    }
    let b: Vec<&str> = b.split('|').collect();
    a.split('|').all(|t| b.contains(&t) || (t == "integer" && b.contains(&"number")))
}

/// D9 (JUDGE-D9 r0): does every value the base enum (or const) allows
/// have a type the post declares? An integer value is also a number; a
/// whole number written 1.0 is an integer.
fn enum_covered(pre: &Value, post: &Value) -> bool {
    let values: Vec<&Value> = match (pre.get("enum").and_then(Value::as_array), pre.get("const")) {
        (Some(e), _) => e.iter().collect(),
        (None, Some(c)) => vec![c],
        _ => return false,
    };
    let declared = type_str(post);
    if declared == "unspecified" {
        return false;
    }
    let t: Vec<&str> = declared.split('|').collect();
    !values.is_empty() && values.iter().all(|v| value_has_type(v, &t))
}

/// Is `v` an instance of one of the JSON Schema types `t`?
fn value_has_type(v: &Value, t: &[&str]) -> bool {
    let ty = match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Number(n) if n.is_i64() || n.is_u64() || n.as_f64().is_some_and(|f| f.fract() == 0.0) => "integer",
        Value::Number(_) => "number",
    };
    t.contains(&ty) || (ty == "integer" && t.contains(&"number"))
}

/// D2 (r2): an object that admits no other keys.
fn is_closed(v: &Value) -> bool {
    v.get("additionalProperties") == Some(&Value::Bool(false)) || v.get("unevaluatedProperties") == Some(&Value::Bool(false))
}

/// A stated finding that is not a break (not evaluated, or a note).
fn stated_witness(path: &str, kind: &'static str, pv: &Value) -> Witness {
    Witness {
        path: path.to_string(),
        kind,
        pre_type: type_str(pv),
        post_type: None,
        pre_line: None,
        post_line: None,
        keyword: None,
        pre_value: None,
        post_value: None,
        anchor_line: None,
        counterexample: None,
        counterexample_reason: None,
    }
}

/// What the judge states without accusing: fields it did not follow (D1)
/// and removals from open objects (D2). Neither is a witness of a break.
#[derive(Default)]
pub struct Stated {
    pub not_evaluated: Vec<String>,
    pub notes: Vec<String>,
}

/// Move the stated findings out of the witness list, sorted.
fn take_stated(w: &mut Vec<Witness>) -> Stated {
    let mut st = Stated::default();
    w.retain(|x| match x.kind {
        "not_evaluated" => {
            st.not_evaluated.push(x.path.clone());
            false
        }
        "retyped_widened" => {
            st.notes.push(format!(
                "retyped (widened): {} was {}, is now {}; every value valid before is still valid.",
                x.path,
                x.pre_type,
                x.post_type.as_deref().unwrap_or("unspecified")
            ));
            false
        }
        "enum_covered" => {
            st.notes.push(format!(
                "enum added: {} now lists every value valid before ({}).",
                x.path,
                x.pre_value.as_deref().unwrap_or("")
            ));
            false
        }
        "type_enum_covered" => {
            st.notes.push(format!(
                "type made explicit: {} is now {}, which every value of its enum already is; every value valid before is still valid.",
                x.path,
                x.post_type.as_deref().unwrap_or("unspecified")
            ));
            false
        }
        "type_explicit" => {
            st.notes.push(format!(
                "type made explicit: {} is now {}, which its own keywords already implied; values of other types, accepted before, are now rejected.",
                x.path, x.pre_type
            ));
            false
        }
        "removed_open" => {
            st.notes.push(format!(
                "removed from an object that admits other keys: {} ({}); instances carrying it still validate.",
                x.path, x.pre_type
            ));
            false
        }
        _ => true,
    });
    st.not_evaluated.sort();
    st.notes.sort();
    st
}

/// D3 (r2): would this value, valid under the old enum, fail the new bound?
/// A keyword constrains only its own type, as JSON Schema defines it.
fn violates(kw: &str, v: &Value, b: f64) -> bool {
    let n = match kw {
        "minimum" | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" => v.as_f64(),
        "minLength" | "maxLength" => v.as_str().map(|s| s.chars().count() as f64),
        _ => v.as_array().map(|a| a.len() as f64),
    };
    match (kw, n) {
        (_, None) => false,
        ("minimum" | "minLength" | "minItems", Some(n)) => n < b,
        ("exclusiveMinimum", Some(n)) => n <= b,
        ("exclusiveMaximum", Some(n)) => n >= b,
        (_, Some(n)) => n > b,
    }
}

/// Names newly present in an object's `required[]` (JUDGE-SUBTYPE JS-D4).
fn newly_required(pre: &Value, post: &Value) -> Vec<String> {
    // J3 (JUDGE-R3 r0): an object requires its own `required` names and
    // those of each direct `allOf` member (members are conjunctive).
    let own = |v: &Value| -> Vec<String> {
        v.get("required")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    let names = |v: &Value| -> Vec<String> {
        let mut n = own(v);
        for m in v.get("allOf").and_then(Value::as_array).into_iter().flatten() {
            n.extend(own(m));
        }
        n
    };
    let before = names(pre);
    let mut added: Vec<String> = names(post).into_iter().filter(|n| !before.contains(n)).collect();
    added.sort();
    added.dedup();
    added
}

/// True when post's `enum` accepts less than pre's (JS-D1): a value lost,
/// or an enum added where there was none. Dropping the enum widens.
/// J2 (JUDGE-R3 r0): the values a field accepts, when they are few and
/// known: `boolean`, or `integer` bounded on both sides, at most 256.
fn finite_domain(v: &Value) -> Option<Vec<Value>> {
    let ty = type_str(v);
    if ty == "boolean" {
        return Some(vec![Value::Bool(true), Value::Bool(false)]);
    }
    if ty != "integer" {
        return None;
    }
    let f = |k: &str| v.get(k).and_then(Value::as_f64);
    let lo = [f("minimum").map(f64::ceil), f("exclusiveMinimum").map(|x| x.floor() + 1.0)].into_iter().flatten().reduce(f64::max)?;
    let hi = [f("maximum").map(f64::floor), f("exclusiveMaximum").map(|x| x.ceil() - 1.0)].into_iter().flatten().reduce(f64::min)?;
    if hi < lo || hi - lo >= 256.0 {
        return None;
    }
    Some((lo as i64..=hi as i64).map(Value::from).collect())
}

/// Numbers compared by value (an enum may write 1 as 1.0).
fn same_value(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) if a.is_number() && b.is_number() => x == y,
        _ => a == b,
    }
}

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
        counterexample: None,
        counterexample_reason: None,
    }
}

/// The JUDGE-SUBTYPE r1 validators on one property present at both refs:
/// enum narrowing, bound tightening, pattern and format changes. A schema
/// is read as a validation contract: narrowing what it accepts is VIOLATED.
fn constraints(path: &str, pv: &Value, qv: &Value, out: &mut Vec<Witness>) {
    match (pv.get("enum").is_none(), finite_domain(pv), qv.get("enum").and_then(Value::as_array)) {
        // J2 (JUDGE-R3 r0): an enum added over a finite base domain is
        // judged as an enum narrowing of that domain.
        (true, Some(dom), Some(new)) => {
            let lost = dom.iter().any(|v| !new.iter().any(|n| same_value(n, v)));
            if lost {
                let dom = Value::Array(dom);
                out.push(constraint_witness(path, "narrowed", "enum", pv, Some(&dom), qv.get("enum")));
            } else {
                let mut w = stated_witness(path, "enum_covered", pv);
                w.pre_value = Some(dom.iter().map(Value::to_string).collect::<Vec<_>>().join(", "));
                out.push(w);
            }
        }
        _ if narrowed_enum(pv, qv) => {
            out.push(constraint_witness(path, "narrowed", "enum", pv, pv.get("enum"), qv.get("enum")));
        }
        _ => {}
    }
    // D3 (r2): an enum dropped at the post ref is judged by its old values.
    let old_enum = if qv.get("enum").is_none() { pv.get("enum").and_then(Value::as_array) } else { None };
    let bounds = LOWER.iter().map(|k| (*k, true)).chain(UPPER.iter().map(|k| (*k, false)));
    for (kw, is_lower) in bounds {
        let (ra, rb) = (pv.get(kw), qv.get(kw));
        if ra == rb || rb.is_none() {
            continue; // unchanged, or the bound was removed: a loosening
        }
        let numeric_a = !matches!(ra, Some(v) if !v.is_number());
        match (ra.and_then(Value::as_f64), rb.and_then(Value::as_f64)) {
            (a, Some(b)) if numeric_a => {
                let tightened = match old_enum {
                    Some(vals) => vals.iter().any(|v| violates(kw, v, b)),
                    None => match a {
                        None => true,
                        Some(a) => (is_lower && b > a) || (!is_lower && b < a),
                    },
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
fn body_json(verdict: &str, reason: Option<&str>, contract: &str, base: &str, w: &[Witness], st: &Stated) -> Value {
    json!({
        "not_evaluated_paths": st.not_evaluated,
        "notes": st.notes,
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
            "counterexample": x.counterexample,
            "counterexample_reason": x.counterexample_reason,
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
                "removed: {} was {}{at}, from an object that admits no other keys. Instances that carry it, valid before, are now rejected; keep the field, or version the change.",
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
                        "narrowed: {} no longer accepts {}. Instances using them, valid before, are now rejected; keep the values, or version the change.",
                        x.path,
                        lost.join(", ")
                    )
                }
                None => format!(
                    "narrowed: {} now accepts only {post}. Values outside that set, valid before, are now rejected; drop the enum, or version the change.",
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
            "escalated: {} is now required. Instances that omit it, valid before, are now rejected; keep it optional, or version the change.",
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
                "retyped: {} was {}, is now {to}. Values outside {to}, valid before, are now rejected; restore the type, or version the change.",
                x.path, x.pre_type
            )
        }
    }
}

/// COUNTEREXAMPLE r0: the line under a witness, shared by text and GitHub.
fn counterexample_line(x: &Witness, base: &str) -> String {
    match (&x.counterexample, &x.counterexample_reason) {
        (Some(v), _) => format!("counterexample (valid at {base}, rejected now): {}", serde_json::to_string(v).unwrap_or_default()),
        (None, Some(r)) => format!("no counterexample built: {r}."),
        (None, None) => "no counterexample built.".to_string(),
    }
}

/// The NOT EVALUATED sentence for a reason code, shared by every format.
fn not_evaluated_sentence(reason: Option<&str>, contract: &str, base: &str) -> String {
    match reason {
        Some("new_contract") => format!(
            "NOT EVALUATED (new_contract): {contract} does not exist at {base}; nothing was promised before. The next change to this contract will be judged against this version."
        ),
        Some("unjudged_construct") => format!(
            "NOT EVALUATED (unjudged_construct): {contract} changed only where this judge does not look (anyOf, oneOf, allOf or $ref). Nothing here was judged a break or a pass; review those fields by hand."
        ),
        Some("contract_deleted") => format!(
            "NOT EVALUATED (contract_deleted): {contract} exists at {base} and this change deletes it; everything it promised is withdrawn. Restore it, or version the change."
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
fn render_text(verdict: &str, reason: Option<&str>, contract: &str, base: &str, w: &[Witness], st: &Stated) -> String {
    let mut s = String::new();
    match verdict {
        "HOLDS" => s.push_str(&format!("HOLDS: {contract} keeps every promise it made at {base}.\n")),
        "VIOLATED" => {
            let n = w.len();
            let noun = if n == 1 { "promise" } else { "promises" };
            s.push_str(&format!("VIOLATED: {contract} breaks {n} {noun} made at {base}.\n"));
            for x in w {
                s.push_str(&format!("  {}\n", witness_sentence(x, base)));
                s.push_str(&format!("    {}\n", counterexample_line(x, base)));
            }
        }
        _ => s.push_str(&format!("{}\n", not_evaluated_sentence(reason, contract, base))),
    }
    for p in &st.not_evaluated {
        s.push_str(&format!("  not evaluated: {p} uses anyOf, oneOf, allOf or $ref, which this judge does not follow.\n"));
    }
    for n in &st.notes {
        s.push_str(&format!("  noted: {n}\n"));
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
fn render_sarif(verdict: &str, reason: Option<&str>, contract: &str, base: &str, w: &[Witness], st: &Stated) -> Value {
    let results = sarif_results(contract, base, w);
    let notifications: Vec<Value> = sarif_notification(verdict, reason, contract, base).into_iter().collect();
    sarif_log(
        results,
        notifications,
        verdict != "NOT EVALUATED",
        json!({
            "unjudged_constraints": UNJUDGED,
            "not_evaluated_paths": st.not_evaluated,
            "notes": st.notes,
            "verdict": verdict,
            "base": base,
        }),
    )
}

fn sarif_results(contract: &str, base: &str, w: &[Witness]) -> Vec<Value> {
    w.iter()
        .map(|x| {
            json!({
                "ruleId": format!("invar.judge.{}", x.kind),
                "level": "error",
                "message": { "text": witness_sentence(x, base) },
                "properties": { "counterexample": x.counterexample },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": contract },
                        "region": { "startLine": location_line(x) },
                    }
                }],
            })
        })
        .collect()
}

fn sarif_notification(verdict: &str, reason: Option<&str>, contract: &str, base: &str) -> Option<Value> {
    match verdict {
        "HOLDS" | "VIOLATED" => None,
        _ => Some(json!({
            "level": "warning",
            "descriptor": { "id": reason.unwrap_or("unparsable") },
            "message": { "text": not_evaluated_sentence(reason, contract, base) },
        })),
    }
}

fn sarif_log(results: Vec<Value>, notifications: Vec<Value>, success: bool, properties: Value) -> Value {
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
                "executionSuccessful": success,
                "toolExecutionNotifications": notifications,
            }],
            "results": results,
            "properties": properties,
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
                    "::error file={file},line={},title={}::{}%0A{}\n",
                    location_line(x),
                    gh_prop(&format!("invar.judge.{}", x.kind)),
                    gh_data(&witness_sentence(x, base)),
                    gh_data(&counterexample_line(x, base))
                )
            })
            .collect(),
        _ => format!(
            "::warning file={file},title=invar.judge.not_evaluated::{}\n",
            gh_data(&not_evaluated_sentence(reason, contract, base))
        ),
    }
}

/// JUDGE-CONTRACTS r0 (docs/falsification/JUDGE-CONTRACTS-SPEC.md): judge
/// every contract a change touches, in one pass. Selection is git's own:
/// pathspec globs (M-D1), files that differ between the base tree and the
/// working tree plus untracked files git does not ignore (M-D2). A glob
/// set matching nothing at either side is an error (M-D4), never a pass.
pub fn run_many(root: &Path, globs: &[String], base: &str, format: &str) -> Result<i32> {
    let specs: Vec<String> = globs.iter().map(|g| format!(":(glob){g}")).collect();
    let base_ok = git_ok(root, &["rev-parse", "--verify", "--quiet", &format!("{base}^{{commit}}")])?;

    // Every file the globs match, at the base and in the working tree.
    let mut matched: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if base_ok {
        // ls-tree has no glob pathspec magic; a diff from the empty tree
        // lists the base tree through the same matcher as every other step.
        let empty = empty_tree(root)?;
        matched.extend(git_paths(root, &["diff", "-z", "--name-only", "--no-renames", &empty, base, "--"], &specs)?);
    }
    let here = git_paths(root, &["ls-files", "-z", "--cached", "--others", "--exclude-standard", "--"], &specs)?;
    matched.extend(here.iter().filter(|p| root.join(p).is_file()).cloned());
    if matched.is_empty() {
        anyhow::bail!(
            "no file matches {} at {base} or in the working tree; check the glob (a glob that matches nothing is never a pass)",
            globs.join(" ")
        );
    }

    // The changed ones (M-D2). Without a readable base nothing can be
    // diffed: every matching contract is judged, and each says why.
    let mut changed: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if base_ok {
        changed.extend(git_paths(root, &["diff", "-z", "--name-only", "--no-renames", base, "--"], &specs)?);
        changed.extend(git_paths(root, &["ls-files", "-z", "--others", "--exclude-standard", "--"], &specs)?);
    } else {
        changed.extend(matched.iter().cloned());
    }
    let changed: Vec<String> = changed.into_iter().filter(|p| matched.contains(p)).collect();
    let unchanged = matched.len() - changed.len();

    let mut ones = Vec::new();
    for c in &changed {
        let post = std::fs::read(root.join(c)).ok();
        let read = base_bytes(root, base, c)?;
        ones.push(judge_one(c, read, post.as_deref()));
    }

    let count = |f: &dyn Fn(&One) -> bool| ones.iter().filter(|o| f(o)).count();
    let violated = count(&|o| o.verdict == "VIOLATED");
    let holds = count(&|o| o.verdict == "HOLDS");
    let not_evaluated = count(&|o| o.verdict == "NOT EVALUATED");
    let new = count(&|o| o.reason == Some("new_contract"));

    match format {
        "json" => {
            let body = json!({
                "base": base,
                "globs": globs,
                "contracts": ones.iter().map(|o| body_json(o.verdict, o.reason, &o.contract, base, &o.witnesses, &o.stated)).collect::<Vec<_>>(),
                "summary": { "violated": violated, "holds": holds, "not_evaluated": not_evaluated, "new": new, "unchanged": unchanged },
            });
            println!("{}", serde_json::to_string_pretty(&body).unwrap_or_default());
        }
        "sarif" => {
            let results = ones.iter().flat_map(|o| sarif_results(&o.contract, base, &o.witnesses)).collect();
            let notes: Vec<Value> = ones.iter().filter_map(|o| sarif_notification(o.verdict, o.reason, &o.contract, base)).collect();
            let props = json!({
                "unjudged_constraints": UNJUDGED,
                "base": base,
                "globs": globs,
                "unchanged": unchanged,
                "contracts": ones.iter().map(|o| json!({
                    "contract": o.contract,
                    "verdict": o.verdict,
                    "reason": o.reason,
                    "not_evaluated_paths": o.stated.not_evaluated,
                    "notes": o.stated.notes,
                })).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&sarif_log(results, notes, not_evaluated == 0, props)).unwrap_or_default());
        }
        "github" => {
            for o in &ones {
                print!("{}", render_github(o.verdict, o.reason, &o.contract, base, &o.witnesses));
            }
        }
        _ => {
            for o in &ones {
                print!("{}", render_text(o.verdict, o.reason, &o.contract, base, &o.witnesses, &o.stated));
            }
            println!(
                "judged {} changed contracts under {} at {base}: {violated} VIOLATED, {holds} HOLDS, {not_evaluated} NOT EVALUATED ({new} new); {unchanged} unchanged, not judged.",
                ones.len(),
                globs.join(" ")
            );
        }
    }

    // M-D5, with Amendment A: the worst verdict decides; a new contract
    // promised nothing before and never moves the exit code.
    Ok(if violated > 0 {
        1
    } else if ones.iter().any(|o| o.verdict == "NOT EVALUATED" && o.reason != Some("new_contract")) {
        4
    } else {
        0
    })
}

/// The empty tree's id in this repository's hash (SHA-1 or SHA-256).
fn empty_tree(root: &Path) -> Result<String> {
    use std::io::Write;
    let mut child = Command::new("git")
        .args(["hash-object", "-t", "tree", "--stdin"])
        .current_dir(root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("running git hash-object")?;
    if let Some(mut i) = child.stdin.take() {
        i.write_all(b"").context("writing to git hash-object")?;
    }
    let o = child.wait_with_output().context("running git hash-object")?;
    let id = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if !o.status.success() || id.is_empty() {
        anyhow::bail!("git hash-object could not name the empty tree");
    }
    Ok(id)
}

fn git_ok(root: &Path, args: &[&str]) -> Result<bool> {
    let o = Command::new("git").args(args).current_dir(root).output().context("running git")?;
    Ok(o.status.success())
}

/// Run git with NUL-separated path output; any failure is an error, never
/// an empty list that would read as "nothing changed".
fn git_paths(root: &Path, args: &[&str], specs: &[String]) -> Result<Vec<String>> {
    let o = Command::new("git").args(args).args(specs).current_dir(root).output().context("running git")?;
    if !o.status.success() {
        anyhow::bail!("git {} failed: {}", args.first().unwrap_or(&""), String::from_utf8_lossy(&o.stderr).trim());
    }
    Ok(o.stdout.split(|b| *b == 0).filter(|p| !p.is_empty()).map(|p| String::from_utf8_lossy(p).into_owned()).collect())
}
