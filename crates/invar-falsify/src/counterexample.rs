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
/// COUNTEREXAMPLE r1 reads minProperties, maxProperties, uniqueItems,
/// dependentRequired, dependencies (array form) and multipleOf; r2 reads
/// allOf, anyOf, oneOf and not.
const UNCHECKED: [&str; 17] = [
    "pattern", "format", "contains", "minContains", "maxContains", "patternProperties",
    "propertyNames", "dependentSchemas", "if", "then", "else", "unevaluatedProperties",
    "unevaluatedItems", "prefixItems", "additionalItems", "$dynamicRef", "$recursiveRef",
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
    let (keys, nodes) = follow(base, &w.path, w.kind == "escalated").ok_or("the path could not be followed")?;
    let leaf_schema = nodes.last().cloned().unwrap_or(Value::Null);
    let mut unknown: Option<String> = None;
    for leaf in candidates(w, base, &leaf_schema) {
        let mut inst = minimal(base, base, 0)?;
        place(&mut inst, base, &keys, &nodes, leaf)?;
        complete(base, base, &mut inst, 0)?;
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
        "retyped" => {
            // JUDGE-D9 r0: the base enum (or const) values first, so the
            // value the new type excludes is the instance shown.
            let mut c: Vec<Value> = leaf.get("enum").and_then(Value::as_array).cloned().unwrap_or_default();
            c.extend(leaf.get("const").cloned());
            c.extend([json!(null), json!(true), json!(0), json!(0.5), json!(""), json!([]), json!({})]);
            c.into_iter().map(Some).collect()
        }
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
            // cx_f14: a bound at the i64 edge saturates; the candidate then
            // fails the violation check and is stated, never a crash.
            (true, Some(i)) => json!(if up { i.saturating_add(1) } else { i.saturating_sub(1) }),
            (true, None) => number(if up { b + 1.0 } else { b - 1.0 }),
            (false, _) => number(if up { b + 0.5 } else { b - 0.5 }),
        }
    };
    let text = |n: f64| Value::String("a".repeat(n.max(0.0) as usize));
    let items = |n: f64| items_of(root, leaf, n.max(0.0) as usize, 0).unwrap_or(Value::Null);
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

/// One step of a witness path: a property, or (JUDGE-ITEMS r0) an
/// array's element.
enum Step {
    Key(String),
    Item,
}

/// Follow a dotted witness path through `properties` (local `$ref`s
/// resolved). Keys may contain dots: the shortest key that exists wins. A
/// segment ending in `[]` that names no property steps into the element
/// schema (`items`), once per `[]` (JUDGE-ITEMS r0). Returns the steps and
/// the schema at each step (root first). With `allow_missing_leaf`, the
/// last key need not be declared (a newly required field may be new).
fn follow(root: &Value, path: &str, allow_missing_leaf: bool) -> Option<(Vec<Step>, Vec<Value>)> {
    let segs: Vec<&str> = path.split('.').collect();
    let mut steps = Vec::new();
    let mut nodes = vec![resolve(root, root, 0)?];
    let mut i = 0;
    while i < segs.len() {
        let mut hit = None;
        for j in i + 1..=segs.len() {
            let key = segs[i..j].join(".");
            let props = nodes.last()?.get("properties").and_then(Value::as_object);
            if let Some(s) = props.and_then(|p| p.get(&key)) {
                hit = Some((key, s.clone(), 0, j));
                break;
            }
            // JUDGE-ITEMS r0: `name[]` is name's element, unless a property
            // bears that literal name (tried first, above).
            let base = key.trim_end_matches("[]");
            let depth = (key.len() - base.len()) / 2;
            if depth > 0 {
                let target = if base.is_empty() { Some(nodes.last()?.clone()) } else { props.and_then(|p| p.get(base)).cloned() };
                if let Some(s) = target {
                    hit = Some((base.to_string(), s, depth, j));
                    break;
                }
            }
        }
        match hit {
            Some((key, s, depth, j)) => {
                if !key.is_empty() {
                    steps.push(Step::Key(key));
                    nodes.push(resolve(root, &s, 0)?);
                }
                for _ in 0..depth {
                    let el = nodes.last()?.get("items").filter(|v| v.is_object()).cloned().unwrap_or_else(|| json!({}));
                    steps.push(Step::Item);
                    nodes.push(resolve(root, &el, 0)?);
                }
                i = j;
            }
            None if allow_missing_leaf => {
                steps.push(Step::Key(segs[i..].join(".")));
                nodes.push(Value::Null);
                i = segs.len();
            }
            None => return None,
        }
    }
    Some((steps, nodes))
}

