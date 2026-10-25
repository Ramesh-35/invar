//! Field surfaces: what a producer provides and what a consumer reads.
//!
//! Producer side, two sources (founder decision 2026-10-08, dual-source):
//!   * a published spec (JSON or YAML; OpenAPI / JSON Schema `properties`);
//!   * the producer's code, read heuristically: Go struct fields (json tag,
//!     else the field name, as encoding/json does), Python class attributes
//!     (`name: Type`, with `alias="..."`), TypeScript members (`name?: Type`),
//!     Rust struct fields (with `#[serde(rename = "...")]`).
//!
//! Consumer side, `broad_reads`: dot access (`x.field`, `x?.field`),
//! subscripts (`x["field"]`, `x['field']`), `.get("field")`, Go `json:"field"`
//! tags, serde `rename = "field"`, `alias="field"`, and declared fields
//! `name: Type`. This is deliberately broad and therefore noisy; it is only
//! used when a contract names the producer's previous version, where it is
//! intersected with the fields that actually changed.
//!
//! Producer fields are bound to their declaring scope (alpha.4): the Go,
//! Rust or TypeScript type, the Python class (attributes at class-body level
//! only, never function parameters), or the enclosing function for payloads
//! assembled at runtime. A field is "removed" when its name no longer exists
//! in any scope of the new version; a type change is only compared within the
//! same scope, so same-named fields of different types do not collide.
//!
//! HONEST SCOPE: pattern matching, not parsing. Nested paths are flattened to
//! names; consumer reads are not scoped (the consumer's types are unknown).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// (scope, field name) -> declared type, if known. Specs use scope "".
pub type Fields = BTreeMap<(String, String), Option<String>>;

/// The distinct field names, across scopes.
pub fn field_names(f: &Fields) -> std::collections::BTreeSet<String> {
    f.keys().map(|(_, n)| n.clone()).collect()
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn ident_at(s: &str, start: usize) -> Option<&str> {
    let b = s.as_bytes();
    if start >= b.len() || !(b[start].is_ascii_alphabetic() || b[start] == b'_') {
        return None;
    }
    let mut end = start;
    while end < b.len() && is_ident_byte(b[end]) {
        end += 1;
    }
    Some(&s[start..end])
}

/// `properties` of a spec object, with each property's type.
pub fn spec_fields(v: &Value) -> Fields {
    let mut out = Fields::new();
    if let Some(props) = v.get("properties").and_then(|p| p.as_object()) {
        for (name, p) in props {
            out.insert((String::new(), name.clone()), spec_type(p));
        }
    }
    out
}

fn spec_type(p: &Value) -> Option<String> {
    if let Some(r) = p.get("$ref").and_then(|r| r.as_str()) {
        return Some(format!("ref:{}", r.rsplit('/').next().unwrap_or(r)));
    }
    let t = p.get("type")?;
    let base = match t {
        Value::String(s) => s.clone(),
        Value::Array(a) => a
            .iter()
            .filter_map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join("|"),
        _ => return None,
    };
    let mut ty = base.clone();
    if base == "array" {
        if let Some(items) = p.get("items").and_then(spec_type) {
            ty = format!("array<{items}>");
        }
    }
    // A1 rule 3: nullability is part of the type (OpenAPI 3.0 `nullable`).
    if p.get("nullable").and_then(Value::as_bool) == Some(true) && !ty.contains("null") {
        ty.push_str("|null");
    }
    Some(ty)
}

/// Code-extracted fields, chosen by file extension. Unknown extensions give
/// `None` (the caller reports an error, never an empty surface).
pub fn code_fields(path: &str, text: &str) -> Option<Fields> {
    let ext = path.rsplit('.').next().unwrap_or("");
    let mut f = match ext {
        "go" => go_fields(text),
        "py" => py_fields(text),
        "ts" | "tsx" => ts_fields(text),
        "js" | "jsx" | "mjs" | "cjs" => Fields::new(),
        "rs" => rs_fields(text),
        _ => return None,
    };
    // Payloads assembled at runtime: dict/map/object literal keys and keyed
    // assignments, for Python, JS/TS and Go, scoped to the enclosing
    // function (Go, Python) or the file (JS/TS). Untyped.
    if matches!(
        ext,
        "py" | "go" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs"
    ) {
        let scopes = line_scopes(text, ext);
        for (ln, key) in assembled_keys(text, ext) {
            let scope = scopes.get(ln).cloned().unwrap_or_default();
            f.entry((scope, key)).or_insert(None);
        }
    }
    Some(f)
}

/// Enclosing function per line: Go `func` (by brace depth) and Python
/// `def`/`class` (by indentation). Other languages: file scope "".
fn line_scopes(text: &str, ext: &str) -> Vec<String> {
    let mut out = Vec::new();
    match ext {
        "go" => {
            let (mut scope, mut depth, mut open_at) = (String::new(), 0i64, -1i64);
            for line in text.lines() {
                let t = line.trim_start();
                if t.starts_with("func ") {
                    let rest = t["func ".len()..].trim_start();
                    let rest = if rest.starts_with('(') {
                        rest.find(')')
                            .map(|i| rest[i + 1..].trim_start())
                            .unwrap_or("")
                    } else {
                        rest
                    };
                    if let Some(id) = ident_at(rest, 0) {
                        scope = id.to_string();
                        open_at = depth;
                    }
                }
                out.push(scope.clone());
                depth += line.matches('{').count() as i64 - line.matches('}').count() as i64;
                if open_at >= 0 && depth <= open_at {
                    scope.clear();
                    open_at = -1;
                }
            }
        }
        "py" => {
            let mut stack: Vec<(usize, String)> = Vec::new();
            for line in text.lines() {
                let t = line.trim();
                let ind = line.len() - line.trim_start().len();
                if !t.is_empty() && !t.starts_with('#') {
                    while stack.last().map(|(i, _)| *i >= ind).unwrap_or(false) {
                        stack.pop();
                    }
                    for kw in ["class ", "def ", "async def "] {
                        if let Some(rest) = t.strip_prefix(kw) {
                            if let Some(id) = ident_at(rest, 0) {
                                stack.push((ind, id.to_string()));
                            }
                        }
                    }
                }
                out.push(stack.last().map(|(_, n)| n.clone()).unwrap_or_default());
            }
        }
        _ => out = vec![String::new(); text.lines().count()],
    }
    out
}

/// Keys of payloads built at runtime: `"key": value` in dict/map/object
/// literals, `x["key"] = value`, and for JS/TS also `key: value` object
/// members and `x.key = value`. Comments are not stripped (heuristic).
fn assembled_keys(text: &str, ext: &str) -> Vec<(usize, String)> {
    let js = matches!(ext, "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx");
    let mut out = Vec::new();
    for (ln, line) in text.lines().enumerate() {
        let b = line.as_bytes();
        let mut i = 0;
        while i < b.len() {
            let c = b[i];
            if c == b'"' || c == b'\'' {
                if let Some(len) = line[i + 1..].find(c as char) {
                    let key = &line[i + 1..i + 1 + len];
                    let after = line[i + 2 + len..].trim_start();
                    let is_dict_key = after.starts_with(':') && !after.starts_with("::");
                    let is_assign = i > 0
                        && b[i - 1] == b'['
                        && after.starts_with(']')
                        && after[1..].trim_start().starts_with('=')
                        && !after[1..].trim_start().starts_with("==");
                    if (is_dict_key || is_assign) && is_field_like(key) {
                        out.push((ln, key.to_string()));
                    }
                    i += len + 2;
                    continue;
                }
            }
            i += 1;
        }
        if js {
            let t = line.trim();
            if let Some(id) = ident_at(t, 0) {
                let rest = t[id.len()..].trim_start();
                if rest.starts_with(':') && !rest.starts_with("::") {
                    out.push((ln, id.to_string()));
                }
            }
            // { key: value, other: value } members after `{` or `,`
            for (pos, _) in line.match_indices(['{', ',']) {
                let rest = &line[pos + 1..];
                let off = rest.len() - rest.trim_start().len();
                if let Some(id) = ident_at(line, pos + 1 + off) {
                    let after = line[pos + 1 + off + id.len()..].trim_start();
                    if after.starts_with(':') && !after.starts_with("::") {
                        out.push((ln, id.to_string()));
                    }
                }
            }
            // x.key = value
            for (pos, _) in line.match_indices('.') {
                if pos > 0 && is_ident_byte(b[pos - 1]) {
                    if let Some(id) = ident_at(line, pos + 1) {
                        let after = line[pos + 1 + id.len()..].trim_start();
                        if after.starts_with('=')
                            && !after.starts_with("==")
                            && !after.starts_with("=>")
                        {
                            out.push((ln, id.to_string()));
                        }
                    }
                }
            }
        }
    }
    out
}

fn is_field_like(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| is_ident_byte(b) || b == b'-' || b == b'.')
        && !s.bytes().next().map(|b| b.is_ascii_digit()).unwrap_or(true)
}

