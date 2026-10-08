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
//! HONEST SCOPE: pattern matching, not parsing. Field sets are file-wide (not
//! per type), nested paths are flattened to names, and names are matched
//! exactly (no case or naming-convention folding).

use std::collections::BTreeMap;

use serde_json::Value;

/// Field name -> declared type, if known.
pub type Fields = BTreeMap<String, Option<String>>;

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
            out.insert(name.clone(), spec_type(p));
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
    if base == "array" {
        if let Some(items) = p.get("items").and_then(spec_type) {
            return Some(format!("array<{items}>"));
        }
    }
    Some(base)
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
    // Payloads assembled at runtime (alpha.4): dict/map/object literal keys
    // and keyed assignments, for Python, JS/TS and Go. Untyped.
    if matches!(
        ext,
        "py" | "go" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs"
    ) {
        for key in assembled_keys(text, ext) {
            f.entry(key).or_insert(None);
        }
    }
    Some(f)
}

/// Keys of payloads built at runtime: `"key": value` in dict/map/object
/// literals, `x["key"] = value`, and for JS/TS also `key: value` object
/// members and `x.key = value`. Comments are not stripped (heuristic).
fn assembled_keys(text: &str, ext: &str) -> Vec<String> {
    let js = matches!(ext, "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx");
    let mut out = Vec::new();
    for line in text.lines() {
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
                        out.push(key.to_string());
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
                    out.push(id.to_string());
                }
            }
            // { key: value, other: value } members after `{` or `,`
            for (pos, _) in line.match_indices(['{', ',']) {
                let rest = &line[pos + 1..];
                let off = rest.len() - rest.trim_start().len();
                if let Some(id) = ident_at(line, pos + 1 + off) {
                    let after = line[pos + 1 + off + id.len()..].trim_start();
                    if after.starts_with(':') && !after.starts_with("::") {
                        out.push(id.to_string());
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
                            out.push(id.to_string());
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
    for line in text.lines() {
        let t = line.trim();
        if t.contains("struct {") || t.ends_with("struct{") {
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
        out.insert(key, Some(ty.to_string()));
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
    let mut out = Fields::new();
    let mut class_indent: Option<usize> = None;
    for line in text.lines() {
        let ind = line.len() - line.trim_start().len();
        let t = line.trim();
        if t.starts_with("class ") {
            class_indent = Some(ind);
            continue;
        }
        let Some(ci) = class_indent else { continue };
        if !t.is_empty() && ind <= ci {
            class_indent = None;
            continue;
        }
        if t.starts_with('#') || t.starts_with("def ") || t.starts_with('@') {
            continue;
        }
        if let Some((name, ty)) = decl_name_type(t) {
            let key = quoted_after(t, "alias").map(str::to_string).unwrap_or(name);
            out.insert(key, Some(ty));
        }
    }
    out
}

fn ts_fields(text: &str) -> Fields {
    let mut out = Fields::new();
    for line in text.lines() {
        let t = line.trim().trim_start_matches("export ");
        if let Some((name, ty)) = decl_name_type(t) {
            out.insert(name, Some(ty.trim_end_matches(['{', ' ']).to_string()));
        }
    }
    out
}

fn rs_fields(text: &str) -> Fields {
    let mut out = Fields::new();
    let mut pending_rename: Option<String> = None;
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
        if let Some((name, ty)) = decl_name_type(d) {
            if t.ends_with(',') || t.ends_with('{') {
                out.insert(pending_rename.take().unwrap_or(name), Some(ty));
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
/// A removed field has `None`; a retyped one has `Some((old, new))`.
pub fn changed_fields(base: &Fields, head: &Fields) -> Vec<(String, Option<(String, String)>)> {
    let mut out = Vec::new();
    for (name, old_ty) in base {
        match head.get(name) {
            None => out.push((name.clone(), None)),
            Some(new_ty) => {
                if let (Some(a), Some(b)) = (old_ty, new_ty) {
                    if a != b {
                        out.push((name.clone(), Some((a.clone(), b.clone()))));
                    }
                }
            }
        }
    }
    out
}
