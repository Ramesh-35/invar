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
const UNJUDGED: [&str; 5] = ["$ref", "allOf", "anyOf", "oneOf", "tuple items"];

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
        "github" => print!("{}", render_github(verdict, reason, contract, base, &witnesses, &stated)),
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
    let One { verdict, reason, witnesses, stated, .. } = judge_pair(base_path, head_path)?;
    let (c, b) = (head_path, base_path);
    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&body_json(verdict, reason, c, b, &witnesses, &stated)).unwrap_or_default()),
        "sarif" => println!("{}", serde_json::to_string_pretty(&render_sarif(verdict, reason, c, b, &witnesses, &stated)).unwrap_or_default()),
        "github" => print!("{}", render_github(verdict, reason, c, b, &witnesses, &stated)),
        _ => print!("{}", render_text(verdict, reason, c, b, &witnesses, &stated)),
    }
    Ok(match verdict {
        "HOLDS" => 0,
        "VIOLATED" => 1,
        _ => 4,
    })
}

/// WITNESS r0 (docs/falsification/WITNESS-SPEC.md): judge BASE and HEAD as
/// `invar judge` does and write each witness that carries a counterexample
/// as one SchemaStore test file. A generator: exit 0 when it ran, 3 on an
/// unreadable file, a NAME that is not one path component, or an
/// unwritable OUT; nothing is written on 3.
pub fn run_witness(base_path: &str, head_path: &str, name: &str, out: &str, positive: bool) -> Result<i32> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        anyhow::bail!("--name must be one path component (the schema's directory under src/test), not {name:?}");
    }
    let One { verdict, witnesses, .. } = judge_pair(base_path, head_path)?;
    let kind_dir = if positive { "src/test" } else { "src/negative_test" };
    let dir = Path::new(out).join(kind_dir).join(name);
    let mut planned: Vec<(String, Vec<u8>, String)> = Vec::new();
    let mut without: Vec<String> = Vec::new();
    for w in &witnesses {
        match &w.counterexample {
            Some(doc) => {
                let n = planned.len() + 1;
                let file = format!("invar-{n:02}-{}.json", witness_slug(&w.path));
                let mut bytes = serde_json::to_vec_pretty(doc).unwrap_or_default();
                bytes.push(b'\n');
                planned.push((file, bytes, format!("{} {}", w.kind, w.path)));
            }
            None => without.push(format!("{} {}: {}", w.kind, w.path, w.counterexample_reason.as_deref().unwrap_or("no document"))),
        }
    }
    if !planned.is_empty() {
        std::fs::create_dir_all(&dir).map_err(|e| anyhow::anyhow!("creating {}: {e}", dir.display()))?;
    }
    for (file, bytes, what) in &planned {
        let path = dir.join(file);
        std::fs::write(&path, bytes).map_err(|e| anyhow::anyhow!("writing {}: {e}", path.display()))?;
        println!("wrote {kind_dir}/{name}/{file}: {what}, valid at {base_path}, rejected at {head_path}");
    }
    for line in &without {
        println!("no document for {line}");
    }
    println!("verdict {verdict}: {} document(s) written, {} witness(es) without one", planned.len(), without.len());
    Ok(0)
}

/// The file-name slug of a witness path: `[]` reads `items`, every run of
/// characters outside `[A-Za-z0-9]` becomes one `-`, trimmed, at most 60.
fn witness_slug(path: &str) -> String {
    let readable = path.replace("[]", ".items");
    let mut slug = String::new();
    let mut dash = false;
    for c in readable.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            dash = false;
        } else if !dash && !slug.is_empty() {
            slug.push('-');
            dash = true;
        }
    }
    let slug = slug.trim_end_matches('-').to_string();
    let mut slug: String = slug.chars().take(60).collect();
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() { "root".to_string() } else { slug }
}