/// Naming-form key: case and `_`/`-` separators removed, so `is_food`,
/// `isFood`, `IsFood` and `is-food` match (alpha.4).
pub fn name_key(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '_' && *c != '-')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Text of the first `key:"value"` in a Go struct tag.
fn quoted_after<'a>(s: &'a str, key: &str) -> Option<&'a str> {
    let i = s.find(key)? + key.len();
    let rest = &s[i..];
    let rest = rest.trim_start_matches([' ', '=', '(']).trim_start();
    let q = rest.chars().next()?;
    if q != '"' && q != '\'' {
        return None;
    }
    let body = &rest[1..];
    let end = body.find(q)?;
    Some(&body[..end])
}

fn go_fields(text: &str) -> Fields {
    let mut out = Fields::new();
    let mut in_struct = 0usize;
    let mut scope = String::new();
    for line in text.lines() {
        let t = line.trim();
        if t.contains("struct {") || t.ends_with("struct{") {
            if in_struct == 0 {
                // `type Name struct {` names the scope; anonymous structs
                // nested inside keep the outer type's scope.
                let head = t.strip_prefix("type ").unwrap_or(t);
                scope = ident_at(head, 0).unwrap_or("").to_string();
            }
            in_struct += 1;
            continue;
        }
        if in_struct > 0 && t.starts_with('}') {
            in_struct -= 1;
            continue;
        }
        if in_struct == 0 || t.is_empty() || t.starts_with("//") {
            continue;
        }
        let mut parts = t.split_whitespace();
        let (Some(name), Some(ty)) = (parts.next(), parts.next()) else {
            continue;
        };
        if !name
            .chars()
            .next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false)
            || ty.starts_with('`')
        {
            continue; // unexported, or an embedded type with only a tag
        }
        let tagged =
            quoted_after(t, "json:").map(|tag| tag.split(',').next().unwrap_or("").to_string());
        let key = match tagged.as_deref() {
            Some("-") => continue,
            Some("") | None => name.to_string(),
            Some(k) => k.to_string(),
        };
        out.insert((scope.clone(), key), Some(ty.to_string()));
    }
    out
}

