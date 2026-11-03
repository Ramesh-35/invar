//! COUNTEREXAMPLE r0 (docs/falsification/COUNTEREXAMPLE-SPEC.md): for a
//! VIOLATED witness, one instance the base contract accepts and the post
//! contract rejects.
//!
//! One idea: each kind of break only proposes candidates; one checker
//! decides. The checker answers valid, invalid, or unknown, and a
//! candidate is printed only when the base says valid and the post says
//! invalid. Unknown is never rounded to an answer: the output then names
//! the keyword it could not check. So every printed instance is a claim
//! any JSON Schema validator can confirm, and a claim the judge cannot
//! confirm is stated as unconfirmed, never shown as proof.

use crate::judge::Witness;
use serde_json::{json, Map, Value};

/// The largest instance printed, compact JSON bytes.
const MAX_BYTES: usize = 512;

/// The assertion keywords the checker reads. Anything listed in
/// UNCHECKED makes a node unknown; every other keyword is an annotation
/// or outside the vocabulary, which validators ignore.
const UNCHECKED: [&str; 27] = [
    "pattern", "format", "multipleOf", "uniqueItems", "contains", "minContains", "maxContains",
    "minProperties", "maxProperties", "patternProperties", "propertyNames", "dependencies",
    "dependentRequired", "dependentSchemas", "not", "if", "then", "else", "anyOf", "oneOf", "allOf",
    "unevaluatedProperties", "unevaluatedItems", "prefixItems", "additionalItems", "$dynamicRef",
    "$recursiveRef",
];

/// What the checker can say about an instance.
#[derive(Debug, PartialEq)]
enum Check {
    Valid,
    Invalid,
    /// The first keyword met that this checker does not read.
    Unknown(String),
}

/// Conjunction: one definite failure decides; else the first unknown.
fn and(a: Check, b: Check) -> Check {
    match (a, b) {
        (Check::Invalid, _) | (_, Check::Invalid) => Check::Invalid,
        (Check::Unknown(k), _) | (_, Check::Unknown(k)) => Check::Unknown(k),
        _ => Check::Valid,
    }
}

/// Build the counterexample for one witness: (instance, None) or (None,
/// the reason none was built).
pub fn build(w: &Witness, base: &Value, post: &Value) -> (Option<Value>, Option<String>) {
    match try_build(w, base, post) {
        Ok(v) => (Some(v), None),
        Err(reason) => (None, Some(reason)),
    }
}

fn try_build(w: &Witness, base: &Value, post: &Value) -> Result<Value, String> {
    if w.kind == "constraint_changed" {
        return Err("pattern and format changes are not decided".into());
    }
    let (keys, nodes) = follow(base, &w.path, w.kind == "escalated").ok_or("the path could not be followed")?;
    let leaf_schema = nodes.last().cloned().unwrap_or(Value::Null);
    let mut unknown: Option<String> = None;
    for leaf in candidates(w, base, &leaf_schema) {
        let mut inst = minimal(base, base, 0)?;
        place(&mut inst, base, &keys, &nodes, leaf)?;
        if serde_json::to_string(&inst).map(|s| s.len()).unwrap_or(usize::MAX) > MAX_BYTES {
            return Err(format!("the instance would be larger than {MAX_BYTES} bytes"));
        }
        match check(base, base, &inst, 0) {
            Check::Valid => {}
            Check::Unknown(k) => {
                unknown.get_or_insert(k);
                continue;
            }
            Check::Invalid => continue,
        }
        match check(post, post, &inst, 0) {
            Check::Invalid => return Ok(inst),
            Check::Unknown(k) => {
                unknown.get_or_insert(k);
            }
            Check::Valid => {}
        }
    }
    Err(match unknown {
        Some(k) => format!("{k} is not checked by this judge"),
        None => "no candidate value was valid at the base and rejected now".into(),
    })
}