/// JUDGE-FILEPAIR r0: two ad-hoc files judged directly. Files that do not
/// parse are operator errors (exit 3), unlike a repository contract, whose
/// unparsable bytes are a change under judgment (NOT EVALUATED, exit 4).
fn judge_pair(base_path: &str, head_path: &str) -> Result<One> {
    let base_raw = std::fs::read(base_path).map_err(|e| anyhow::anyhow!("reading {base_path}: {e}"))?;
    let head_raw = std::fs::read(head_path).map_err(|e| anyhow::anyhow!("reading {head_path}: {e}"))?;
    serde_json::from_slice::<Value>(&base_raw).map_err(|e| anyhow::anyhow!("{base_path} is not JSON: {e}"))?;
    serde_json::from_slice::<Value>(&head_raw).map_err(|e| anyhow::anyhow!("{head_path} is not JSON: {e}"))?;
    Ok(judge_one(head_path, BaseRead::Found(base_raw), Some(&head_raw)))
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
                        reach::begin();
                        walk("", &pre, &post, &mut w);
                        let (unchecked, notes) = reach::finish(&pre, &post);
                        ROOTS.with(|r| *r.borrow_mut() = None);
                        let mut stated = take_stated(&mut w);
                        stated.unchecked = unchecked;
                        // JUDGE-DRAFT r0 and JUDGE-CANON r0: what was covered is stated, never silent.
                        stated.notes.extend(notes);
                        w.sort_by(|a, b| (a.path.as_str(), a.kind, a.keyword).cmp(&(b.path.as_str(), b.kind, b.keyword)));
                        for x in &mut w {
                            // JUDGE-ITEMS r0: an element (`r[]`) sits on its array's line.
                            let last = x.path.rsplit('.').next().unwrap_or(&x.path).trim_end_matches("[]").to_string();
                            x.pre_line = find_line(&pre_text, &last);
                            if x.kind != "removed" {
                                x.post_line = find_line(post_bytes, &last);
                            } else {
                                let parent = x.path.rsplit('.').nth(1).unwrap_or("properties").trim_end_matches("[]");
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
                        } else if !stated.unchecked.is_empty() {
                            // JUDGE-REACH r0: HOLDS asserts every change was judged.
                            ("NOT EVALUATED", Some("unchecked_change"))
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
    // JUDGE-REACH r0: what this node's judgment covers, by JSON Pointer on
    // each side: `required` (with the direct allOf members' J3 reads) and
    // the `properties` key set, whose additions are safe only when the base
    // node admitted no other key.
    let (pc, qc) = reach::current();
    let adds_ok = is_closed(pre) && pre.get("patternProperties").is_none();
    for (side, chain) in [(0, &pc), (1, &qc)] {
        reach::cover_kw(side, reach::owner(side, chain, "required"), "required");
        let all_of = reach::owner(side, chain, "allOf");
        for i in 0..reach::len_at(side, &format!("{all_of}/allOf")) {
            reach::cover_kw(side, format!("{all_of}/allOf/{i}"), "required");
        }
        reach::cover_props(side, reach::owner(side, chain, "properties"), adds_ok);
    }
    let p_props = reach::owner(0, &pc, "properties");
    let q_props = reach::owner(1, &qc, "properties");
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
        let site = |props: &str| format!("{props}/properties/{}", reach::escape(k));
        judge_child(&dotted, pv, qp.get(k), site(&p_props), site(&q_props), closed, patterned, out);
    }
}

/// One child of a judged node, a property or (JUDGE-ITEMS r0) an array's
/// element: `p_site` and `q_site` are its JSON Pointers on each side
/// before its local `$ref` chain is followed; `qv_raw` None means the
/// head dropped it, judged by `closed` and `patterned` (D2, J1).
#[allow(clippy::too_many_arguments)]
fn judge_child(dotted: &str, pv_raw: &Value, qv_raw: Option<&Value>, p_site: String, q_site: String, closed: bool, patterned: bool, out: &mut Vec<Witness>) {
    let dotted = dotted.to_string();
    // D4 (r2 Amendment A): local references resolved before comparing.
    let (pv_resolved, p_refs) = resolve_ref(0, pv_raw);
    let pv = &pv_resolved;
    let q_pair = qv_raw.map(|q| resolve_ref(1, q));
    let q_refs = q_pair.as_ref().map(|(_, r)| r.clone()).unwrap_or_default();
    let qv_resolved = q_pair.map(|(v, _)| v);
    // The pointers whose merged keywords form this child, site first.
    let child = |site: String, refs: &[String]| {
        let mut c = vec![site];
        c.extend(refs.iter().map(|r| r[1..].to_string()));
        c
    };
    let (p_chain, q_chain) = (child(p_site, &p_refs), child(q_site, &q_refs));
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
            // JUDGE-CANON r0: the same meaning in another spelling is no
            // witness; `reach::finish` notes the rewritten node.
            if pv != qv && !reach::equivalent(pv, qv) {
                out.push(stated_witness(&dotted, "not_evaluated", pv));
            }
            // JUDGE-REACH r0: D1 compared these nodes whole.
            reach::cover_whole(0, &p_chain);
            reach::cover_whole(1, &q_chain);
        }
        Some(qv) => {
            // JUDGE-REACH r0: the keywords judged at a property node.
            for (side, chain) in [(0, &p_chain), (1, &q_chain)] {
                for kw in ["type", "enum", "const", "pattern", "format"].iter().chain(LOWER.iter()).chain(UPPER.iter()) {
                    reach::cover_kw(side, reach::owner(side, chain, kw), kw);
                }
                reach::cover_refs(side, chain);
            }
            // JUDGE-D5 r1: the D5 branch of r2 Amendment B ("type made
            // explicit", a note) is withdrawn. A type written where none was
            // rejects every other type a validator accepted there before,
            // and is judged below by the type rule like any other retype.
            if type_str(pv) == "unspecified" && enum_covered(pv, qv) {
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
            reach::push(p_chain.clone(), q_chain.clone());
            walk(&dotted, pv, qv, out);
            reach::pop();
            walk_items(&dotted, pv, qv, &p_chain, &q_chain, out);
        }
    }
    ACTIVE.with(|a| {
        let mut a = a.borrow_mut();
        let (n0, n1) = (a[0].len() - p_refs.len(), a[1].len() - q_refs.len());
        a[0].truncate(n0);
        a[1].truncate(n1);
    });
}

/// JUDGE-ITEMS r0: the element schema of a judged array node, judged as
/// its child `[]` by the property-node rules. An object or absent `items`
/// on each side (absent reads as `{}`), no `prefixItems`, and a type that
/// admits arrays; anything else stays as it was (unchecked, by pointer).
fn walk_items(dotted: &str, pv: &Value, qv: &Value, p_chain: &[String], q_chain: &[String], out: &mut Vec<Witness>) {
    let judged = |v: &Value| matches!(v.get("items"), None | Some(Value::Object(_))) && v.get("prefixItems").is_none() && admits_array(v);
    if !judged(pv) || !judged(qv) || (pv.get("items").is_none() && qv.get("items").is_none()) {
        return;
    }
    let (p_items, q_items) = (reach::owner(0, p_chain, "items"), reach::owner(1, q_chain, "items"));
    reach::cover_kw(0, p_items.clone(), "items");
    reach::cover_kw(1, q_items.clone(), "items");
    let empty = Value::Object(serde_json::Map::new());
    let (pi, qi) = (pv.get("items").unwrap_or(&empty), qv.get("items").unwrap_or(&empty));
    judge_child(&format!("{dotted}[]"), pi, Some(qi), format!("{p_items}/items"), format!("{q_items}/items"), false, false, out);
}