/// The member texts after a declaration line's opening brace (the first `{`
/// outside `<...>`, `(...)` and `[...]`, so a type parameter's constraint or
/// default is never read as a member), up to its matching `}` or the end of
/// the line, split at `;` and `,` outside `{}`, `()` and `[]`. A comma inside
/// `<...>` may split a segment; the piece after it has no `name:` and is
/// dropped, and the type text is cut at that comma as for every member line,
/// so angle brackets are not tracked there. Empty when nothing follows the
/// brace on that line.
fn same_line_members(t: &str) -> Vec<&str> {
    let b = t.as_bytes();
    let mut d = 0i32;
    let mut open = None;
    for (i, &c) in b.iter().enumerate() {
        match c {
            b'<' | b'(' | b'[' => d += 1,
            b'>' if i > 0 && b[i - 1] == b'=' => {}
            b'>' | b')' | b']' => d -= 1,
            b'{' if d <= 0 => {
                open = Some(i + 1);
                break;
            }
            // LOCK-SLICE2: `= Readonly<{ ... }>`, the one wrapped body init
            // proposes, opens at its brace; nothing else inside `<...>` does.
            b'=' if d <= 0 && b.get(i + 1) != Some(&b'>') && b.get(i + 1) != Some(&b'=') => {
                let rest = t[i + 1..].trim_start();
                if let Some(r) = rest.strip_prefix("Readonly") {
                    if let Some(r) = r.trim_start().strip_prefix('<') {
                        if r.trim_start().starts_with('{') {
                            open = Some(b.len() - r.trim_start().len() + 1);
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    let Some(start) = open else { return Vec::new() };
    let mut out = Vec::new();
    let (mut d, mut seg) = (0i32, start);
    for i in start..b.len() {
        match b[i] {
            b'{' | b'(' | b'[' => d += 1,
            b'}' if d == 0 => {
                out.push(&t[seg..i]);
                return out;
            }
            b'}' | b')' | b']' => d -= 1,
            b';' | b',' if d == 0 => {
                out.push(&t[seg..i]);
                seg = i + 1;
            }
            _ => {}
        }
    }
    out.push(&t[seg..]);
    out
}

fn decl_name_type(t: &str) -> Option<(String, String)> {
    let t = t.strip_prefix("readonly ").unwrap_or(t);
    let name = ident_at(t, 0)?;
    let rest = t[name.len()..].trim_start_matches('?');
    let rest = rest.strip_prefix(':')?;
    if rest.starts_with(':') {
        return None; // `a::b` path, not a declaration
    }
    let ty = rest
        .split(['=', ';', ','])
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if ty.is_empty() {
        return None;
    }
    Some((name.to_string(), ty))
}

fn py_fields(text: &str) -> Fields {
    // Class attributes only at the class body's own indentation; method
    // bodies and multi-line signatures (parameters) are deeper and skipped.
    let mut out = Fields::new();
    // (indent, name, is_class, body indent once known)
    let mut stack: Vec<(usize, String, bool, Option<usize>)> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let ind = line.len() - line.trim_start().len();
        while stack.last().map(|e| e.0 >= ind).unwrap_or(false) {
            stack.pop();
        }
        if let Some(top) = stack.last_mut() {
            if top.3.is_none() {
                top.3 = Some(ind);
            }
        }
        let header = ["class ", "def ", "async def "]
            .iter()
            .find_map(|kw| t.strip_prefix(kw).map(|r| (*kw == "class ", r)));
        if let Some((is_class, rest)) = header {
            if let Some(id) = ident_at(rest, 0) {
                stack.push((ind, id.to_string(), is_class, None));
            }
            continue;
        }
        let Some((_, cls, true, Some(body))) = stack.last() else {
            continue;
        };
        if ind != *body || t.starts_with('@') {
            continue;
        }
        if let Some((name, ty)) = decl_name_type(t) {
            let key = quoted_after(t, "alias").map(str::to_string).unwrap_or(name);
            out.insert((cls.clone(), key), Some(ty));
        } else if let Some((name, callee)) = py_assigned_field(t) {
            // D1 rule 2: `name = models.CharField(...)`, `x = FooSerializer()`.
            out.insert((cls.clone(), name), Some(callee));
        }
    }
    out
}

fn ts_fields(text: &str) -> Fields {
    let mut out = Fields::new();
    let (mut scope, mut depth, mut open_at) = (String::new(), 0i64, -1i64);
    for line in text.lines() {
        let t = line
            .trim()
            .trim_start_matches("export ")
            .trim_start_matches("declare ");
        let mut opened = false;
        for kw in ["interface ", "type ", "class "] {
            if let Some(rest) = t.strip_prefix(kw) {
                if let Some(id) = ident_at(rest, 0) {
                    scope = id.to_string();
                    open_at = depth;
                    opened = true;
                }
            }
        }
        // LOCK-SLICE1: members written on the declaration's own line,
        // `type X = { a: A; readonly b: B }`.
        if opened {
            for seg in same_line_members(t) {
                if let Some((name, ty)) = decl_name_type(seg.trim()) {
                    out.insert((scope.clone(), name), Some(ty.trim_end_matches(['{', ' ']).to_string()));
                }
            }
        }
        if let Some((name, ty)) = decl_name_type(t) {
            out.insert(
                (scope.clone(), name),
                Some(ty.trim_end_matches(['{', ' ']).to_string()),
            );
        }
        depth += line.matches('{').count() as i64 - line.matches('}').count() as i64;
        if open_at >= 0 && depth <= open_at && line.contains('}') {
            scope.clear();
            open_at = -1;
        }
    }
    out
}

fn rs_fields(text: &str) -> Fields {
    let mut out = Fields::new();
    let mut pending_rename: Option<String> = None;
    let mut scope = String::new();
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("#[serde") {
            if let Some(r) = quoted_after(t, "rename") {
                pending_rename = Some(r.to_string());
            }
            continue;
        }
        let d = t.strip_prefix("pub ").unwrap_or(t);
        let d = d.strip_prefix("pub(crate) ").unwrap_or(d);
        if let Some(rest) = d.strip_prefix("struct ") {
            scope = ident_at(rest, 0).unwrap_or("").to_string();
            continue;
        }
        if let Some((name, ty)) = decl_name_type(d) {
            if t.ends_with(',') || t.ends_with('{') {
                out.insert(
                    (scope.clone(), pending_rename.take().unwrap_or(name)),
                    Some(ty),
                );
                continue;
            }
        }
        if !t.starts_with("#[") {
            pending_rename = None;
        }
    }
    out
}

/// Every field-like name a consumer source reads, with 1-based line numbers.
pub fn broad_reads(source: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    // CHECK-PERF-1: each key is searched once over the whole text, then its
    // matches are replayed line by line in the original order. A key holds
    // no newline, so every match lies inside one line.
    const KEYS: [&str; 4] = [".get(", "json:", "rename", "alias"];
    let hits: Vec<Vec<usize>> = KEYS.iter().map(|k| source.match_indices(k).map(|(i, _)| i).collect()).collect();
    let mut next = [0usize; 4];
    let base = source.as_ptr() as usize;
    for (ln, line) in source.lines().enumerate() {
        let start = line.as_ptr() as usize - base;
        let end = start + line.len();
        let line_no = ln + 1;
        let b = line.as_bytes();
        let mut push = |s: &str| {
            if !s.is_empty() {
                out.push((s.to_string(), line_no));
            }
        };
        for i in 0..b.len() {
            match b[i] {
                // x.field, x?.field, ).field, ].field
                b'.' if i > 0
                    && (is_ident_byte(b[i - 1]) || matches!(b[i - 1], b')' | b']' | b'?')) =>
                {
                    if let Some(id) = ident_at(line, i + 1) {
                        push(id);
                    }
                }
                // x["field"], x['field']
                b'[' if i + 1 < b.len() && (b[i + 1] == b'"' || b[i + 1] == b'\'') => {
                    let q = b[i + 1];
                    if let Some(end) = line[i + 2..].find(q as char) {
                        let inner = &line[i + 2..i + 2 + end];
                        if line[i + 2 + end + 1..].starts_with(']') {
                            push(inner);
                        }
                    }
                }
                _ => {}
            }
        }
        for (k, key) in KEYS.iter().enumerate() {
            while next[k] < hits[k].len() && hits[k][next[k]] + key.len() <= end {
                let at = hits[k][next[k]] - start;
                if let Some(v) = quoted_after(&line[at..], key) {
                    push(v.split(',').next().unwrap_or(""));
                }
                next[k] += 1;
            }
        }
        if let Some((name, _)) = decl_name_type(line.trim().trim_start_matches("pub ")) {
            push(&name);
        }
    }
    out
}

/// What changed between two producer versions: (field, type change if any).
/// A field is removed when its name exists in no scope of `head`; a type
/// change is reported only within the same scope. Each name appears once.
pub fn changed_fields(base: &Fields, head: &Fields) -> Vec<(String, Option<(String, String)>)> {
    let head_names = field_names(head);
    let mut out: Vec<(String, Option<(String, String)>)> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for ((scope, name), old_ty) in base {
        let change = match head.get(&(scope.clone(), name.clone())) {
            Some(new_ty) => match (old_ty, new_ty) {
                (Some(a), Some(b)) if a != b => Some(Some((a.clone(), b.clone()))),
                _ => None,
            },
            None if !head_names.contains(name) => Some(None),
            None => None,
        };
        if let Some(c) = change {
            if seen.insert(name.clone()) {
                out.push((name.clone(), c));
            }
        }
    }
    out
}

/// A1 rule 2: fields that left their type but still exist in another type:
/// (name, from scope, to scopes). Unscoped (spec) fields cannot move.
pub fn moved_fields(base: &Fields, head: &Fields) -> Vec<(String, String, Vec<String>)> {
    let mut out: Vec<(String, String, Vec<String>)> = Vec::new();
    for (scope, name) in base.keys() {
        if scope.is_empty() || head.contains_key(&(scope.clone(), name.clone())) {
            continue;
        }
        let to: Vec<String> = head
            .keys()
            .filter(|(s, n)| n == name && s != scope)
            .map(|(s, _)| if s.is_empty() { "(unscoped)".to_string() } else { s.clone() })
            .collect();
        if !to.is_empty() && !out.iter().any(|(n, _, _)| n == name) {
            out.push((name.clone(), scope.clone(), to));
        }
    }
    out
}

/// A1 rule 1: 1-based lines where `name` appears in the consumer as a whole
/// identifier or as a quoted string, compared by naming form.
pub fn mention_lines(source: &str, name: &str) -> Vec<usize> {
    let key = name_key(name);
    let mut out = Vec::new();
    for (ln, line) in source.lines().enumerate() {
        let b = line.as_bytes();
        let mut hit = false;
        let mut i = 0;
        while i < b.len() && !hit {
            let c = b[i];
            if c == b'"' || c == b'\'' || c == b'`' {
                if let Some(len) = line[i + 1..].find(c as char) {
                    hit = name_key(&line[i + 1..i + 1 + len]) == key;
                    i += len + 2;
                    continue;
                }
            }
            if c.is_ascii_alphabetic() || c == b'_' {
                let start = i;
                while i < b.len() && is_ident_byte(b[i]) {
                    i += 1;
                }
                hit = name_key(&line[start..i]) == key;
                continue;
            }
            i += 1;
        }
        if hit {
            out.push(ln + 1);
        }
    }
    out
}

/// D1 rule 2: a Python class-body `name = Callee(...)` whose callee ends in
/// Field, Serializer, Column or ForeignKey; returns (name, callee's last
/// segment). ALL_CAPS names and dunders are not fields.
fn py_assigned_field(t: &str) -> Option<(String, String)> {
    let name = ident_at(t, 0)?;
    if name.starts_with("__") || !name.bytes().any(|b| b.is_ascii_lowercase()) {
        return None;
    }
    let rest = t[name.len()..].trim_start().strip_prefix('=')?;
    if rest.starts_with('=') {
        return None;
    }
    let rest = rest.trim_start();
    let end = rest.find('(')?;
    let callee = &rest[..end];
    if callee.is_empty() || !callee.bytes().all(|b| is_ident_byte(b) || b == b'.') {
        return None;
    }
    let last = callee.rsplit('.').next().unwrap_or(callee);
    ["Field", "Serializer", "Column", "ForeignKey"]
        .iter()
        .any(|s| last.ends_with(s))
        .then(|| (name.to_string(), last.to_string()))
}

/// D1 rule 1: TypeScript `type T = Omit<..., "a" | "b">` (possibly nested or
/// spread over lines): (T, omitted fields) for each such alias.
pub fn ts_omissions(text: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim().trim_start_matches("export ").trim_start_matches("declare ");
        let Some(rest) = t.strip_prefix("type ") else {
            i += 1;
            continue;
        };
        let Some(name) = ident_at(rest, 0) else {
            i += 1;
            continue;
        };
        // The statement runs to the first line ending in ';' (or 40 lines).
        let mut stmt = String::new();
        let mut j = i;
        while j < lines.len() && j < i + 40 {
            stmt.push_str(lines[j]);
            stmt.push(' ');
            if lines[j].trim_end().ends_with(';') {
                break;
            }
            j += 1;
        }
        if let Some(eq) = stmt.find('=') {
            if let Some(pos) = stmt[eq..].find("Omit<") {
                let body = &stmt[eq + pos + 5..];
                let (mut depth, mut last_comma, mut end) = (1i32, None, None);
                for (k, c) in body.char_indices() {
                    match c {
                        '<' => depth += 1,
                        '>' if k > 0 && body.as_bytes()[k - 1] == b'=' => {}
                        '>' => {
                            depth -= 1;
                            if depth == 0 {
                                end = Some(k);
                                break;
                            }
                        }
                        ',' if depth == 1 => last_comma = Some(k),
                        _ => {}
                    }
                }
                if let (Some(c), Some(e)) = (last_comma, end) {
                    let keys: Vec<String> = body[c + 1..e]
                        .split('|')
                        .filter_map(|k| {
                            let k = k.trim();
                            let q = k.chars().next()?;
                            (q == '"' || q == '\'').then(|| k.trim_matches(q).to_string())
                        })
                        .filter(|k| !k.is_empty())
                        .collect();
                    if !keys.is_empty() {
                        out.push((name.to_string(), keys));
                    }
                }
            }
        }
        i = j.max(i) + 1;
    }
    out
}

/// E1 rule 1: keys of dict/object/map literals with the path of literal keys
/// that enclose them, e.g. ("log_message", "message"). A container opened
/// under a non-literal key contributes "*". Python, JS/TS and Go only.
pub fn key_paths(text: &str, ext: &str) -> Vec<(String, String)> {
    let js = matches!(ext, "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx");
    let py = ext == "py";
    if !(js || py || ext == "go") {
        return Vec::new();
    }
    let mut out = Vec::new();
    // (opener, segment)
    let mut stack: Vec<(u8, String)> = Vec::new();
    let mut pending: Option<String> = None;
    let path = |st: &Vec<(u8, String)>| {
        st.iter()
            .map(|(_, s)| s.as_str())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(".")
    };
    for line in text.lines() {
        let b = line.as_bytes();
        let mut i = 0;
        while i < b.len() {
            let c = b[i];
            if (py && c == b'#') || ((js || ext == "go") && c == b'/' && b.get(i + 1) == Some(&b'/')) {
                break;
            }
            let in_obj = stack.last().map(|(o, _)| *o == b'{').unwrap_or(false);
            let after_colon = |j: usize| {
                let r = line[j..].trim_start();
                r.starts_with(':') && !r.starts_with("::")
            };
            if c == b'"' || c == b'\'' || c == b'`' {
                let Some(len) = line[i + 1..].find(c as char) else { break };
                let s = &line[i + 1..i + 1 + len];
                let end = i + 2 + len;
                if in_obj && is_field_like(s) && after_colon(end) {
                    out.push((path(&stack), s.to_string()));
                    pending = Some(s.to_string());
                } else {
                    pending = None;
                }
                i = end;
                continue;
            }
            if c.is_ascii_alphabetic() || c == b'_' || c == b'$' {
                let start = i;
                while i < b.len() && (is_ident_byte(b[i]) || b[i] == b'$') {
                    i += 1;
                }
                let id = &line[start..i];
                if in_obj && after_colon(i) {
                    if js {
                        out.push((path(&stack), id.to_string()));
                        pending = Some(id.to_string());
                    } else {
                        pending = Some("*".into());
                    }
                }
                continue;
            }
            match c {
                b'[' if js && in_obj && line[i..].contains("]:") && pending.is_none() => {
                    // computed key `[k]: {`
                    if let Some(e) = line[i..].find("]:") {
                        i += e + 1;
                        pending = Some("*".into());
                        continue;
                    }
                }
                b'[' if pending.is_some() && {
                    let r = line[i + 1..].trim_start();
                    r.starts_with(']') || r.split(']').next().map(|t| t.bytes().all(|x| is_ident_byte(x) || x == b'.')).unwrap_or(false) && r.contains(']') && !r.starts_with('{')
                } && ext == "go" =>
                {
                    // Go type syntax like map[string]any or []T: not a container.
                    if let Some(e) = line[i..].find(']') {
                        i += e + 1;
                        continue;
                    }
                }
                b'{' | b'[' | b'(' => {
                    let seg = if c == b'(' { String::new() } else { pending.take().unwrap_or_default() };
                    stack.push((c, seg));
                }
                b'}' | b']' | b')' => {
                    let open = match c { b'}' => b'{', b']' => b'[', _ => b'(' };
                    if let Some(pos) = stack.iter().rposition(|(o, _)| *o == open) {
                        stack.truncate(pos);
                    }
                    pending = None;
                }
                b',' | b';' | b'=' => pending = None,
                _ => {}
            }
            i += 1;
        }
    }
    out
}

/// E1 rule 2: keys whose head paths share none of their base paths and sit
/// deeper at head: (key, base path, head path, new path segments).
pub fn relocated_keys(base: &str, head: &str, ext: &str) -> Vec<(String, String, String, Vec<String>)> {
    use std::collections::{BTreeMap, BTreeSet};
    let index = |t: &str| {
        let mut m: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (p, k) in key_paths(t, ext) {
            m.entry(k).or_default().insert(p);
        }
        m
    };
    let depth = |p: &str| if p.is_empty() { 0 } else { p.split('.').count() };
    let (b, h) = (index(base), index(head));
    let mut out = Vec::new();
    for (k, bp) in &b {
        let Some(hp) = h.get(k) else { continue };
        if !bp.is_disjoint(hp) {
            continue;
        }
        let bmin = bp.iter().min_by_key(|p| depth(p)).cloned().unwrap_or_default();
        let hmin = hp.iter().min_by_key(|p| depth(p)).cloned().unwrap_or_default();
        if depth(&hmin) <= depth(&bmin) {
            continue;
        }
        let bsegs: BTreeSet<&str> = bmin.split('.').collect();
        let new: Vec<String> = hmin
            .split('.')
            .filter(|s| !s.is_empty() && !bsegs.contains(s))
            .map(str::to_string)
            .collect();
        out.push((k.clone(), bmin, hmin, new));
    }
    out
}

/// F1: the value literal at the start of `s`, normalized: an integer, `true`,
/// `false`, or a quoted integer / `v` + integer (quotes and the `v` stripped).
fn selector_value(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let first = *b.first()?;
    if first == b'"' || first == b'\'' {
        let q = first as char;
        let end = s[1..].find(q)?;
        let inner = &s[1..1 + end];
        let digits = inner.strip_prefix(['v', 'V']).unwrap_or(inner);
        return (!digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit()))
            .then(|| digits.to_string());
    }
    if first.is_ascii_digit() {
        let n = b.iter().take_while(|c| c.is_ascii_digit()).count();
        let next = b.get(n);
        return next
            .is_none_or(|&c| !(c.is_ascii_alphanumeric() || c == b'_' || c == b'.'))
            .then(|| s[..n].to_string());
    }
    for word in ["true", "false"] {
        if s.starts_with(word)
            && b.get(word.len())
                .is_none_or(|&c| !(c.is_ascii_alphanumeric() || c == b'_'))
        {
            return Some(word.to_string());
        }
    }
    None
}

/// F1 selector extraction (spec docs/falsification/F1-SPEC.md): per line,
/// (name, value) pairs where the name contains `version` or `format`
/// (case-insensitive), by four textual patterns: assignment (`name = v`,
/// `name: v`, `name: Type = v`), `defaultValue` on the same line, a call
/// argument (`name(v`, version names only), and a quoted key (`"name", v` or
/// `"name": v`). G3 (docs/falsification/G3-SPEC.md) adds P5, variant
/// literals keyed by their type, and P6, default-site occurrences keyed by
/// trigger and type. Lines opening as comments are ignored. Returns
/// lowercased name -> set of normalized values.
pub fn default_selectors(text: &str) -> BTreeMap<String, BTreeSet<String>> {
    let is_sel = |n: &str| {
        let l = n.to_ascii_lowercase();
        l.contains("version") || l.contains("format")
    };
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for raw in text.lines() {
        let t = raw.trim_start();
        if t.starts_with("//") || t.starts_with('#') || t.starts_with('*') || t.starts_with("/*")
        {
            continue;
        }
        let b = raw.as_bytes();
        let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
        let mut push = |name: &str, v: Option<String>| {
            if let Some(v) = v {
                out.entry(name.to_ascii_lowercase()).or_default().insert(v);
            }
        };
        let mut i = 0;
        while i < raw.len() {
            let c = b[i];
            if c == b'"' || c == b'\'' {
                // Quoted key: `"name", v` or `"name": v`.
                let Some(end) = raw[i + 1..].find(c as char) else {
                    i += 1;
                    continue;
                };
                let key = &raw[i + 1..i + 1 + end];
                if is_sel(key) && key.bytes().all(ident) {
                    let rest = raw[i + 2 + end..].trim_start();
                    if let Some(r) = rest.strip_prefix([',', ':']) {
                        push(key, selector_value(r.trim_start()));
                    }
                }
                i += 2 + end;
                continue;
            }
            if ident(c) && !c.is_ascii_digit() && (i == 0 || !ident(b[i - 1])) {
                let mut j = i;
                while j < raw.len() && ident(b[j]) {
                    j += 1;
                }
                let name = &raw[i..j];
                if is_sel(name) {
                    let rest = &raw[j..];
                    // Call argument, version names only (`.format(` is string
                    // formatting in half the corpus's languages).
                    if name.to_ascii_lowercase().contains("version") {
                        if let Some(r) = rest.strip_prefix('(') {
                            push(name, selector_value(r.trim_start()));
                        }
                    }
                    // `defaultValue: v` / `defaultValue = v` on the same line.
                    if let Some(p) = rest.find("defaultValue") {
                        let r = rest[p + "defaultValue".len()..].trim_start();
                        if let Some(r) = r.strip_prefix([':', '=']) {
                            push(name, selector_value(r.trim_start()));
                        }
                    }
                    // Assignment: `= v`, `: v`, or `: Type = v` (never `==`).
                    let r = rest.trim_start();
                    if let Some(r1) = r.strip_prefix('=') {
                        if !r1.starts_with(['=', '>']) {
                            push(name, selector_value(r1.trim_start()));
                        }
                    } else if let Some(r1) = r.strip_prefix(':') {
                        let rt = r1.trim_start();
                        if selector_value(rt).is_some() {
                            push(name, selector_value(rt));
                        } else if let Some(ep) = r1.find('=') {
                            let after = &r1[ep + 1..];
                            if !after.starts_with(['=', '>'])
                                && !r1[..ep].contains([';', ',', '(', ')'])
                            {
                                push(name, selector_value(after.trim_start()));
                            }
                        }
                    }
                }
                i = j;
                continue;
            }
            i += 1;
        }
        // G3 P5 (docs/falsification/G3-SPEC.md): every V5 variant literal
        // on a non-comment line is an occurrence of its type.
        for (ty, v) in variant_literals(raw) {
            push(&ty, Some(v));
        }
    }
    // G3 P6: default-site occurrences, keyed by the trigger identifier and
    // the type (A-1); the second map counts trigger lines per key so that
    // indistinguishable same-named sites are disclosed, never silently merged.
    let (sites, _) = default_sites(text);
    for (key, v) in sites {
        out.entry(key).or_default().insert(v);
    }
    out
}

/// A line opening as a comment (`//`, `#`, `*`, `/*`): F1's skip rule, which
/// G3 shares unchanged.
fn is_comment_line(raw: &str) -> bool {
    let t = raw.trim_start();
    t.starts_with("//") || t.starts_with('#') || t.starts_with('*') || t.starts_with("/*")
}

/// G3 V5: every variant literal on `line`, in order, as (lowercased type
/// identifier, digits). A variant literal is `TypeIdent::Variant` or
/// `TypeIdent.Variant` (A-2), both whole identifiers recognized token by
/// token (never by substring), TypeIdent containing `version` or `format`
/// (case-insensitive), Variant exactly `V`/`v` + digits or digits alone.
/// Comment lines yield nothing.
fn variant_literals(line: &str) -> Vec<(String, String)> {
    if is_comment_line(line) {
        return Vec::new();
    }
    let b = line.as_bytes();
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if !(ident(b[i]) && !b[i].is_ascii_digit() && (i == 0 || !ident(b[i - 1]))) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < b.len() && ident(b[j]) {
            j += 1;
        }
        let ty = &line[i..j];
        let l = ty.to_ascii_lowercase();
        if l.contains("version") || l.contains("format") {
            let sep = if line[j..].starts_with("::") {
                2
            } else if line[j..].starts_with('.') {
                1
            } else {
                0
            };
            if sep > 0 {
                let k = j + sep;
                let mut m = k;
                while m < b.len() && ident(b[m]) {
                    m += 1;
                }
                let var = &line[k..m];
                let digits = var.strip_prefix(['V', 'v']).unwrap_or(var);
                if !digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit()) {
                    out.push((l, digits.to_string()));
                }
            }
        }
        i = j;
    }
    out
}