/// The value at the witness path to try, in order. `None` means the leaf
/// is left out (escalated: the instance omits the newly required field).
fn candidates(w: &Witness, root: &Value, leaf: &Value) -> Vec<Option<Value>> {
    let parse = |s: &Option<String>| s.as_deref().and_then(|t| serde_json::from_str::<Value>(t).ok());
    match w.kind {
        "escalated" => vec![None],
        "removed" => vec![minimal(root, leaf, 0).ok()],
        "retyped" => [json!(null), json!(true), json!(0), json!(0.5), json!(""), json!([]), json!({})].into_iter().map(Some).collect(),
        "narrowed" => match (parse(&w.pre_value), parse(&w.post_value)) {
            (Some(Value::Array(a)), Some(Value::Array(b))) => a.into_iter().filter(|v| !b.contains(v)).map(Some).collect(),
            _ => vec![minimal(root, leaf, 0).ok()],
        },
        "tightened" => tightened(w, root, leaf),
        _ => Vec::new(),
    }
}

/// One step past the new bound, then the old inclusive bound; for an enum
/// replaced by a bound (r2 D3), the old enum values.
fn tightened(w: &Witness, root: &Value, leaf: &Value) -> Vec<Option<Value>> {
    let kw = w.keyword.unwrap_or("");
    let num = |s: &Option<String>| s.as_deref().and_then(|t| t.parse::<f64>().ok());
    let int = |s: &Option<String>| s.as_deref().and_then(|t| t.parse::<i64>().ok());
    let integer_only = {
        let t = types(leaf);
        t.contains(&"integer".to_string()) && !t.contains(&"number".to_string())
    };
    let step = |b_int: Option<i64>, b: f64, up: bool| -> Value {
        match (integer_only, b_int) {
            (true, Some(i)) => json!(if up { i + 1 } else { i - 1 }),
            (true, None) => number(if up { b + 1.0 } else { b - 1.0 }),
            (false, _) => number(if up { b + 0.5 } else { b - 0.5 }),
        }
    };
    let item = || leaf.get("items").map(|s| minimal(root, s, 0).unwrap_or(Value::Null)).unwrap_or(Value::Null);
    let text = |n: f64| Value::String("a".repeat(n.max(0.0) as usize));
    let items = |n: f64| Value::Array(vec![item(); n.max(0.0) as usize]);
    let (b, b_int, a) = (num(&w.post_value), int(&w.post_value), num(&w.pre_value));
    let Some(b) = b else { return Vec::new() };
    let mut out: Vec<Value> = Vec::new();
    match kw {
        "maximum" => out.push(step(b_int, b, true)),
        "minimum" => out.push(step(b_int, b, false)),
        "exclusiveMaximum" | "exclusiveMinimum" => out.push(number(b)),
        "maxLength" => out.push(text(b + 1.0)),
        "minLength" if b >= 1.0 => out.push(text(b - 1.0)),
        "maxItems" => out.push(items(b + 1.0)),
        "minItems" if b >= 1.0 => out.push(items(b - 1.0)),
        _ => {}
    }
    if let Some(a) = a {
        match kw {
            "maximum" | "minimum" => out.push(int(&w.pre_value).map(|i| json!(i)).unwrap_or_else(|| number(a))),
            "minLength" => out.push(text(a)),
            "minItems" => out.push(items(a)),
            _ => {}
        }
    }
    if leaf.get("enum").is_some() {
        if let Some(vals) = leaf.get("enum").and_then(Value::as_array) {
            out.extend(vals.iter().cloned());
        }
    }
    out.into_iter().map(Some).collect()
}

/// A whole number as an integer literal, so `0` prints as `0`, not `0.0`.
fn number(v: f64) -> Value {
    if v.fract() == 0.0 && v.abs() < 9_007_199_254_740_992.0 {
        json!(v as i64)
    } else {
        serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null)
    }
}