/// Does this node's `type` admit arrays (no `type` admits every type)?
fn admits_array(v: &Value) -> bool {
    match v.get("type") {
        None => true,
        Some(Value::String(t)) => t == "array",
        Some(Value::Array(ts)) => ts.iter().any(|t| t == "array"),
        Some(_) => false,
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
    /// JUDGE-REACH r0: JSON Pointers of the schemas that changed in a way
    /// the judge does not check, sorted, an ancestor standing for its
    /// descendants.
    pub unchecked: Vec<String>,
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

/// JUDGE-CONST r0: the values a node's `enum` allows, a `const` read as an
/// enum of one value. Amendment A: values the BASE node's own `type` never
/// admitted are left out on both sides (`{"type":"string","enum":["a",null]}`
/// allows exactly "a", so losing null is no narrowing); what the head's
/// type excludes is the type rule's verdict, not the enum rule's.
fn enum_values(v: &Value, admitted_by: &Value) -> Option<Vec<Value>> {
    let vals: Vec<Value> = match (v.get("enum").and_then(Value::as_array), v.get("const")) {
        (Some(e), _) => e.clone(),
        (None, Some(c)) => vec![c.clone()],
        _ => return None,
    };
    let types: Vec<&str> = match admitted_by.get("type") {
        Some(Value::String(t)) => vec![t.as_str()],
        Some(Value::Array(ts)) => ts.iter().filter_map(Value::as_str).collect(),
        _ => return Some(vals),
    };
    Some(vals.into_iter().filter(|x| reach::value_types(x).iter().any(|t| types.contains(t))).collect())
}

fn narrowed_enum(pv: &Value, qv: &Value) -> bool {
    match (enum_values(pv, pv), enum_values(qv, pv)) {
        (_, None) => false,
        (None, Some(_)) => true,
        (Some(a), Some(b)) => a.iter().any(|x| !b.iter().any(|y| same_value(x, y))),
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
    let q_enum = enum_values(qv, pv);
    let p_enum = enum_values(pv, pv);
    match (p_enum.is_none(), finite_domain(pv), q_enum.as_ref()) {
        // J2 (JUDGE-R3 r0): an enum added over a finite base domain is
        // judged as an enum narrowing of that domain.
        (true, Some(dom), Some(new)) => {
            let lost = dom.iter().any(|v| !new.iter().any(|n| same_value(n, v)));
            if lost {
                let dom = Value::Array(dom);
                out.push(constraint_witness(path, "narrowed", "enum", pv, Some(&dom), qv.get("enum").or(qv.get("const"))));
            } else {
                let mut w = stated_witness(path, "enum_covered", pv);
                w.pre_value = Some(dom.iter().map(Value::to_string).collect::<Vec<_>>().join(", "));
                out.push(w);
            }
        }
        _ if narrowed_enum(pv, qv) => {
            let (a, b) = (p_enum.clone().map(Value::Array), q_enum.clone().map(Value::Array));
            out.push(constraint_witness(path, "narrowed", "enum", pv, a.as_ref(), b.as_ref()));
        }
        _ => {}
    }
    // D3 (r2): an enum dropped at the post ref is judged by its old values.
    let old_enum_owned = if q_enum.is_none() { p_enum.clone() } else { None };
    let old_enum = old_enum_owned.as_ref();
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
        "unchecked": st.unchecked,
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
/// JUDGE-REACH r0: the one sentence every surface carries for a schema that
/// changed in a way the judge does not check. The root pointer reads as
/// the document root.
fn unchecked_sentence(pointer: &str) -> String {
    if pointer.is_empty() {
        "not evaluated: the document root changed in a way this judge does not check.".to_string()
    } else {
        format!("not evaluated: the schema at {pointer} changed in a way this judge does not check.")
    }
}

fn not_evaluated_sentence(reason: Option<&str>, contract: &str, base: &str) -> String {
    match reason {
        Some("new_contract") => format!(
            "NOT EVALUATED (new_contract): {contract} does not exist at {base}; nothing was promised before. The next change to this contract will be judged against this version."
        ),
        Some("unjudged_construct") => format!(
            "NOT EVALUATED (unjudged_construct): {contract} changed only where this judge does not look (anyOf, oneOf, allOf or $ref). Nothing here was judged a break or a pass; review those fields by hand."
        ),
        Some("unchecked_change") => format!(
            "NOT EVALUATED (unchecked_change): {contract} changed in a way this judge does not check. Nothing there was judged a break or a pass; review those schemas by hand."
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
    for p in &st.unchecked {
        s.push_str(&format!("  {}\n", unchecked_sentence(p)));
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
            "unchecked": st.unchecked,
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
fn render_github(verdict: &str, reason: Option<&str>, contract: &str, base: &str, w: &[Witness], st: &Stated) -> String {
    let file = gh_prop(contract);
    // JUDGE-REACH r0: every unchecked schema is its own warning, on every
    // verdict, so a proven break never hides a change nobody checked.
    let unchecked: String = st
        .unchecked
        .iter()
        .map(|p| format!("::warning file={file},title=invar.judge.unchecked::{}\n", gh_data(&unchecked_sentence(p))))
        .collect();
    let main = match verdict {
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
    };
    main + &unchecked
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
                    "unchecked": o.stated.unchecked,
                    "notes": o.stated.notes,
                })).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&sarif_log(results, notes, not_evaluated == 0, props)).unwrap_or_default());
        }
        "github" => {
            for o in &ones {
                print!("{}", render_github(o.verdict, o.reason, &o.contract, base, &o.witnesses, &o.stated));
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

/// JUDGE-REACH r0 (docs/falsification/JUDGE-REACH-SPEC.md, Amendment A):
/// HOLDS asserts every change was judged. The walk records what it covers,
/// by JSON Pointer on each side; after the walk, the two documents are
/// diffed schema-aware, and every difference the walk did not cover is
/// reported as the shallowest schema node that changed.
mod reach {
    use super::ROOTS;
    use serde_json::Value;
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Default)]
    struct Cover {
        /// (side, node pointer, keyword) the judge read and judged.
        kw: BTreeSet<(usize, String, String)>,
        /// (side, node pointer whose `properties` map was judged, additions safe).
        props: Vec<(usize, String, bool)>,
        /// (side, node pointer) compared whole by D1.
        whole: Vec<(usize, String)>,
        /// The walk's current node, as pointer chains (site first) per side.
        stack: Vec<(Vec<String>, Vec<String>)>,
        /// JUDGE-DRAFT r0: the root's draft migration is covered.
        draft: bool,
        /// JUDGE-CANON r0 and r1: pointers covered by a rewrite, and whether
        /// it was r0's type-list rewrite (else r1's literals).
        canon: BTreeMap<String, bool>,
    }

    thread_local! {
        static COVER: RefCell<Cover> = RefCell::new(Cover::default());
    }

    pub(super) fn begin() {
        COVER.with(|c| {
            *c.borrow_mut() = Cover::default();
            c.borrow_mut().stack.push((vec![String::new()], vec![String::new()]));
        });
    }

    pub(super) fn current() -> (Vec<String>, Vec<String>) {
        COVER.with(|c| c.borrow().stack.last().cloned().unwrap_or_default())
    }

    pub(super) fn push(p: Vec<String>, q: Vec<String>) {
        COVER.with(|c| c.borrow_mut().stack.push((p, q)));
    }

    pub(super) fn pop() {
        COVER.with(|c| {
            c.borrow_mut().stack.pop();
        });
    }

    /// RFC 6901 escaping of one reference token.
    pub(super) fn escape(k: &str) -> String {
        k.replace('~', "~0").replace('/', "~1")
    }

    fn node(side: usize, ptr: &str) -> Option<Value> {
        ROOTS.with(|r| r.borrow().as_ref().and_then(|roots| roots[side].pointer(ptr).cloned()))
    }

    /// The pointer whose node supplies `kw` to the merged node `chain`
    /// (the site's keywords override its targets'); the site when none does.
    pub(super) fn owner(side: usize, chain: &[String], kw: &str) -> String {
        for p in chain {
            if node(side, p).is_some_and(|n| n.get(kw).is_some()) {
                return p.clone();
            }
        }
        chain.first().cloned().unwrap_or_default()
    }

    pub(super) fn len_at(side: usize, ptr: &str) -> usize {
        node(side, ptr).and_then(|v| v.as_array().map(Vec::len)).unwrap_or(0)
    }

    pub(super) fn cover_kw(side: usize, ptr: String, kw: &str) {
        COVER.with(|c| {
            c.borrow_mut().kw.insert((side, ptr, kw.to_string()));
        });
    }

    pub(super) fn cover_props(side: usize, ptr: String, adds_ok: bool) {
        COVER.with(|c| c.borrow_mut().props.push((side, ptr, adds_ok)));
    }

    pub(super) fn cover_whole(side: usize, chain: &[String]) {
        COVER.with(|c| {
            let mut c = c.borrow_mut();
            for p in chain {
                c.whole.push((side, p.clone()));
            }
        });
    }

    /// Every `$ref` along a resolved chain was followed and judged.
    pub(super) fn cover_refs(side: usize, chain: &[String]) {
        for p in chain {
            if node(side, p).is_some_and(|n| n.get("$ref").is_some()) {
                cover_kw(side, p.clone(), "$ref");
            }
        }
    }

    /// Keywords no validator asserts. Everything not in KNOWN is also
    /// ignored: validators ignore keywords no draft defines.
    const ANNOTATIONS: [&str; 8] = ["title", "description", "$comment", "examples", "default", "deprecated", "readOnly", "writeOnly"];
    const KNOWN: [&str; 52] = [
        "$schema", "$id", "id", "$ref", "$anchor", "$dynamicRef", "$dynamicAnchor", "$recursiveRef", "$recursiveAnchor", "$vocabulary",
        "definitions", "$defs", "type", "enum", "const", "multipleOf", "maximum", "exclusiveMaximum", "minimum", "exclusiveMinimum",
        "maxLength", "minLength", "pattern", "maxItems", "minItems", "uniqueItems", "maxContains", "minContains", "maxProperties",
        "minProperties", "required", "dependentRequired", "properties", "patternProperties", "additionalProperties",
        "unevaluatedProperties", "propertyNames", "dependencies", "dependentSchemas", "items", "prefixItems", "additionalItems",
        "unevaluatedItems", "contains", "allOf", "anyOf", "oneOf", "not", "if", "then", "else", "format",
    ];
    /// Keywords whose value maps names to schemas: under them, keys are names.
    const NAME_MAPS: [&str; 5] = ["properties", "patternProperties", "definitions", "$defs", "dependentSchemas"];
    const SCHEMA_VALUED: [&str; 11] = ["additionalProperties", "unevaluatedProperties", "propertyNames", "contains", "not", "if", "then", "else", "additionalItems", "unevaluatedItems", "items"];
    const SCHEMA_ARRAYS: [&str; 5] = ["allOf", "anyOf", "oneOf", "prefixItems", "items"];

    /// One difference: a node's own keyword (`kw`), or the node itself
    /// (`kw` None: present on one side only, or not an object).
    struct Diff {
        ptr: String,
        kw: Option<String>,
        base: Option<Value>,
        head: Option<Value>,
        /// The keyword whose map or array holds this node, when it is a node.
        via: Option<String>,
    }

    /// JUDGE-CANON r0: keywords that apply only to instances of one type.
    const TYPED_STRING: [&str; 6] = ["minLength", "maxLength", "pattern", "format", "contentMediaType", "contentEncoding"];
    const TYPED_NUMBER: [&str; 5] = ["minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum", "multipleOf"];
    const TYPED_ARRAY: [&str; 7] = ["items", "minItems", "maxItems", "uniqueItems", "additionalItems", "contains", "prefixItems"];
    const TYPED_OBJECT: [&str; 11] = ["properties", "required", "additionalProperties", "patternProperties", "minProperties", "maxProperties", "dependencies", "dependentRequired", "dependentSchemas", "propertyNames", "unevaluatedProperties"];

    fn typed_for(t: &str) -> &'static [&'static str] {
        match t {
            "string" => &TYPED_STRING,
            "number" | "integer" => &TYPED_NUMBER,
            "array" => &TYPED_ARRAY,
            "object" => &TYPED_OBJECT,
            _ => &[],
        }
    }

    fn is_typed(k: &str) -> bool {
        TYPED_STRING.contains(&k) || TYPED_NUMBER.contains(&k) || TYPED_ARRAY.contains(&k) || TYPED_OBJECT.contains(&k)
    }

    /// JUDGE-CANON r0: the normal form. Annotations and keywords the judge
    /// does not know are set aside (the diff ignores them too); a node whose
    /// `type` is a list of two or more names, with no `anyOf` of its own,
    /// becomes the `anyOf` of one node per name, each typed keyword following
    /// its type and every other keyword shared beside the `anyOf`.
    pub(super) fn normal(v: &Value) -> Value {
        form(v, true)
    }

    /// JUDGE-CANON r1: the JSON types a literal value has (an integral
    /// number is both `integer` and `number`).
    pub(super) fn value_types(v: &Value) -> &'static [&'static str] {
        match v {
            Value::Null => &["null"],
            Value::Bool(_) => &["boolean"],
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() || n.as_f64().is_some_and(|f| f.fract() == 0.0) {
                    &["integer", "number"]
                } else {
                    &["number"]
                }
            }
            Value::String(_) => &["string"],
            Value::Array(_) => &["array"],
            Value::Object(_) => &["object"],
        }
    }

    /// Canonical text of a value, for sorting and sameness (serde_json
    /// keeps object keys sorted).
    fn canon_key(v: &Value) -> String {
        serde_json::to_string(v).unwrap_or_default()
    }

    /// A node that is exactly `{"enum": [...]}`: its values.
    fn bare_enum(v: &Value) -> Option<&Vec<Value>> {
        match v {
            Value::Object(m) if m.len() == 1 => m.get("enum").and_then(Value::as_array),
            _ => None,
        }
    }

    /// A node that is exactly `{"type": "<name>"}`: the name.
    fn bare_type(v: &Value) -> Option<&str> {
        match v {
            Value::Object(m) if m.len() == 1 => m.get("type").and_then(Value::as_str),
            _ => None,
        }
    }

    fn sorted_distinct(values: Vec<Value>) -> Vec<Value> {
        let mut by_key: std::collections::BTreeMap<String, Value> = std::collections::BTreeMap::new();
        for v in values {
            by_key.entry(canon_key(&v)).or_insert(v);
        }
        by_key.into_values().collect()
    }

    /// JUDGE-CANON r1 steps 1 to 3 at one node whose children are already in
    /// normal form. Ok(node) when the node became a bare enum (nothing more
    /// to do), Err(kept) to continue with r0's type-list rewrite.
    fn literals(mut kept: serde_json::Map<String, Value>) -> Result<Value, serde_json::Map<String, Value>> {
        let bare = |vals: Vec<Value>| Ok(Value::Object(std::iter::once(("enum".to_string(), Value::Array(vals))).collect()));
        // Step 1: a bare boolean or null type names its values.
        if kept.len() == 1 {
            match kept.get("type").and_then(Value::as_str) {
                Some("boolean") => return bare(vec![Value::Bool(false), Value::Bool(true)]),
                Some("null") => return bare(vec![Value::Null]),
                _ => {}
            }
        }
        // Step 1: const is an enum of one value.
        if let Some(c) = kept.remove("const") {
            let vals = match kept.get("enum").and_then(Value::as_array) {
                Some(e) => e.iter().filter(|v| canon_key(v) == canon_key(&c)).cloned().collect(),
                None => vec![c],
            };
            kept.insert("enum".to_string(), Value::Array(vals));
        }
        // Step 2: a type constraint beside the enum is folded into it.
        if let Some(Value::Array(vals)) = kept.get("enum").cloned() {
            // The constraint beside the enum: type names, and (from a branch
            // step 1 already turned into an enum) literal values.
            let mut types: Vec<String> = Vec::new();
            let mut lits: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            let mut drop: Option<&str> = None;
            match kept.get("type") {
                Some(Value::String(t)) => {
                    types.push(t.clone());
                    drop = Some("type");
                }
                Some(Value::Array(ts)) if ts.iter().all(Value::is_string) => {
                    types.extend(ts.iter().filter_map(Value::as_str).map(str::to_string));
                    drop = Some("type");
                }
                _ => {
                    for kw in ["anyOf", "oneOf"] {
                        if let Some(Value::Array(bs)) = kept.get(kw) {
                            if !bs.is_empty() && bs.iter().all(|b| bare_type(b).is_some() || bare_enum(b).is_some()) {
                                for b in bs {
                                    if let Some(t) = bare_type(b) {
                                        types.push(t.to_string());
                                    } else if let Some(e) = bare_enum(b) {
                                        lits.extend(e.iter().map(canon_key));
                                    }
                                }
                                drop = Some(kw);
                                break;
                            }
                        }
                    }
                }
            }
            let vals = match drop {
                Some(kw) => {
                    kept.remove(kw);
                    vals.into_iter()
                        .filter(|v| value_types(v).iter().any(|t| types.iter().any(|u| u == t)) || lits.contains(&canon_key(v)))
                        .collect()
                }
                None => vals,
            };
            kept.insert("enum".to_string(), Value::Array(sorted_distinct(vals)));
        }
        // Step 3: an anyOf of enums is their union; a oneOf only when disjoint.
        if kept.len() == 1 {
            for kw in ["anyOf", "oneOf"] {
                if let Some(Value::Array(bs)) = kept.get(kw) {
                    if !bs.is_empty() && bs.iter().all(|b| bare_enum(b).is_some()) {
                        let sets: Vec<std::collections::BTreeSet<String>> = bs.iter().map(|b| bare_enum(b).unwrap().iter().map(canon_key).collect()).collect();
                        let disjoint = (0..sets.len()).all(|i| (i + 1..sets.len()).all(|j| sets[i].is_disjoint(&sets[j])));
                        if kw == "anyOf" || disjoint {
                            let all: Vec<Value> = bs.iter().flat_map(|b| bare_enum(b).unwrap().iter().cloned()).collect();
                            return bare(sorted_distinct(all));
                        }
                    }
                }
            }
        }
        Err(kept)
    }

    /// The node with annotations and unknown keywords set aside, and with
    /// its type lists and literals rewritten when `rewrite` is set. Subschemas are found
    /// where `diff` finds them: under name maps, schema-valued keywords and
    /// schema arrays; every other keyword's value is kept as written.
    fn form(v: &Value, rewrite: bool) -> Value {
        match v {
            Value::Object(m) => {
                let sub = |x: &Value| form(x, rewrite);
                let kept: serde_json::Map<String, Value> = m.iter()
                    .filter(|(k, _)| !ANNOTATIONS.contains(&k.as_str()) && KNOWN.contains(&k.as_str()))
                    .map(|(k, x)| {
                        let kw = k.as_str();
                        let y = match x {
                            Value::Object(names) if NAME_MAPS.contains(&kw) || kw == "dependencies" => {
                                Value::Object(names.iter().map(|(n, s)| (n.clone(), if s.is_object() { sub(s) } else { s.clone() })).collect())
                            }
                            Value::Object(_) if SCHEMA_VALUED.contains(&kw) => sub(x),
                            Value::Array(xs) if SCHEMA_ARRAYS.contains(&kw) => Value::Array(xs.iter().map(sub).collect()),
                            _ => x.clone(),
                        };
                        (k.clone(), y)
                    })
                    .collect();
                // JUDGE-CANON r1: literals, then r0's type lists.
                let kept = if rewrite {
                    match literals(kept) {
                        Ok(done) => return done,
                        Err(kept) => kept,
                    }
                } else {
                    kept
                };
                let names: Option<Vec<&str>> = match kept.get("type") {
                    Some(Value::Array(ts)) if rewrite && ts.len() > 1 && !kept.contains_key("anyOf") => ts.iter().map(Value::as_str).collect(),
                    _ => None,
                };
                let Some(names) = names else { return Value::Object(kept) };
                let mut out: serde_json::Map<String, Value> = kept.iter()
                    .filter(|(k, _)| k.as_str() != "type" && !is_typed(k))
                    .map(|(k, x)| (k.clone(), x.clone()))
                    .collect();
                let branches = names.iter().map(|t| {
                    let mut b = serde_json::Map::new();
                    b.insert("type".to_string(), Value::String((*t).to_string()));
                    for (k, x) in &kept {
                        if typed_for(t).contains(&k.as_str()) {
                            b.insert(k.clone(), x.clone());
                        }
                    }
                    // JUDGE-CANON r1: a branch the rewrite created is itself in normal form.
                    form(&Value::Object(b), true)
                }).collect();
                out.insert("anyOf".to_string(), Value::Array(branches));
                // JUDGE-CANON r1: the anyOf the rewrite produced may itself be a
                // union of literals (`["boolean", "null"]`): step 3 once more.
                match literals(out) {
                    Ok(done) => done,
                    Err(out) => Value::Object(out),
                }
            }
            _ => v.clone(),
        }
    }

    /// JUDGE-CANON r0 and r1: two nodes equal in normal form, whatever
    /// differs in their spelling or annotations.
    pub(super) fn equivalent(a: &Value, b: &Value) -> bool {
        a.is_object() && b.is_object() && normal(a) == normal(b)
    }

    /// JUDGE-CANON r1: the spelling of this very node differs (its type,
    /// literals, or how many branches it has), so the note belongs here
    /// rather than at a child.
    fn spelled_here(a: &Value, b: &Value) -> bool {
        let shape = |v: &Value| {
            let m = match form(v, false) {
                Value::Object(m) => m,
                _ => return String::new(),
            };
            let count = |k: &str| m.get(k).and_then(Value::as_array).map(Vec::len);
            format!("{:?}|{:?}|{:?}|{:?}|{:?}", m.get("type"), m.get("const"), m.get("enum"), count("anyOf"), count("oneOf"))
        };
        shape(a) != shape(b)
    }

    /// JUDGE-CANON r0: the rewrite happened at this very node, one side
    /// spelling a type list and the other its `anyOf`.
    fn rewritten_here(a: &Value, b: &Value) -> bool {
        let lists = |v: &Value| matches!(v.get("type"), Some(Value::Array(ts)) if ts.len() > 1);
        (lists(a) && b.get("anyOf").is_some()) || (lists(b) && a.get("anyOf").is_some())
    }

    /// JUDGE-CANON r0: record a covered equivalence at `ptr`.
    pub(super) fn note_canon(ptr: &str, type_list: bool) {
        COVER.with(|c| {
            c.borrow_mut().canon.entry(ptr.to_string()).or_insert(type_list);
        });
    }

    fn diff(ptr: &str, via: Option<&str>, a: Option<&Value>, b: Option<&Value>, out: &mut Vec<Diff>) {
        if a == b {
            return;
        }
        // JUDGE-CANON r0 and r1: the same meaning in another spelling is
        // covered. A difference only in annotations is silent; a rewritten
        // node is noted here; above it the diff descends to find the node.
        if let (Some(x), Some(y)) = (a, b) {
            if equivalent(x, y) {
                if form(x, false) == form(y, false) {
                    return;
                }
                if spelled_here(x, y) {
                    note_canon(ptr, rewritten_here(x, y));
                    return;
                }
            }
        }
        let (Some(Value::Object(ma)), Some(Value::Object(mb))) = (a, b) else {
            out.push(Diff { ptr: ptr.to_string(), kw: None, base: a.cloned(), head: b.cloned(), via: via.map(str::to_string) });
            return;
        };
        let keys: BTreeSet<&String> = ma.keys().chain(mb.keys()).collect();
        for k in keys {
            let (va, vb) = (ma.get(k), mb.get(k));
            if va == vb || ANNOTATIONS.contains(&k.as_str()) || !KNOWN.contains(&k.as_str()) {
                continue;
            }
            let here = format!("{ptr}/{}", escape(k));
            let own = |out: &mut Vec<Diff>| out.push(Diff { ptr: ptr.to_string(), kw: Some(k.clone()), base: va.cloned(), head: vb.cloned(), via: None });
            let both_obj = matches!((va, vb), (Some(Value::Object(_)) | None, Some(Value::Object(_)) | None));
            let both_arr = matches!((va, vb), (Some(Value::Array(_)) | None, Some(Value::Array(_)) | None));
            if NAME_MAPS.contains(&k.as_str()) && both_obj {
                let (na, nb) = (va.and_then(Value::as_object), vb.and_then(Value::as_object));
                let names: BTreeSet<&String> = na.into_iter().flat_map(|m| m.keys()).chain(nb.into_iter().flat_map(|m| m.keys())).collect();
                for n in names {
                    diff(&format!("{here}/{}", escape(n)), Some(k), na.and_then(|m| m.get(n)), nb.and_then(|m| m.get(n)), out);
                }
            } else if k == "dependencies" && both_obj {
                let (na, nb) = (va.and_then(Value::as_object), vb.and_then(Value::as_object));
                let names: BTreeSet<&String> = na.into_iter().flat_map(|m| m.keys()).chain(nb.into_iter().flat_map(|m| m.keys())).collect();
                for n in names {
                    let (x, y) = (na.and_then(|m| m.get(n)), nb.and_then(|m| m.get(n)));
                    if x.is_some_and(Value::is_object) && y.is_some_and(Value::is_object) {
                        diff(&format!("{here}/{}", escape(n)), Some(k), x, y, out);
                    } else if x != y {
                        own(out);
                    }
                }
            } else if SCHEMA_VALUED.contains(&k.as_str()) && matches!((va, vb), (Some(Value::Object(_)), Some(Value::Object(_)))) {
                diff(&here, Some(k), va, vb, out);
            } else if SCHEMA_ARRAYS.contains(&k.as_str()) && both_arr && ((va.is_some() && vb.is_some()) || k == "allOf") {
                // Members are compared one by one. For allOf alone a side
                // without the array reads as an empty one (each member is a
                // conjunct of its own); a whole anyOf, oneOf or prefixItems
                // appearing or vanishing is a change of its node's own keyword.
                let none = Vec::new();
                let (xa, xb) = (va.and_then(Value::as_array).unwrap_or(&none), vb.and_then(Value::as_array).unwrap_or(&none));
                for i in 0..xa.len().max(xb.len()) {
                    diff(&format!("{here}/{i}"), Some(k), xa.get(i), xb.get(i), out);
                }
            } else {
                own(out);
            }
        }
    }

    fn under(ptr: &str, prefix: &str) -> bool {
        ptr == prefix || ptr.starts_with(&format!("{prefix}/"))
    }

    fn covered(d: &Diff, c: &Cover) -> bool {
        if c.whole.iter().any(|(_, w)| under(&d.ptr, w)) {
            return true;
        }
        // JUDGE-DRAFT r0: a draft migration's identity keywords at the root.
        if d.ptr.is_empty() && c.draft && matches!(d.kw.as_deref(), Some("$schema" | "id" | "$id")) {
            return true;
        }
        match &d.kw {
            // Opening an object (false to true or absent) rejects nothing.
            Some(k) if k == "additionalProperties" || k == "unevaluatedProperties" => {
                d.base == Some(Value::Bool(false)) && matches!(d.head, None | Some(Value::Bool(true)))
            }
            Some(k) => c.kw.iter().any(|(_, p, kk)| p == &d.ptr && kk == k),
            None => {
                // A property added or removed under a judged `properties` map.
                let Some((parent, tail)) = d.ptr.rsplit_once('/') else { return false };
                let Some(owner) = parent.strip_suffix("/properties").or(if parent == "/properties" { Some("") } else { None }) else {
                    return false;
                };
                let _ = tail;
                c.props.iter().any(|(_, p, adds_ok)| {
                    p == owner && match (&d.base, &d.head) {
                        (Some(_), None) => true,
                        (None, Some(_)) => *adds_ok,
                        _ => false,
                    }
                })
            }
        }
    }

    /// A node on one side only that no property rule covers is read
    /// keyword by keyword against an empty schema: covered exactly when the
    /// walk judged every keyword it carries (a definition the walk reached
    /// through a new `$ref`, an allOf member whose `required` J3 read).
    fn covered_deep(d: &Diff, c: &Cover) -> bool {
        if covered(d, c) {
            return true;
        }
        // Expansion is sound only where a node's existence asserts nothing
        // by itself: a definition (used only through references) and an
        // allOf member (removing a conjunct only widens). A pattern entry,
        // an anyOf or oneOf branch or a prefixItems entry is semantic.
        if d.kw.is_some() || !matches!(d.via.as_deref(), Some("definitions" | "$defs" | "allOf")) {
            return false;
        }
        let empty = Value::Object(serde_json::Map::new());
        let mut sub = Vec::new();
        match (&d.base, &d.head) {
            (Some(b @ Value::Object(_)), None) => diff(&d.ptr, None, Some(b), Some(&empty), &mut sub),
            (None, Some(h @ Value::Object(_))) => diff(&d.ptr, None, Some(&empty), Some(h), &mut sub),
            _ => return false,
        }
        sub.iter().all(|s| covered_deep(s, c))
    }

    /// After the walk: the shallowest unchecked schema nodes, sorted, an
    /// ancestor standing for its descendants.
    pub(super) fn finish(pre: &Value, post: &Value) -> (Vec<String>, Vec<String>) {
        let mut diffs = Vec::new();
        diff("", None, Some(pre), Some(post), &mut diffs);
        let migration = draft_migration(pre, post);
        let unchecked: BTreeSet<String> = COVER.with(|c| {
            let mut c = c.borrow_mut();
            c.draft = migration.is_some();
            diffs.iter().filter(|d| !covered_deep(d, &c)).map(|d| d.ptr.clone()).collect()
        });
        let all: Vec<String> = unchecked.into_iter().collect();
        let shallowest = all.iter().filter(|p| !all.iter().any(|q| q != *p && under(p, q))).cloned().collect();
        let canon: Vec<(String, bool)> = COVER.with(|c| c.borrow().canon.iter().map(|(p, t)| (p.clone(), *t)).collect());
        let mut notes: Vec<String> = migration.into_iter().collect();
        notes.extend(
            canon.iter()
                .filter(|(p, _)| !canon.iter().any(|(q, _)| q != p && under(p, q)))
                .map(|(p, type_list)| {
                    let what = if *type_list { "type lists as anyOf" } else { "literals (const, enum, boolean, null)" };
                    format!("equivalent after rewriting {what} at {}", if p.is_empty() { "the document root" } else { p.as_str() })
                }),
        );
        (shallowest, notes)
    }

    /// The draft a `$schema` URI names, among the three JUDGE-DRAFT r0 reads.
    fn draft_name(v: Option<&Value>) -> Option<&'static str> {
        let u = v?.as_str()?;
        let u = u.strip_suffix('#').unwrap_or(u);
        let u = u.strip_prefix("https://").or_else(|| u.strip_prefix("http://"))?;
        match u {
            "json-schema.org/draft-04/schema" => Some("draft-04"),
            "json-schema.org/draft-06/schema" => Some("draft-06"),
            "json-schema.org/draft-07/schema" => Some("draft-07"),
            _ => None,
        }
    }

    /// Keywords whose meaning differs between draft-04, -06 and -07.
    const DRAFT_GUARDS: [&str; 10] = ["exclusiveMinimum", "exclusiveMaximum", "const", "contains", "propertyNames", "if", "then", "else", "contentMediaType", "contentEncoding"];
    /// Where draft-04 already allows a boolean in place of a schema.
    const DRAFT04_BOOLEANS: [&str; 2] = ["additionalProperties", "additionalItems"];

    /// True when nothing in the schema at this position means something
    /// different between the three drafts: no guard keyword, no boolean
    /// subschema outside DRAFT04_BOOLEANS, no non-local `$ref`.
    fn draft_neutral(s: &Value, via: Option<&str>) -> bool {
        match s {
            Value::Bool(_) => via.is_some_and(|k| DRAFT04_BOOLEANS.contains(&k)),
            Value::Object(m) => m.iter().all(|(k, v)| {
                if DRAFT_GUARDS.contains(&k.as_str()) {
                    return false;
                }
                if k == "$ref" {
                    return v.as_str().is_some_and(|r| r == "#" || r.starts_with("#/"));
                }
                match v {
                    Value::Object(inner) if NAME_MAPS.contains(&k.as_str()) => inner.values().all(|x| draft_neutral(x, Some(k))),
                    Value::Object(inner) if k == "dependencies" => inner.values().all(|x| !x.is_object() && !x.is_boolean() || draft_neutral(x, Some(k))),
                    Value::Object(_) | Value::Bool(_) if SCHEMA_VALUED.contains(&k.as_str()) => draft_neutral(v, Some(k)),
                    Value::Array(xs) if SCHEMA_ARRAYS.contains(&k.as_str()) => xs.iter().all(|x| draft_neutral(x, Some(k))),
                    _ => true,
                }
            }),
            _ => true,
        }
    }

    /// JUDGE-DRAFT r0: the note for a covered migration, or None.
    fn draft_migration(pre: &Value, post: &Value) -> Option<String> {
        let (from, to) = (draft_name(pre.get("$schema"))?, draft_name(post.get("$schema"))?);
        if pre.get("$schema") == post.get("$schema") {
            return None;
        }
        let ident = |d: &Value| d.get("$id").or_else(|| d.get("id")).cloned();
        if ident(pre) != ident(post) || !draft_neutral(pre, None) || !draft_neutral(post, None) {
            return None;
        }
        let renamed = if pre.get("id").is_some() && post.get("$id").is_some() { ", with id renamed $id (same value)" } else { "" };
        Some(format!("draft migration: $schema moves from {from} to {to}{renamed}; no keyword either version uses means something different between those drafts."))
    }
}