/// G3 P6: (key, value) per default-site occurrence, and the number of
/// trigger lines per key. A trigger is a non-comment line holding a whole
/// identifier containing `default`; the first such identifier names the
/// site. Its window is the trigger line plus up to two following
/// non-comment lines, extended one line at a time only while the previous
/// line of the chain ends with a continuation token, so a completed
/// statement never lends its neighbor a value. The first V5 literal in the
/// window is the occurrence; key = "<trigger> <type>", both lowercased.
fn default_sites(text: &str) -> (Vec<(String, String)>, BTreeMap<String, usize>) {
    const CONT: [&str; 7] = ["{", "(", ":", "=", ",", "->", "=>"];
    let lines: Vec<&str> = text.lines().collect();
    let ends_cont = |l: &str| {
        let t = l.trim_end();
        CONT.iter().any(|c| t.ends_with(c))
    };
    let mut occ = Vec::new();
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for (idx, raw) in lines.iter().enumerate() {
        if is_comment_line(raw) {
            continue;
        }
        let Some(trigger) = default_identifier(raw) else { continue };
        let mut window = vec![*raw];
        let mut prev = *raw;
        let mut j = idx + 1;
        while window.len() < 3 && ends_cont(prev) {
            while j < lines.len() && is_comment_line(lines[j]) {
                j += 1;
            }
            if j >= lines.len() {
                break;
            }
            window.push(lines[j]);
            prev = lines[j];
            j += 1;
        }
        if let Some((ty, v)) = window.iter().find_map(|l| variant_literals(l).into_iter().next()) {
            let key = format!("{} {ty}", trigger.to_ascii_lowercase());
            *count.entry(key.clone()).or_default() += 1;
            occ.push((key, v));
        }
    }
    (occ, count)
}

