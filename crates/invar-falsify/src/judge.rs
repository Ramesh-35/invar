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
const UNJUDGED: [&str; 4] = ["enum", "pattern", "maximum", "required"];

pub struct Witness {
    pub path: String,
    pub kind: &'static str, // "removed" | "retyped"
    pub pre_type: String,
    pub post_type: Option<String>,
    pub pre_line: Option<u64>,
    pub post_line: Option<u64>,
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
    let post_bytes = match std::fs::read(root.join(contract)) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) => anyhow::bail!("reading {contract} from the working tree: {e}"),
    };
    let (verdict, reason, witnesses, pre_text) = match base_bytes(root, base, contract)? {
        BaseRead::NoRef => ("NOT EVALUATED", Some("unreadable_base"), Vec::new(), None),
        BaseRead::NoFile => ("NOT EVALUATED", Some("new_contract"), Vec::new(), None),
        BaseRead::Found(bytes) => {
            let pre_text = String::from_utf8_lossy(&bytes).into_owned();
            match (serde_json::from_str::<Value>(&pre_text), serde_json::from_str::<Value>(&post_bytes)) {
                (Err(_), _) | (_, Err(_)) => ("NOT EVALUATED", Some("unparsable"), Vec::new(), Some(pre_text)),
                (Ok(pre), Ok(post)) => {
                    let mut w = Vec::new();
                    walk("", &pre, &post, &mut w);
                    w.sort_by(|a, b| a.path.cmp(&b.path));
                    for x in &mut w {
                        let last = x.path.rsplit('.').next().unwrap_or(&x.path).to_string();
                        x.pre_line = find_line(&pre_text, &last);
                        if x.kind == "retyped" {
                            x.post_line = find_line(&post_bytes, &last);
                        }
                    }
                    let v = if w.is_empty() { "HOLDS" } else { "VIOLATED" };
                    (v, None, w, Some(pre_text))
                }
            }
        }
    };
    let _ = pre_text; // pre bytes are consumed by line lookup above
    let body = body_json(verdict, reason, contract, base, &witnesses);
    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&body).unwrap_or_default()),
        _ => print!("{}", render_text(verdict, reason, contract, base, &witnesses)),
    }
    if let Some(p) = receipt_out {
        std::fs::write(p, crate::receipt::render(&body)).with_context(|| format!("writing receipt {p}"))?;
    }
    Ok(match verdict {
        "HOLDS" => 0,
        "VIOLATED" => 1,
        _ => 4,
    })
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
                    });
                }
                walk(&dotted, pv, qv, out);
            }
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
        "judged": "property presence and type, recursive",
        "unjudged_constraints": UNJUDGED,
        "witnesses": w.iter().map(|x| json!({
            "path": x.path,
            "kind": x.kind,
            "pre_type": x.pre_type,
            "post_type": x.post_type,
            "contract": contract,
            "pre_line": x.pre_line,
            "post_line": x.post_line,
        })).collect::<Vec<_>>(),
    })
}

/// The human surface, frozen as part of the contract (JUDGE-SPEC r1,
/// Voice): sentences that name the field, the change, and a next step.
fn render_text(verdict: &str, reason: Option<&str>, contract: &str, base: &str, w: &[Witness]) -> String {
    let mut s = String::new();
    match (verdict, reason) {
        ("HOLDS", _) => {
            s.push_str(&format!("HOLDS: {contract} keeps every promise it made at {base}.\n"));
        }
        ("VIOLATED", _) => {
            let n = w.len();
            let noun = if n == 1 { "promise" } else { "promises" };
            s.push_str(&format!("VIOLATED: {contract} breaks {n} {noun} made at {base}.\n"));
            for x in w {
                match x.kind {
                    "removed" => {
                        let at = x.pre_line.map(|l| format!(" (line {l} at {base})")).unwrap_or_default();
                        s.push_str(&format!(
                            "  removed: {} was {}{at}. Consumers reading this field will break; keep the field, or ship this as a new major version and migrate them.\n",
                            x.path, x.pre_type
                        ));
                    }
                    _ => {
                        let to = x.post_type.as_deref().unwrap_or("unspecified");
                        s.push_str(&format!(
                            "  retyped: {} was {}, is now {to}. Consumers parsing the old type can misread values; restore the type, or version the change.\n",
                            x.path, x.pre_type
                        ));
                    }
                }
            }
        }
        (_, Some("new_contract")) => {
            s.push_str(&format!(
                "NOT EVALUATED (new_contract): {contract} does not exist at {base}; nothing was promised before. The next change to this contract will be judged against this version.\n"
            ));
        }
        (_, Some("unreadable_base")) => {
            s.push_str(&format!(
                "NOT EVALUATED (unreadable_base): the ref {base} cannot be read in this repository. Fetch it, or pass --base a ref that exists.\n"
            ));
        }
        _ => {
            s.push_str(&format!(
                "NOT EVALUATED (unparsable): could not read {contract} as JSON (checked the working tree and {base}). Fix the JSON before judging.\n"
            ));
        }
    }
    s.push_str(&format!(
        "judged: property presence and type, recursively. not judged in r0: {} (stated, never skipped).\n",
        UNJUDGED.join(", ")
    ));
    s
}