/// Put `leaf` at the end of `steps`, building every parent on the way from
/// its schema: an object carrying its required properties, or for an
/// element step a one-element array (JUDGE-ITEMS r0); `None` removes it.
fn place(inst: &mut Value, root: &Value, steps: &[Step], nodes: &[Value], leaf: Option<Value>) -> Result<(), String> {
    let Some((last, parents)) = steps.split_last() else {
        return Err("the path could not be followed".into());
    };
    let mut cur = inst;
    if !cur.is_object() {
        *cur = object_of(root, &nodes[0], 0)?;
    }
    // The container each step writes into is built from the schema of the
    // node that step leaves; the next step decides object or array.
    let shell = |next: &Step, node: &Value| -> Result<Value, String> {
        match next {
            Step::Key(_) => object_of(root, node, 0),
            Step::Item => Ok(Value::Array(Vec::new())),
        }
    };
    let rest: Vec<&Step> = steps.iter().skip(1).collect();
    for (n, (step, node)) in parents.iter().zip(&nodes[1..]).enumerate() {
        let next = rest[n];
        match step {
            Step::Key(k) => {
                let obj = cur.as_object_mut().ok_or("the path could not be followed")?;
                let fits = match next {
                    Step::Key(_) => obj.get(k).is_some_and(Value::is_object),
                    Step::Item => obj.get(k).is_some_and(Value::is_array),
                };
                if !fits {
                    obj.insert(k.clone(), shell(next, node)?);
                }
                cur = obj.get_mut(k).ok_or("the path could not be followed")?;
            }
            Step::Item => {
                let arr = cur.as_array_mut().ok_or("the path could not be followed")?;
                arr.truncate(1);
                if arr.is_empty() {
                    arr.push(shell(next, node)?);
                }
                cur = arr.get_mut(0).ok_or("the path could not be followed")?;
            }
        }
    }
    match (last, leaf) {
        (Step::Key(k), Some(v)) => {
            cur.as_object_mut().ok_or("the path could not be followed")?.insert(k.clone(), v);
        }
        (Step::Key(k), None) => {
            cur.as_object_mut().ok_or("the path could not be followed")?.remove(k);
        }
        (Step::Item, Some(v)) => *cur = Value::Array(vec![v]),
        (Step::Item, None) => return Err("the path could not be followed".into()),
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
    if ty.is_empty() {
        // COUNTEREXAMPLE r2: the first anyOf/oneOf member whose minimal
        // value the checker says is valid for this node; else the first.
        for kw in ["anyOf", "oneOf"] {
            if let Some(members) = s.get(kw).and_then(Value::as_array) {
                let mut first = None;
                for m in members {
                    let cand = minimal(root, m, depth + 1)?;
                    if check(root, &s, &cand, 0) == Check::Valid {
                        return Ok(cand);
                    }
                    first.get_or_insert(cand);
                }
                if let Some(c) = first {
                    return Ok(c);
                }
            }
        }
    }
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
            // r1: the smallest multiple at or above the value so far.
            if let Some(m) = f("multipleOf").filter(|m| *m > 0.0) {
                v = (v / m).ceil() * m;
            }
            number(v)
        }
        "string" => Value::String("a".repeat(s.get("minLength").and_then(Value::as_u64).unwrap_or(0) as usize)),
        "array" => {
            let n = s.get("minItems").and_then(Value::as_u64).unwrap_or(0) as usize;
            items_of(root, &s, n, depth)?
        }
        "object" => object_of(root, &s, depth)?,
        _ => Value::Null,
    })
}