/// The first whole identifier on the line containing `default`
/// (case-insensitive).
fn default_identifier(line: &str) -> Option<&str> {
    let b = line.as_bytes();
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut i = 0;
    while i < b.len() {
        if ident(b[i]) && (i == 0 || !ident(b[i - 1])) {
            let mut j = i;
            while j < b.len() && ident(b[j]) {
                j += 1;
            }
            let w = &line[i..j];
            if !b[i].is_ascii_digit() && w.to_ascii_lowercase().contains("default") {
                return Some(w);
            }
            i = j;
            continue;
        }
        i += 1;
    }
    None
}

/// F1: selectors whose head value set gained a value absent at base:
/// (name, base values, new values, disclosure). Names on only one side never
/// count. The disclosure is empty except for a G3 P6 key that two or more
/// trigger lines share on either side: those sites cannot be told apart, so
/// the merge is stated (G3-SPEC r2, A-1), with k the larger side's count.
pub fn selector_switches(base: &str, head: &str) -> Vec<(String, String, String, String)> {
    let b = default_selectors(base);
    let h = default_selectors(head);
    let (_, bc) = default_sites(base);
    let (_, hc) = default_sites(head);
    let mut out = Vec::new();
    for (name, hv) in &h {
        let Some(bv) = b.get(name) else { continue };
        let new: Vec<String> = hv.difference(bv).cloned().collect();
        if !new.is_empty() {
            let old: Vec<String> = bv.iter().cloned().collect();
            let k = bc.get(name).copied().unwrap_or(0).max(hc.get(name).copied().unwrap_or(0));
            let note = if k >= 2 {
                format!(" ({k} same-named default sites; site attribution not established)")
            } else {
                String::new()
            };
            out.push((name.clone(), old.join("|"), new.join("|"), note));
        }
    }
    out
}