/// Follow a dotted witness path through `properties` (local `$ref`s
/// resolved). Keys may contain dots: the shortest key that exists wins.
/// Returns the keys and the schema at each step (root first). With
/// `allow_missing_leaf`, the last key need not be declared (a newly
/// required field may be new).
fn follow(root: &Value, path: &str, allow_missing_leaf: bool) -> Option<(Vec<String>, Vec<Value>)> {
    let segs: Vec<&str> = path.split('.').collect();
    let mut keys = Vec::new();
    let mut nodes = vec![resolve(root, root, 0)?];
    let mut i = 0;
    while i < segs.len() {
        let props = nodes.last()?.get("properties").and_then(Value::as_object);
        let mut hit = None;
        for j in i + 1..=segs.len() {
            let key = segs[i..j].join(".");
            if let Some(s) = props.and_then(|p| p.get(&key)) {
                hit = Some((key, s.clone(), j));
                break;
            }
        }
        match hit {
            Some((key, s, j)) => {
                keys.push(key);
                nodes.push(resolve(root, &s, 0)?);
                i = j;
            }
            None if allow_missing_leaf => {
                keys.push(segs[i..].join("."));
                nodes.push(Value::Null);
                i = segs.len();
            }
            None => return None,
        }
    }
    Some((keys, nodes))
}

/// Put `leaf` at the end of `keys`, building every parent object on the
/// way from its schema (its required properties); `None` removes it.
fn place(inst: &mut Value, root: &Value, keys: &[String], nodes: &[Value], leaf: Option<Value>) -> Result<(), String> {
    let Some((last, parents)) = keys.split_last() else {
        return Err("the path could not be followed".into());
    };
    let mut cur = inst;
    if !cur.is_object() {
        *cur = object_of(root, &nodes[0], 0)?;
    }
    for (k, node) in parents.iter().zip(&nodes[1..]) {
        let obj = cur.as_object_mut().ok_or("the path could not be followed")?;
        if !obj.get(k).is_some_and(Value::is_object) {
            obj.insert(k.clone(), object_of(root, node, 0)?);
        }
        cur = obj.get_mut(k).ok_or("the path could not be followed")?;
    }
    let obj = cur.as_object_mut().ok_or("the path could not be followed")?;
    match leaf {
        Some(v) => {
            obj.insert(last.clone(), v);
        }
        None => {
            obj.remove(last);
        }
    }
    Ok(())
}

/// The smallest instance a schema allows, deterministically (spec order:
/// const, first enum value, first declared type).
fn minimal(root: &Value, schema: &Value, depth: usize) -> Result<Value, String> {
    if depth > 32 {
        return Err(format!("the instance would be larger than {MAX_BYTES} bytes"));
    }
    let Some(s) = resolve(root, schema, 0) else { return Ok(Value::Null) };
    if let Some(c) = s.get("const") {
        return Ok(c.clone());
    }
    if let Some(first) = s.get("enum").and_then(Value::as_array).and_then(|a| a.first()) {
        return Ok(first.clone());
    }
    let ty = types(&s).into_iter().next().unwrap_or_default();
    Ok(match ty.as_str() {
        "boolean" => json!(true),
        "integer" | "number" => {
            let f = |k: &str| s.get(k).and_then(Value::as_f64);
            let int = ty == "integer";
            let mut v = 0.0_f64;
            if let Some(lo) = f("minimum") {
                v = v.max(if int { lo.ceil() } else { lo });
            }
            if let Some(lo) = f("exclusiveMinimum") {
                v = v.max(if int { lo.floor() + 1.0 } else { lo + 1.0 });
            }
            if let Some(hi) = f("maximum") {
                v = v.min(if int { hi.floor() } else { hi });
            }
            if let Some(hi) = f("exclusiveMaximum") {
                v = v.min(if int { hi.ceil() - 1.0 } else { hi - 1.0 });
            }
            number(v)
        }
        "string" => Value::String("a".repeat(s.get("minLength").and_then(Value::as_u64).unwrap_or(0) as usize)),
        "array" => {
            let n = s.get("minItems").and_then(Value::as_u64).unwrap_or(0) as usize;
            let item = match s.get("items") {
                Some(i) if i.is_object() || i.is_boolean() => minimal(root, i, depth + 1)?,
                _ => Value::Null,
            };
            Value::Array(vec![item; n])
        }
        "object" => object_of(root, &s, depth)?,
        _ => Value::Null,
    })
}