/// n items for an array schema; under `uniqueItems: true`, n distinct
/// values where the item type allows it (r1), else n copies (the checker
/// then rejects, and nothing is printed).
fn items_of(root: &Value, s: &Value, n: usize, depth: usize) -> Result<Value, String> {
    let item_schema = match s.get("items") {
        Some(i) if i.is_object() || i.is_boolean() => i.clone(),
        _ => Value::Null,
    };
    let first = minimal(root, &item_schema, depth + 1)?;
    if s.get("uniqueItems") != Some(&Value::Bool(true)) {
        return Ok(Value::Array(vec![first; n]));
    }
    let it = resolve(root, &item_schema, 0).unwrap_or(Value::Null);
    let distinct: Option<Vec<Value>> = match types(&it).first().map(String::as_str) {
        Some("string") => first.as_str().map(|base| (0..n).map(|i| Value::String(format!("{base}{}", "a".repeat(i)))).collect()),
        Some("integer") | Some("number") => first.as_f64().map(|base| (0..n).map(|i| number(base + i as f64)).collect()),
        Some("boolean") if n <= 2 => Some([json!(true), json!(false)].into_iter().take(n).collect()),
        _ => None,
    };
    Ok(Value::Array(distinct.unwrap_or_else(|| vec![first; n])))
}

/// An object carrying the schema's required properties, padded with
/// declared ones (sorted) up to `minProperties` (r1).
fn object_of(root: &Value, schema: &Value, depth: usize) -> Result<Value, String> {
    let s = resolve(root, schema, 0).unwrap_or(Value::Null);
    let mut m = Map::new();
    let own = |v: &Value| -> Vec<String> {
        v.get("required").and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default()
    };
    let mut req = own(&s);
    // COUNTEREXAMPLE r2: names required by direct allOf members (a local
    // $ref member resolved) are required too.
    for member in s.get("allOf").and_then(Value::as_array).into_iter().flatten() {
        if let Some(r) = resolve(root, member, 0) {
            req.extend(own(&r));
        }
    }
    for name in req.iter().map(String::as_str) {
        let sub = s.get("properties").and_then(|p| p.get(name)).cloned().unwrap_or(Value::Null);
        m.insert(name.to_string(), minimal(root, &sub, depth + 1)?);
    }
    let want = s.get("minProperties").and_then(Value::as_u64).unwrap_or(0) as usize;
    if let Some(props) = s.get("properties").and_then(Value::as_object) {
        for (name, sub) in props {
            if m.len() >= want {
                break;
            }
            if !m.contains_key(name) {
                m.insert(name.clone(), minimal(root, sub, depth + 1)?);
            }
        }
    }
    Ok(Value::Object(m))
}