// ---------------------------------------------------------------- request side
// REQUEST-SIDE-SPEC r1, sections B1, B2, B3. Hand-written grammars; every key
// is a whole string literal (or, for the B2 JavaScript member form, a whole
// identifier), never a substring. Comment lines are skipped (shared rule).

/// String literals on one line, left to right: (open quote index, index after
/// the close quote, content). A quote opens a literal that closes at the next
/// unescaped matching quote; an unterminated literal ends the scan.
fn string_literals(line: &str) -> Vec<(usize, usize, &str)> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let q = b[i];
        if q == b'"' || q == b'\'' || q == b'`' {
            let mut j = i + 1;
            while j < b.len() && b[j] != q {
                j += if b[j] == b'\\' { 2 } else { 1 };
            }
            if j >= b.len() {
                break;
            }
            out.push((i, j + 1, &line[i + 1..j]));
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// B1 key shape: `[A-Za-z_][A-Za-z0-9_-]*`.
fn is_request_key(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && (b[0].is_ascii_alphabetic() || b[0] == b'_')
        && b.iter().all(|&c| is_ident_byte(c) || c == b'-')
}

/// `=` at the start of `rest` (after optional spaces) that is not `==`.
fn assigns(rest: &str) -> bool {
    let r = rest.trim_start();
    r.starts_with('=') && !r.starts_with("==")
}

/// `word` ends `prefix` as a whole word (the byte before it is not an
/// identifier byte).
fn ends_with_word(prefix: &str, word: &str) -> bool {
    prefix.ends_with(word) && {
        let k = prefix.len() - word.len();
        let lead = word.as_bytes()[0];
        k == 0 || !is_ident_byte(lead) || !is_ident_byte(prefix.as_bytes()[k - 1])
    }
}

/// B1: consumer writes, key -> 1-based lines.
pub fn consumer_writes(text: &str) -> BTreeMap<String, Vec<usize>> {
    const SETTERS: [&str; 5] = [".Add(", ".Set(", ".WriteField(", ".SetFormData(", ".SetQueryParam("];
    let mut out: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (ln, line) in text.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }
        for (s, e, k) in string_literals(line) {
            if !is_request_key(k) {
                continue;
            }
            let (pre, post) = (&line[..s], &line[e..]);
            let subscript = pre.ends_with('[')
                && pre.len() >= 2
                && matches!(pre.as_bytes()[pre.len() - 2], c if is_ident_byte(c) || c == b']' || c == b')')
                && post.starts_with(']')
                && assigns(&post[1..]);
            let dict_key = post.starts_with(':') && !post.starts_with("::") && !post[1..].trim().is_empty();
            let setter = SETTERS.iter().any(|w| pre.ends_with(w));
            if subscript || dict_key || setter {
                let v = out.entry(k.to_string()).or_default();
                if v.last() != Some(&(ln + 1)) {
                    v.push(ln + 1);
                }
            }
        }
    }
    out
}