/// An object carrying exactly the schema's required properties.
fn object_of(root: &Value, schema: &Value, depth: usize) -> Result<Value, String> {
    let s = resolve(root, schema, 0).unwrap_or(Value::Null);
    let mut m = Map::new();
    let req: Vec<&str> = s.get("required").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
    for name in req {
        let sub = s.get("properties").and_then(|p| p.get(name)).cloned().unwrap_or(Value::Null);
        m.insert(name.to_string(), minimal(root, &sub, depth + 1)?);
    }
    Ok(Value::Object(m))
}

/// Declared types, in order.
fn types(s: &Value) -> Vec<String> {
    match s.get("type") {
        Some(Value::String(t)) => vec![t.clone()],
        Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

/// A local `$ref` resolved against its own document, at most 16 hops;
/// None when it cannot be (external, missing, too deep).
fn resolve(root: &Value, s: &Value, hops: usize) -> Option<Value> {
    match s.get("$ref").and_then(Value::as_str) {
        None => Some(s.clone()),
        Some(r) if hops < 16 && r.starts_with('#') => {
            let mut cur = root;
            for part in r.trim_start_matches('#').split('/').filter(|p| !p.is_empty()) {
                let key = part.replace("~1", "/").replace("~0", "~");
                cur = match cur {
                    Value::Object(m) => m.get(&key)?,
                    Value::Array(a) => a.get(key.parse::<usize>().ok()?)?,
                    _ => return None,
                };
            }
            resolve(root, cur, hops + 1)
        }
        Some(_) => None,
    }
}

/// The JSON types an instance has (`integer` for whole numbers, which
/// are also `number`).
fn instance_types(v: &Value) -> Vec<&'static str> {
    match v {
        Value::Null => vec!["null"],
        Value::Bool(_) => vec!["boolean"],
        Value::Number(n) => {
            let whole = n.is_i64() || n.is_u64() || n.as_f64().is_some_and(|f| f.fract() == 0.0);
            if whole { vec!["integer", "number"] } else { vec!["number"] }
        }
        Value::String(_) => vec!["string"],
        Value::Array(_) => vec!["array"],
        Value::Object(_) => vec!["object"],
    }
}

/// Validate `inst` against `schema` for the keywords this checker reads.
fn check(root: &Value, schema: &Value, inst: &Value, depth: usize) -> Check {
    if depth > 64 {
        return Check::Unknown("$ref".into());
    }
    let s = match schema {
        Value::Bool(true) => return Check::Valid,
        Value::Bool(false) => return Check::Invalid,
        Value::Object(_) => schema,
        _ => return Check::Valid,
    };
    if s.get("$ref").is_some() {
        // A reference is followed only when it is local and stands alone:
        // drafts disagree on what keywords beside it mean.
        let alone = s.as_object().is_some_and(|m| m.keys().all(|k| k == "$ref" || !asserts(k)));
        return match resolve(root, s, 0) {
            Some(t) if alone => check(root, &t, inst, depth + 1),
            _ => Check::Unknown("$ref".into()),
        };
    }
    let mut out = Check::Valid;
    let m = s.as_object().map(|m| m.iter().collect::<Vec<_>>()).unwrap_or_default();
    for (k, v) in m {
        let k = k.as_str();
        let c = if UNCHECKED.contains(&k) {
            Check::Unknown(k.to_string())
        } else {
            keyword(root, s, k, v, inst, depth)
        };
        out = and(out, c);
    }
    out
}

/// Does a keyword assert anything this checker must account for?
fn asserts(k: &str) -> bool {
    UNCHECKED.contains(&k)
        || ["type", "properties", "required", "additionalProperties", "enum", "const", "minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum", "minLength", "maxLength", "minItems", "maxItems", "items"].contains(&k)
}

fn keyword(root: &Value, s: &Value, k: &str, v: &Value, inst: &Value, depth: usize) -> Check {
    let ok = |b: bool| if b { Check::Valid } else { Check::Invalid };
    let num = inst.as_f64().filter(|_| inst.is_number());
    let len = |x: &str| x.chars().count() as f64;
    match k {
        "type" => {
            let want = match v {
                Value::String(t) => vec![t.as_str()],
                Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
                _ => return Check::Unknown("type".into()),
            };
            if want.iter().any(|t| !["null", "boolean", "integer", "number", "string", "array", "object"].contains(t)) {
                return Check::Unknown("type".into());
            }
            ok(instance_types(inst).iter().any(|t| want.contains(t)))
        }
        "enum" => match v.as_array() {
            Some(a) => ok(a.iter().any(|x| same(x, inst))),
            None => Check::Unknown("enum".into()),
        },
        "const" => ok(same(v, inst)),
        "minimum" | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" => {
            let Some(b) = v.as_f64() else { return Check::Unknown(k.to_string()) };
            match num {
                None => Check::Valid,
                Some(x) => ok(match k {
                    "minimum" => x >= b,
                    "maximum" => x <= b,
                    "exclusiveMinimum" => x > b,
                    _ => x < b,
                }),
            }
        }
        "minLength" | "maxLength" => match (v.as_f64(), inst.as_str()) {
            (Some(b), Some(t)) => ok(if k == "minLength" { len(t) >= b } else { len(t) <= b }),
            (None, _) => Check::Unknown(k.to_string()),
            _ => Check::Valid,
        },
        "minItems" | "maxItems" => match (v.as_f64(), inst.as_array()) {
            (Some(b), Some(a)) => ok(if k == "minItems" { a.len() as f64 >= b } else { a.len() as f64 <= b }),
            (None, _) => Check::Unknown(k.to_string()),
            _ => Check::Valid,
        },
        "items" => match (v, inst.as_array()) {
            (Value::Array(_), Some(_)) => Check::Unknown("items".into()),
            (_, Some(a)) => a.iter().fold(Check::Valid, |acc, x| and(acc, check(root, v, x, depth + 1))),
            _ => Check::Valid,
        },
        "required" => match (v.as_array(), inst.as_object()) {
            (Some(r), Some(o)) => ok(r.iter().filter_map(Value::as_str).all(|n| o.contains_key(n))),
            _ => Check::Valid,
        },
        "properties" => match (v.as_object(), inst.as_object()) {
            (Some(p), Some(o)) => o.iter().filter_map(|(n, x)| p.get(n).map(|sub| (sub, x))).fold(Check::Valid, |acc, (sub, x)| and(acc, check(root, sub, x, depth + 1))),
            _ => Check::Valid,
        },
        "additionalProperties" => {
            let Some(o) = inst.as_object() else { return Check::Valid };
            if s.get("patternProperties").is_some() {
                // Keys patternProperties admits are not "additional"; that
                // keyword is not read, so neither is this one decided here.
                return Check::Unknown("patternProperties".into());
            }
            let declared = s.get("properties").and_then(Value::as_object);
            let extra = o.iter().filter(|(n, _)| !declared.is_some_and(|p| p.contains_key(*n)));
            extra.fold(Check::Valid, |acc, (_, x)| and(acc, check(root, v, x, depth + 1)))
        }
        _ => Check::Valid,
    }
}

/// JSON equality with numbers compared by value (1 equals 1.0).
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w))),
        _ => a == b,
    }
}