/// Add, for every present key, the keys its `dependentRequired` (or array
/// `dependencies`) names, when the object declares them, recursively (r1).
fn complete(root: &Value, schema: &Value, inst: &mut Value, depth: usize) -> Result<(), String> {
    if depth > 32 {
        return Ok(());
    }
    let Some(s) = resolve(root, schema, 0) else { return Ok(()) };
    let props = s.get("properties").and_then(Value::as_object).cloned().unwrap_or_default();
    if let Some(o) = inst.as_object_mut() {
        loop {
            let mut added = false;
            for kw in ["dependentRequired", "dependencies"] {
                let Some(deps) = s.get(kw).and_then(Value::as_object) else { continue };
                for (key, names) in deps {
                    if !o.contains_key(key) {
                        continue;
                    }
                    for n in names.as_array().into_iter().flatten().filter_map(Value::as_str) {
                        if !o.contains_key(n) {
                            if let Some(sub) = props.get(n) {
                                o.insert(n.to_string(), minimal(root, sub, depth + 1)?);
                                added = true;
                            }
                        }
                    }
                }
            }
            if !added {
                break;
            }
        }
        for (k, v) in o.iter_mut() {
            if let Some(sub) = props.get(k) {
                complete(root, sub, v, depth + 1)?;
            }
        }
    } else if let (Some(a), Some(el)) = (inst.as_array_mut(), s.get("items").filter(|i| i.is_object())) {
        // JUDGE-ITEMS r0: a placed element is completed by its own schema.
        for v in a.iter_mut() {
            complete(root, el, v, depth + 1)?;
        }
    }
    Ok(())
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
        || ["type", "properties", "required", "additionalProperties", "enum", "const", "minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum", "minLength", "maxLength", "minItems", "maxItems", "items", "minProperties", "maxProperties", "uniqueItems", "dependentRequired", "dependencies", "multipleOf", "allOf", "anyOf", "oneOf", "not"].contains(&k)
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
        "minProperties" | "maxProperties" => match (v.as_f64(), inst.as_object()) {
            (Some(b), Some(o)) => ok(if k == "minProperties" { o.len() as f64 >= b } else { o.len() as f64 <= b }),
            (None, _) => Check::Unknown(k.to_string()),
            _ => Check::Valid,
        },
        "uniqueItems" => match (v.as_bool(), inst.as_array()) {
            (Some(true), Some(a)) => ok(!a.iter().enumerate().any(|(i, x)| a[i + 1..].iter().any(|y| same(x, y)))),
            (Some(_), _) | (None, None) => Check::Valid,
            (None, Some(_)) => Check::Unknown(k.to_string()),
        },
        "dependentRequired" | "dependencies" => {
            let Some(deps) = v.as_object() else { return Check::Unknown(k.to_string()) };
            // dependencies with a schema value is not read (r1): unknown.
            if deps.values().any(|d| !d.is_array()) {
                return Check::Unknown(k.to_string());
            }
            match inst.as_object() {
                Some(o) => ok(deps.iter().filter(|(key, _)| o.contains_key(*key)).all(|(_, d)| {
                    d.as_array().is_some_and(|names| names.iter().filter_map(Value::as_str).all(|n| o.contains_key(n)))
                })),
                None => Check::Valid,
            }
        }
        "multipleOf" => match (v.as_f64(), num) {
            (Some(m), Some(x)) if m > 0.0 => {
                let q = x / m;
                ok((q - q.round()).abs() < 1e-9)
            }
            (Some(m), _) if m > 0.0 => Check::Valid,
            _ => Check::Unknown(k.to_string()),
        },
        // COUNTEREXAMPLE r2: combinators over checks already made. An
        // unknown keeps the inner keyword's name, never the combinator's.
        "allOf" | "anyOf" | "oneOf" => {
            let Some(members) = v.as_array() else { return Check::Unknown(k.to_string()) };
            let results: Vec<Check> = members.iter().map(|m| check(root, m, inst, depth + 1)).collect();
            let valid = results.iter().filter(|c| **c == Check::Valid).count();
            let first_unknown = results.iter().find_map(|c| match c {
                Check::Unknown(r) => Some(r.clone()),
                _ => None,
            });
            match k {
                "allOf" => results.into_iter().fold(Check::Valid, and),
                "anyOf" => match (valid, first_unknown) {
                    (n, _) if n > 0 => Check::Valid,
                    (_, Some(r)) => Check::Unknown(r),
                    _ => Check::Invalid,
                },
                _ => match (valid, first_unknown) {
                    (n, _) if n >= 2 => Check::Invalid,
                    (1, None) => Check::Valid,
                    (_, Some(r)) => Check::Unknown(r),
                    _ => Check::Invalid,
                },
            }
        }
        "not" => match check(root, v, inst, depth + 1) {
            Check::Valid => Check::Invalid,
            Check::Invalid => Check::Valid,
            u => u,
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