/// B2: producer inbound reads, key -> 1-based lines.
pub fn inbound_reads(text: &str) -> BTreeMap<String, Vec<usize>> {
    const GO_ACCESSORS: [&str; 8] = [
        ".FormValue(", ".PostFormValue(", ".PostForm(", ".Query(", ".QueryParam(", ".FormFile(",
        ".DefaultPostForm(", ".DefaultQuery(",
    ];
    const GO_BUILDERS: [&str; 11] = [
        "String(", "Bool(", "Int(", "Int64(", "Float64(", "Duration(", "Strings(", "Content(",
        "MandatoryString(", "MandatoryContent(", "Custom(",
    ];
    const PY_ATTRS: [&str; 7] = ["data", "POST", "GET", "args", "form", "json", "query_params"];
    const JS_OBJS: [&str; 3] = ["req.body", "req.query", "req.params"];
    let mut out: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut add = |k: &str, ln: usize| {
        let v = out.entry(k.to_string()).or_default();
        if v.last() != Some(&ln) {
            v.push(ln);
        }
    };
    for (ln, line) in text.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }
        for (s, _, k) in string_literals(line) {
            if !is_request_key(k) {
                continue;
            }
            let pre = &line[..s];
            let go = GO_ACCESSORS.iter().any(|w| pre.ends_with(w))
                || GO_BUILDERS.iter().any(|w| ends_with_word(pre, w));
            let py = PY_ATTRS.iter().any(|a| {
                ends_with_word(pre, &format!("request.{a}["))
                    || ends_with_word(pre, &format!("request.{a}.get("))
            });
            let js = JS_OBJS
                .iter()
                .any(|o| ends_with_word(pre, &format!("{o}[")) || ends_with_word(pre, &format!("{o}(")));
            if go || py || js {
                add(k, ln + 1);
            }
        }
        for o in JS_OBJS {
            let pat = format!("{o}.");
            let mut from = 0;
            while let Some(p) = line[from..].find(&pat) {
                let at = from + p;
                if at == 0 || !is_ident_byte(line.as_bytes()[at - 1]) {
                    if let Some(k) = ident_at(line, at + pat.len()) {
                        add(k, ln + 1);
                    }
                }
                from = at + pat.len();
            }
        }
    }
    out
}

/// RS-1 condition 3: `k` is a whole string literal on a non-comment line.
pub fn has_string_literal(text: &str, k: &str) -> bool {
    text.lines()
        .filter(|l| !is_comment_line(l))
        .any(|l| string_literals(l).iter().any(|(_, _, c)| *c == k))
}

/// B3: serializer fields `name = BareIdent(args)`, name -> (read_only, line).
pub fn serializer_fields(text: &str) -> BTreeMap<String, (bool, usize)> {
    let mut out = BTreeMap::new();
    for (ln, line) in text.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }
        let t = line.trim_start();
        let Some(name) = ident_at(t, 0) else { continue };
        let rest = &t[name.len()..];
        if !assigns(rest) {
            continue;
        }
        let rhs = rest.trim_start()[1..].trim_start();
        let Some(callee) = ident_at(rhs, 0) else { continue };
        let Some(args) = rhs[callee.len()..].strip_prefix('(') else { continue };
        let args: String = args[..args.find(')').unwrap_or(args.len())].chars().filter(|c| *c != ' ').collect();
        out.entry(name.to_string()).or_insert((args.contains("read_only=True"), ln + 1));
    }
    out
}

