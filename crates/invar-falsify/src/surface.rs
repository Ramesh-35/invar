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

use std::collections::BTreeMap;

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
        for kw in ["interface ", "type ", "class "] {
            if let Some(rest) = t.strip_prefix(kw) {
                if let Some(id) = ident_at(rest, 0) {
                    scope = id.to_string();
                    open_at = depth;
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
    for (ln, line) in source.lines().enumerate() {
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
        for key in [".get(", "json:", "rename", "alias"] {
            let mut from = 0;
            while let Some(pos) = line[from..].find(key) {
                let at = from + pos;
                if let Some(v) = quoted_after(&line[at..], key) {
                    push(v.split(',').next().unwrap_or(""));
                }
                from = at + key.len();
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