/// RS-2: 1-based non-comment lines where `name` occurs as a whole word.
pub fn word_lines(text: &str, name: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for (ln, line) in text.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }
        let b = line.as_bytes();
        let hit = line.match_indices(name).any(|(at, _)| {
            let end = at + name.len();
            (at == 0 || !is_ident_byte(b[at - 1])) && (end >= b.len() || !is_ident_byte(b[end]))
        });
        if hit {
            out.push(ln + 1);
        }
    }
    out
}

/// RS-0: the first non-comment line holding an opaque payload construct:
/// `x[k] =`, `**name`, `...name`, whole-word `json=name` or `data=name`,
/// `SetBody(name` or `SetBody(&name`.
pub fn opaque_payload(text: &str) -> Option<usize> {
    for (ln, line) in text.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }
        let b = line.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if !line.is_char_boundary(i) {
                i += 1;
                continue;
            }
            let rest = &line[i..];
            let prev = if i == 0 { None } else { Some(b[i - 1]) };
            let after_ident = |off: usize| ident_at(line, i + off).is_some();
            let subscript = b[i] == b'['
                && matches!(prev, Some(c) if is_ident_byte(c) || c == b']' || c == b')')
                && ident_at(line, i + 1).is_some_and(|k| {
                    line[i + 1 + k.len()..].starts_with(']') && assigns(&line[i + 2 + k.len()..])
                });
            let kwargs = rest.starts_with("**")
                && !matches!(prev, Some(c) if is_ident_byte(c) || c == b')' || c == b']' || c == b'*')
                && after_ident(2);
            let spread = rest.starts_with("...") && after_ident(3);
            let kw = ["json", "data"].iter().any(|w| {
                rest.starts_with(w)
                    && !matches!(prev, Some(c) if is_ident_byte(c))
                    && assigns(&rest[w.len()..])
                    && {
                        let r = rest[w.len()..].trim_start()[1..].trim_start();
                        ident_at(r, 0).is_some()
                    }
            });
            let set_body = rest.starts_with("SetBody(")
                && (after_ident(8) || (rest[8..].starts_with('&') && after_ident(9)));
            if subscript || kwargs || spread || kw || set_body {
                return Some(ln + 1);
            }
            i += 1;
        }
    }
    None
}

// ----------------------------------------------------------- G1-E emissions
// G1-E-SPEC r0, section B. An emission site is one of the seven frozen call
// forms on a non-comment line; its payload is classified as a literal object
// (with its depth-one keys) or a single opaque identifier, else not counted.

/// One emission site: the form, its 1-based line, and the payload class.
pub struct Emission {
    pub form: &'static str,
    pub line: usize,
    /// `Ok(keys)` for a literal object, `Err(ident)` for an opaque payload.
    pub payload: Result<Vec<String>, String>,
}

pub const EMISSION_FORMS: [&str; 7] =
    [".json(", ".send(", "jsonify(", "JsonResponse(", "json.dumps(", "Response(", "send_result("];

/// The quoted literal starting at `b[i]` (a quote): index after its close
/// quote, or None when unterminated.
fn skip_quoted(b: &[u8], i: usize) -> Option<usize> {
    let q = b[i];
    let mut j = i + 1;
    while j < b.len() && b[j] != q {
        j += if b[j] == b'\\' { 2 } else { 1 };
    }
    (j < b.len()).then_some(j + 1)
}

/// E2: the top-level arguments of a call whose `(` ends just before
/// `text[start..]`, scanned to the matching `)` across lines.
fn call_args(text: &str, start: usize) -> Option<Vec<&str>> {
    let b = text.as_bytes();
    let (mut depth, mut from, mut j) = (0usize, start, start);
    let mut out = Vec::new();
    while j < b.len() {
        match b[j] {
            b'"' | b'\'' | b'`' => {
                j = skip_quoted(b, j)?;
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' if depth == 0 => {
                out.push(&text[from..j]);
                return Some(out);
            }
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                out.push(&text[from..j]);
                from = j + 1;
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// E3: depth-one keys of an object literal `{...}`.
fn literal_keys(obj: &str) -> Vec<String> {
    let body = &obj[1..];
    let b = body.as_bytes();
    let (mut depth, mut entry_start, mut j) = (0usize, true, 0usize);
    let mut keys = Vec::new();
    let colon_after = |k: usize| {
        let r = body[k..].trim_start();
        r.starts_with(':') && !r.starts_with("::")
    };
    while j < b.len() {
        let c = b[j];
        if c == b'"' || c == b'\'' || c == b'`' {
            let Some(end) = skip_quoted(b, j) else { break };
            if depth == 0 && entry_start && colon_after(end) {
                keys.push(body[j + 1..end - 1].to_string());
            }
            entry_start = false;
            j = end;
            continue;
        }
        if depth == 0 && entry_start && (c.is_ascii_alphabetic() || c == b'_') {
            let id = ident_at(body, j).unwrap_or_default();
            if colon_after(j + id.len()) {
                keys.push(id.to_string());
            }
            entry_start = false;
            j += id.len();
            continue;
        }
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                entry_start = true;
                j += 1;
                continue;
            }
            _ => {}
        }
        if !c.is_ascii_whitespace() {
            entry_start = false;
        }
        j += 1;
    }
    keys
}

/// E1 to E3: every classified emission site in `text`, in order.
pub fn emissions(text: &str) -> Vec<Emission> {
    const KEYWORDS: [&str; 7] = ["null", "None", "true", "false", "True", "False", "undefined"];
    let mut out = Vec::new();
    let mut offset = 0;
    for (ln, line) in text.split('\n').enumerate() {
        let here = offset;
        offset += line.len() + 1;
        if is_comment_line(line) {
            continue;
        }
        for form in EMISSION_FORMS {
            for (at, _) in line.match_indices(form) {
                let whole = form.starts_with('.') || at == 0 || !is_ident_byte(line.as_bytes()[at - 1]);
                if !whole {
                    continue;
                }
                let Some(args) = call_args(text, here + at + form.len()) else { continue };
                let idx = usize::from(form == "send_result(");
                let Some(arg) = args.get(idx).map(|a| a.trim()) else { continue };
                let payload = if arg.starts_with('{') {
                    Ok(literal_keys(arg))
                } else if ident_at(arg, 0).is_some_and(|id| id.len() == arg.len()) && !KEYWORDS.contains(&arg) {
                    Err(arg.to_string())
                } else {
                    continue;
                };
                out.push(Emission { form, line: ln + 1, payload });
            }
        }
    }
    out
}
