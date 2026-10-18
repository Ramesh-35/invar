//! INIT-DISCOVERY slice 2: zod VALUE producers (K2) and egress-lock
//! boundaries (K1) (docs/architecture/INIT-DISCOVERY-SLICE2.md, FROZEN r2).
//!
//! HONEST SCOPE: line and bracket-depth patterns over comment- and
//! string-masked text, not a parser (ruling R1: no AST, no type system, no
//! new dependencies). A shape is the set of DEPTH-1 member keys of
//! `export const <Name> = z.object({ ... })`, attributed to its own const.
//! K1 proposes an emitted-shape contract only where (a) the boundary file is
//! ELIGIBLE (versioned `v<N>` path segment, or declared under `[egress]` in
//! invar.toml), (b) the schema is imported by name, and (c) the schema is
//! referenced in a RESPONSE-PRODUCING position by the frozen marker sets.
//! Even then runtime serialization is NOT verified. K2 proposes producer and
//! consumer pairs only for type-blind consumers. Everything that cannot be
//! stated statically is an explicit abstention, never a guess.
//!
//! This module is self-contained: it does not modify or depend on private
//! parts of the slice 1 engine (`codepairs`), only its public constants.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::codepairs::{Abstention, MIN_FIELDS, STOPLIST};
use crate::surface::name_key;

pub mod strings;

const SRC_EXTS: &[&str] = &["ts", "tsx", "js", "jsx", "mjs"];
const OBJECT_CTORS: &[&str] = &["object", "strictObject", "looseObject"];
/// Builder chains that change the key set (R-X-2 plus the section C
/// "builder chains" class). A shape carrying one is unrepresentable.
const DYNAMIC_CHAIN: &[&str] = &[
    "extend", "merge", "pick", "omit", "passthrough", "catchall", "and", "setKey",
];

/// Frozen response-marker and request-marker vocabulary (R-K1-2). Extending
/// either set requires a ruling. Matching rules are in `marker_side`.
pub const RESPONSE_MARKERS: &[&str] = &[".json(", ".send(", "new Response", "successResponse("];
pub const REQUEST_MARKERS: &[&str] = &["req.", "request.", "body", "params", "searchParams", "safeParse("];

// ------------------------------------------------------------------ text

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b'$'
}

/// Same byte length as `text`; string contents and comments become spaces,
/// quotes and newlines are kept, so offsets and line numbers line up.
fn mask(text: &str) -> Vec<u8> {
    #[derive(PartialEq)]
    enum S {
        Code,
        Line,
        Block,
        Str(u8),
    }
    let b = text.as_bytes();
    let mut out = b.to_vec();
    let mut s = S::Code;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        match s {
            S::Code => {
                if c == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
                    s = S::Line;
                    out[i] = b' ';
                } else if c == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
                    s = S::Block;
                    out[i] = b' ';
                    out[i + 1] = b' ';
                    i += 1;
                } else if c == b'"' || c == b'\'' || c == b'`' {
                    s = S::Str(c);
                }
            }
            S::Line => {
                if c == b'\n' {
                    s = S::Code;
                } else {
                    out[i] = b' ';
                }
            }
            S::Block => {
                if c == b'*' && i + 1 < b.len() && b[i + 1] == b'/' {
                    out[i] = b' ';
                    out[i + 1] = b' ';
                    i += 1;
                    s = S::Code;
                } else if c != b'\n' {
                    out[i] = b' ';
                }
            }
            S::Str(q) => {
                if c == b'\\' && i + 1 < b.len() {
                    out[i] = b' ';
                    if b[i + 1] != b'\n' {
                        out[i + 1] = b' ';
                    }
                    i += 1;
                } else if c == q {
                    s = S::Code;
                } else if c == b'\n' && q != b'`' {
                    s = S::Code;
                } else if c != b'\n' {
                    out[i] = b' ';
                }
            }
        }
        i += 1;
    }
    out
}

struct Text<'a> {
    orig: &'a str,
    m: Vec<u8>,
    line_starts: Vec<usize>,
}

impl<'a> Text<'a> {
    fn new(orig: &'a str) -> Self {
        let mut line_starts = vec![0];
        for (i, c) in orig.bytes().enumerate() {
            if c == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Text { orig, m: mask(orig), line_starts }
    }
    fn line(&self, off: usize) -> usize {
        match self.line_starts.binary_search(&off) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    }
    fn ws(&self, mut i: usize) -> usize {
        while i < self.m.len() && self.m[i].is_ascii_whitespace() {
            i += 1;
        }
        i
    }
    fn ident(&self, i: usize) -> Option<(String, usize)> {
        if i >= self.m.len() || !is_ident_start(self.m[i]) {
            return None;
        }
        let mut e = i;
        while e < self.m.len() && is_ident(self.m[e]) {
            e += 1;
        }
        Some((self.orig[i..e].to_string(), e))
    }
    fn kw(&self, i: usize, word: &str) -> bool {
        let w = word.as_bytes();
        i + w.len() <= self.m.len()
            && &self.m[i..i + w.len()] == w
            && (i + w.len() == self.m.len() || !is_ident(self.m[i + w.len()]))
            && (i == 0 || !is_ident(self.m[i - 1]))
    }
    fn string_at(&self, q: usize) -> Option<(String, usize)> {
        let qc = *self.m.get(q)?;
        if !matches!(qc, b'"' | b'\'' | b'`') {
            return None;
        }
        let mut e = q + 1;
        while e < self.m.len() && self.m[e] != qc && self.m[e] != b'\n' {
            e += 1;
        }
        if e >= self.m.len() || self.m[e] != qc {
            return None;
        }
        Some((self.orig[q + 1..e].to_string(), e + 1))
    }
    /// Offset after the bracket matching the opener at `i`.
    fn close(&self, i: usize) -> Option<usize> {
        let mut depth = 0i64;
        let mut j = i;
        while j < self.m.len() {
            match self.m[j] {
                b'{' | b'(' | b'[' => depth += 1,
                b'}' | b')' | b']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(j + 1);
                    }
                }
                _ => {}
            }
            j += 1;
        }
        None
    }
}

// ----------------------------------------------------------- zod shapes

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeBody {
    /// Depth-1 member keys with their line.
    Fields(Vec<(String, usize)>),
    /// Cannot be stated statically; the reason is shown in the abstention.
    Dynamic(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZodDecl {
    pub name: String,
    pub line: usize,
    pub body: ShapeBody,
}

/// Parse the members of the object literal opened at `open` (a `{`).
fn zod_members(t: &Text, open: usize) -> ShapeBody {
    let Some(end) = t.close(open) else {
        return ShapeBody::Dynamic("unterminated object literal".into());
    };
    let stop = end - 1;
    let mut out: Vec<(String, usize)> = Vec::new();
    let mut j = open + 1;
    loop {
        j = t.ws(j);
        if j >= stop {
            break;
        }
        let c = t.m[j];
        if c == b',' {
            j += 1;
            continue;
        }
        if t.m[j..].starts_with(b"...") {
            return ShapeBody::Dynamic("spread member".into());
        }
        if c == b'[' {
            return ShapeBody::Dynamic("computed key".into());
        }
        let (key, e) = if matches!(c, b'"' | b'\'') {
            match t.string_at(j) {
                Some(x) => x,
                None => return ShapeBody::Dynamic("unreadable quoted key".into()),
            }
        } else if let Some(x) = t.ident(j) {
            x
        } else {
            return ShapeBody::Dynamic("unrecognized member form".into());
        };
        if key.is_empty() {
            return ShapeBody::Dynamic("empty key".into());
        }
        let e2 = t.ws(e);
        match t.m.get(e2) {
            Some(b':') | Some(b',') => {}
            _ if e2 >= stop => {} // shorthand member at the end
            Some(b'(') => return ShapeBody::Dynamic("method member".into()),
            _ => return ShapeBody::Dynamic("unrecognized member form".into()),
        }
        if !out.iter().any(|(k, _)| *k == key) {
            out.push((key, t.line(j)));
        }
        // Skip to the next top-level comma.
        let mut d = 0i64;
        let mut k = e2;
        while k < stop {
            match t.m[k] {
                b'(' | b'[' | b'{' => d += 1,
                b')' | b']' | b'}' => d -= 1,
                b',' if d == 0 => break,
                _ => {}
            }
            k += 1;
        }
        j = k + 1;
    }
    ShapeBody::Fields(out)
}

/// Classify the declaration `export const NAME = <rhs>` starting at `eq`+1.
fn zod_rhs(t: &Text, rhs: usize) -> Option<ShapeBody> {
    let (a, ae) = t.ident(rhs)?;
    let p = t.ws(ae);
    if t.m.get(p) != Some(&b'.') {
        return None;
    }
    let q = t.ws(p + 1);
    let (b, be) = t.ident(q)?;
    if a != "z" {
        // `Other.extend(...)`: a shape derived from another one.
        if DYNAMIC_CHAIN.contains(&b.as_str()) && t.m.get(t.ws(be)) == Some(&b'(') {
            return Some(ShapeBody::Dynamic(format!("derived from {a} via .{b}()")));
        }
        return None;
    }
    if !OBJECT_CTORS.contains(&b.as_str()) {
        return None; // z.array, z.enum, ...: not an object shape
    }
    let po = t.ws(be);
    if t.m.get(po) != Some(&b'(') {
        return None;
    }
    let paren_end = t.close(po)?;
    let arg = t.ws(po + 1);
    let body = if t.m.get(arg) == Some(&b'{') {
        zod_members(t, arg)
    } else {
        ShapeBody::Dynamic("argument is not an object literal".into())
    };
    // Builder chain after the literal.
    let mut c = t.ws(paren_end);
    while t.m.get(c) == Some(&b'.') {
        let Some((id, ie)) = t.ident(t.ws(c + 1)) else { break };
        if DYNAMIC_CHAIN.contains(&id.as_str()) {
            return Some(ShapeBody::Dynamic(format!(".{id}() chain changes the key set")));
        }
        let mut n = t.ws(ie);
        if t.m.get(n) == Some(&b'(') {
            n = t.close(n).unwrap_or(n + 1);
        }
        c = t.ws(n);
    }
    Some(body)
}

/// Every top-level `export const NAME = z.object(...)` (and derived shapes)
/// in a file, in source order.
pub fn scan_zod(text: &str) -> Vec<ZodDecl> {
    let t = Text::new(text);
    let n = t.m.len();
    let mut out = Vec::new();
    let mut depth = 0i64;
    let mut i = 0;
    while i < n {
        match t.m[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            _ if depth == 0 && t.kw(i, "export") => {
                let j = t.ws(i + 6);
                if t.kw(j, "const") {
                    let k = t.ws(j + 5);
                    if let Some((name, e)) = t.ident(k) {
                        let eq = t.ws(e);
                        if t.m.get(eq) == Some(&b'=') && t.m.get(eq + 1) != Some(&b'=') {
                            if let Some(body) = zod_rhs(&t, t.ws(eq + 1)) {
                                out.push(ZodDecl { name, line: t.line(k), body });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeRead {
    Found(Vec<String>),
    Dynamic(String),
    NotFound,
}

/// The current depth-1 key set of shape `name` in `text` (used by `invar
/// check`: the locked shape is re-extracted with this same code).
pub fn read_shape(text: &str, name: &str) -> ShapeRead {
    match scan_zod(text).into_iter().find(|d| d.name == name) {
        Some(ZodDecl { body: ShapeBody::Fields(f), .. }) => ShapeRead::Found(f.into_iter().map(|(k, _)| k).collect()),
        Some(ZodDecl { body: ShapeBody::Dynamic(why), .. }) => ShapeRead::Dynamic(why),
        None => ShapeRead::NotFound,
    }
}

/// Shape-attributed (scope, field) entries for the named zod consts, for the
/// lock/check producer surface. Shapes that are dynamic or absent add nothing.
pub fn zod_surface(text: &str, names: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for d in scan_zod(text) {
        if !names.contains(&d.name) {
            continue;
        }
        if let ShapeBody::Fields(f) = d.body {
            out.extend(f.into_iter().map(|(k, _)| (d.name.clone(), k)));
        }
    }
    out
}

// -------------------------------------------------------------- modules

#[derive(Debug, Clone)]
struct Imp {
    line: usize,
    span: (usize, usize),
    spec: String,
    /// (imported name, local binding)
    names: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
struct Reex {
    line: usize,
    spec: String,
    /// None = `export *`; Some = (exported name, original name).
    names: Option<Vec<(String, String)>>,
}

#[derive(Debug, Default, Clone)]
struct Module {
    decls: Vec<ZodDecl>,
    imports: Vec<Imp>,
    reexports: Vec<Reex>,
}

fn braces(t: &Text, open: usize, close: usize) -> Vec<(String, String)> {
    let inner = String::from_utf8_lossy(&t.m[open + 1..close - 1]).to_string();
    let mut out = Vec::new();
    for item in inner.split(',') {
        let mut words: Vec<&str> = item.split_whitespace().collect();
        if words.first() == Some(&"type") && words.len() > 1 {
            words.remove(0);
        }
        match words.as_slice() {
            [a] => out.push((a.to_string(), a.to_string())),
            [a, "as", b] => out.push((a.to_string(), b.to_string())),
            _ => {}
        }
    }
    out.retain(|(a, b)| !a.is_empty() && !b.is_empty() && a != "default" && b != "default");
    out
}

fn from_spec(t: &Text, i: usize) -> Option<(String, usize)> {
    let i = t.ws(i);
    if !t.kw(i, "from") {
        return None;
    }
    t.string_at(t.ws(i + 4))
}

fn scan_module(text: &str) -> Module {
    let t = Text::new(text);
    let mut md = Module { decls: scan_zod(text), ..Default::default() };
    let n = t.m.len();
    let mut depth = 0i64;
    let mut i = 0;
    while i < n {
        match t.m[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            _ if depth == 0 && t.kw(i, "import") => {
                let mut j = t.ws(i + 6);
                if t.kw(j, "type") {
                    j = t.ws(j + 4);
                }
                // optional default binding: `X,` or `X from`
                if let Some((_, e)) = t.ident(j) {
                    if !t.kw(j, "from") {
                        j = t.ws(e);
                        if t.m.get(j) == Some(&b',') {
                            j = t.ws(j + 1);
                        }
                    }
                }
                if t.m.get(j) == Some(&b'{') {
                    if let Some(end) = t.close(j) {
                        let names = braces(&t, j, end);
                        if let Some((spec, e)) = from_spec(&t, end) {
                            md.imports.push(Imp { line: t.line(i), span: (i, e), spec, names });
                            i = e;
                            continue;
                        }
                    }
                }
            }
            _ if depth == 0 && t.kw(i, "export") => {
                let mut j = t.ws(i + 6);
                if t.kw(j, "type") {
                    j = t.ws(j + 4);
                }
                if t.m.get(j) == Some(&b'{') {
                    if let Some(end) = t.close(j) {
                        if let Some((spec, e)) = from_spec(&t, end) {
                            let names = braces(&t, j, end).into_iter().map(|(orig, exp)| (exp, orig)).collect();
                            md.reexports.push(Reex { line: t.line(i), spec, names: Some(names) });
                            i = e;
                            continue;
                        }
                    }
                } else if t.m.get(j) == Some(&b'*') {
                    let mut k = t.ws(j + 1);
                    if t.kw(k, "as") {
                        // `export * as ns from`: a namespace, not a name list
                        k = t.ws(k + 2);
                        if let Some((_, e)) = t.ident(k) {
                            k = e;
                        }
                        if let Some((_, e)) = from_spec(&t, k) {
                            i = e;
                            continue;
                        }
                    } else if let Some((spec, e)) = from_spec(&t, k) {
                        md.reexports.push(Reex { line: t.line(i), spec, names: None });
                        i = e;
                        continue;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    md
}

// ----------------------------------------------------------- reads (K2)

/// (dot reads, bracket-literal reads): name_key -> (spelling, first line).
type Reads = BTreeMap<String, (String, usize)>;

fn split_reads(t: &Text) -> (Reads, Reads) {
    let mut dot: Reads = BTreeMap::new();
    let mut brk: Reads = BTreeMap::new();
    let b = &t.m;
    for i in 0..b.len() {
        match b[i] {
            b'.' if i > 0 && (is_ident(b[i - 1]) || matches!(b[i - 1], b')' | b']' | b'?')) => {
                let mut k = i;
                while k > 0 && is_ident(b[k - 1]) {
                    k -= 1;
                }
                let number = k < i && b[k].is_ascii_digit();
                if let (false, Some((id, _))) = (number, t.ident(i + 1)) {
                    dot.entry(name_key(&id)).or_insert((id, t.line(i)));
                }
            }
            b'[' if i > 0 && (is_ident(b[i - 1]) || matches!(b[i - 1], b')' | b']')) => {
                if let Some((s, e)) = t.string_at(i + 1) {
                    if b.get(e) == Some(&b']') && !s.is_empty() {
                        brk.entry(name_key(&s)).or_insert((s, t.line(i)));
                    }
                }
            }
            _ => {}
        }
    }
    (dot, brk)
}

// ------------------------------------------------- reference classification

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Response,
    Request,
    Unclassified,
}

/// Byte span of the masked statement containing the reference at `at`.
fn statement_span(t: &Text, at: usize) -> (usize, usize) {
    let m = &t.m;
    // Backward.
    let mut d = 0i64;
    let mut start = 0;
    let mut j = at;
    while j > 0 {
        j -= 1;
        let c = m[j];
        match c {
            b')' | b']' => d += 1,
            b'}' => {
                if d == 0 {
                    // A block or literal that ended before us. If something
                    // that continues an expression follows it, keep going;
                    // otherwise it terminated the previous statement.
                    let k = t.ws(j + 1);
                    let cont = matches!(m.get(k), Some(b',') | Some(b')') | Some(b']') | Some(b'.') | Some(b'?') | Some(b':') | Some(b'|') | Some(b'&') | Some(b'+') | Some(b'=') | Some(b';'));
                    if !cont {
                        start = j + 1;
                        break;
                    }
                }
                d += 1;
            }
            b'(' | b'[' => {
                if d > 0 {
                    d -= 1;
                }
            }
            b'{' => {
                if d > 0 {
                    d -= 1;
                } else {
                    // Enclosing brace: a block ends the statement, an object
                    // literal does not.
                    let mut k = j;
                    while k > 0 && m[k - 1].is_ascii_whitespace() {
                        k -= 1;
                    }
                    let prev = if k > 0 { m[k - 1] } else { 0 };
                    let word_before = {
                        let mut s = k;
                        while s > 0 && is_ident(m[s - 1]) {
                            s -= 1;
                        }
                        String::from_utf8_lossy(&m[s..k]).to_string()
                    };
                    let block = prev == b')'
                        || prev == b'>' && k >= 2 && m[k - 2] == b'='
                        || matches!(word_before.as_str(), "else" | "try" | "finally" | "do");
                    if block {
                        start = j + 1;
                        break;
                    }
                }
            }
            b';' if d == 0 => {
                start = j + 1;
                break;
            }
            _ => {}
        }
    }
    // Forward.
    let mut e = 0i64;
    let mut end = m.len();
    let mut k = at;
    while k < m.len() {
        match m[k] {
            b'(' | b'[' | b'{' => e += 1,
            b')' | b']' => {
                if e > 0 {
                    e -= 1;
                }
            }
            b'}' => {
                if e > 0 {
                    e -= 1;
                } else {
                    end = k;
                    break;
                }
            }
            b';' if e == 0 => {
                end = k;
                break;
            }
            _ => {}
        }
        k += 1;
    }
    (start, end.max(start))
}

fn has_word(s: &str, w: &str) -> bool {
    let b = s.as_bytes();
    let mut from = 0;
    while let Some(p) = s[from..].find(w) {
        let a = from + p;
        let z = a + w.len();
        if (a == 0 || !is_ident(b[a - 1])) && (z >= b.len() || !is_ident(b[z])) {
            return true;
        }
        from = a + 1;
    }
    false
}

/// `.json(` counts as a response marker unless its receiver is the request
/// (`req.json(`, `request.json(`): a request-body read is not a response.
fn response_json(s: &str) -> bool {
    let b = s.as_bytes();
    let mut from = 0;
    while let Some(p) = s[from..].find(".json(") {
        let a = from + p;
        let mut k = a;
        while k > 0 && is_ident(b[k - 1]) {
            k -= 1;
        }
        let recv = &s[k..a];
        if recv != "req" && recv != "request" {
            return true;
        }
        from = a + 1;
    }
    false
}

fn marker_side(stmt: &str) -> Side {
    let resp = response_json(stmt)
        || stmt.contains(".send(")
        || stmt.contains("new Response")
        || stmt.contains("successResponse(");
    if resp {
        return Side::Response;
    }
    let req = stmt.contains("req.")
        && has_word(stmt, "req")
        || stmt.contains("request.") && has_word(stmt, "request")
        || has_word(stmt, "body")
        || has_word(stmt, "params")
        || has_word(stmt, "searchParams")
        || stmt.contains("safeParse(");
    if req {
        Side::Request
    } else {
        Side::Unclassified
    }
}

/// Classify every reference to `local` outside the import statement `span`.
fn classify_refs(t: &Text, local: &str, span: (usize, usize)) -> Vec<(usize, Side)> {
    let mut out = Vec::new();
    let w = local.as_bytes();
    let m = &t.m;
    let mut i = 0;
    while i + w.len() <= m.len() {
        if &m[i..i + w.len()] == w
            && (i == 0 || !is_ident(m[i - 1]))
            && (i + w.len() == m.len() || !is_ident(m[i + w.len()]))
            && !(i >= span.0 && i < span.1)
            && !(i > 0 && m[i - 1] == b'.')
        {
            let (a, z) = statement_span(t, i);
            let stmt = String::from_utf8_lossy(&m[a..z]).to_string();
            out.push((t.line(i), marker_side(&stmt)));
            i += w.len();
        } else {
            i += 1;
        }
    }
    out
}

// ------------------------------------------------------------ resolution

enum Res {
    File(String),
    /// A bare specifier that names no workspace package (a library).
    NotCandidate,
    Unresolved(String),
    Ambig(String),
}

struct World {
    files: BTreeSet<String>,
    /// package name -> dirs
    pkgs: BTreeMap<String, Vec<String>>,
    mods: BTreeMap<String, Module>,
}

fn norm(path: &str) -> Option<String> {
    let mut out: Vec<&str> = Vec::new();
    for s in path.split('/') {
        match s {
            "" | "." => {}
            ".." => {
                out.pop()?;
            }
            x => out.push(x),
        }
    }
    Some(out.join("/"))
}

fn dir_of(f: &str) -> &str {
    f.rsplit_once('/').map(|(d, _)| d).unwrap_or("")
}

fn join(dir: &str, rest: &str) -> String {
    if dir.is_empty() {
        rest.to_string()
    } else if rest.is_empty() {
        dir.to_string()
    } else {
        format!("{dir}/{rest}")
    }
}

impl World {
    fn probe(&self, base: &str) -> Option<String> {
        let mut bases = vec![base.to_string()];
        for (from, to) in [(".js", ".ts"), (".jsx", ".tsx"), (".mjs", ".ts")] {
            if let Some(stem) = base.strip_suffix(from) {
                bases.push(format!("{stem}{to}"));
            }
        }
        for b in &bases {
            if self.files.contains(b) && self.mods.contains_key(b) {
                return Some(b.clone());
            }
            for e in SRC_EXTS {
                let c = format!("{b}.{e}");
                if self.files.contains(&c) {
                    return Some(c);
                }
            }
            for e in SRC_EXTS {
                let c = format!("{b}/index.{e}");
                if self.files.contains(&c) {
                    return Some(c);
                }
            }
        }
        None
    }

    fn resolve(&self, spec: &str, from: &str) -> Res {
        if spec == "." || spec == ".." || spec.starts_with("./") || spec.starts_with("../") {
            let Some(joined) = norm(&join(dir_of(from), spec)) else {
                return Res::Unresolved("relative path leaves the repository".into());
            };
            return match self.probe(&joined) {
                Some(f) => Res::File(f),
                None => Res::Unresolved("no such source file".into()),
            };
        }
        if spec.starts_with("@/") || spec.starts_with("~/") || spec.starts_with('#') {
            return Res::Unresolved("path alias (tsconfig paths are not read)".into());
        }
        let mut best: Option<(&String, &Vec<String>)> = None;
        for (name, dirs) in &self.pkgs {
            if (spec == name || spec.starts_with(&format!("{name}/")))
                && best.is_none_or(|(b, _)| name.len() > b.len())
            {
                best = Some((name, dirs));
            }
        }
        let Some((name, dirs)) = best else { return Res::NotCandidate };
        if dirs.len() > 1 {
            return Res::Ambig(format!("package {name:?} is declared by more than one package.json: {}", dirs.join(", ")));
        }
        let dir = &dirs[0];
        let sub = spec[name.len()..].trim_start_matches('/');
        let tries: Vec<String> = if sub.is_empty() {
            vec![join(dir, "src/index"), join(dir, "index")]
        } else {
            vec![join(dir, sub), join(dir, &format!("src/{sub}"))]
        };
        for t in tries {
            if let Some(f) = self.probe(&t) {
                return Res::File(f);
            }
        }
        Res::Unresolved(format!("workspace package {name:?} has no matching source file for {spec:?}"))
    }
}

enum Found {
    NotShape,
    Barrel2,
    Ambig(Vec<String>),
    Decl { file: String, decl: ZodDecl, barrel: Option<(String, usize)> },
}

impl World {
    fn direct(&self, file: &str, name: &str) -> Option<ZodDecl> {
        self.mods.get(file)?.decls.iter().find(|d| d.name == name).cloned()
    }

    /// `name` as exported by `file`, through at most one barrel level.
    fn find(&self, file: &str, name: &str) -> Found {
        if let Some(d) = self.direct(file, name) {
            return Found::Decl { file: file.into(), decl: d, barrel: None };
        }
        let Some(md) = self.mods.get(file) else { return Found::NotShape };
        let mut hits: Vec<(String, ZodDecl, (String, usize))> = Vec::new();
        let mut deeper = false;
        for r in &md.reexports {
            let orig = match &r.names {
                None => Some(name.to_string()),
                Some(ns) => ns.iter().find(|(e, _)| e == name).map(|(_, o)| o.clone()),
            };
            let Some(orig) = orig else { continue };
            let Res::File(t) = self.resolve(&r.spec, file) else { continue };
            if let Some(d) = self.direct(&t, &orig) {
                hits.push((t, d, (file.to_string(), r.line)));
            } else if self.mods.get(&t).is_some_and(|m| !m.reexports.is_empty()) {
                deeper = true;
            }
        }
        match hits.len() {
            0 if deeper => Found::Barrel2,
            0 => Found::NotShape,
            1 => {
                let (f, d, b) = hits.remove(0);
                Found::Decl { file: f, decl: d, barrel: Some(b) }
            }
            _ => Found::Ambig(hits.into_iter().map(|(f, _, _)| f).collect()),
        }
    }
}

// ------------------------------------------------------------ discovery

#[derive(Debug, Clone)]
pub struct BoundaryEvidence {
    pub file: String,
    pub import_line: usize,
    pub specifier: String,
    pub resolved: String,
    pub barrel: Option<(String, usize)>,
    pub eligibility: String,
    /// file:line of each response-position reference.
    pub response_refs: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct EgressProposal {
    pub producer: String,
    pub shape: String,
    pub decl_line: usize,
    pub fields: Vec<(String, usize)>,
    pub boundaries: Vec<BoundaryEvidence>,
    pub category: &'static str,
}

#[derive(Debug, Clone)]
pub struct K2Shape {
    pub shape: String,
    pub decl_line: usize,
    pub import_line: usize,
    pub specifier: String,
    pub resolved: String,
    pub barrel: Option<(String, usize)>,
    pub fields: Vec<(String, usize)>,
    pub stoplisted: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct K2Pair {
    pub producer: String,
    pub consumer: String,
    pub consumer_kind: &'static str,
    pub category: &'static str,
    pub shapes: Vec<K2Shape>,
}

#[derive(Debug, Default)]
pub struct ValueDiscovery {
    pub egress: Vec<EgressProposal>,
    pub pairs: Vec<K2Pair>,
    pub abstentions: Vec<Abstention>,
}

fn is_test_path(rel: &str) -> bool {
    let segs: Vec<&str> = rel.split('/').collect();
    let (dirs, file) = segs.split_at(segs.len().saturating_sub(1));
    let file = file.first().copied().unwrap_or("");
    dirs.iter().any(|d| matches!(*d, "test" | "tests" | "__tests__"))
        || [".test.ts", ".test.tsx", ".spec.ts", ".spec.tsx", ".test.js", ".spec.js"].iter().any(|s| file.ends_with(s))
}

fn category(rel: &str) -> &'static str {
    let segs: Vec<&str> = rel.split('/').collect();
    let (dirs, file) = segs.split_at(segs.len().saturating_sub(1));
    let file = file.first().copied().unwrap_or("");
    let has = |set: &[&str]| dirs.iter().any(|d| set.contains(d));
    let test_file = [".test.ts", ".test.tsx", ".spec.ts", ".spec.tsx", ".test.js", ".spec.js"].iter().any(|s| file.ends_with(s));
    if has(&["test", "tests", "__tests__", "testUtils", "__mocks__", "fixtures"]) || test_file {
        "test"
    } else if has(&["example", "examples", "demo"]) {
        "example"
    } else if has(&["scripts", "tools"]) {
        "tooling"
    } else if rel.starts_with("src/") || rel.starts_with("lib/") || rel.contains("/src/") {
        "production"
    } else {
        "unknown"
    }
}

fn ext_of(f: &str) -> &str {
    f.rsplit('.').next().unwrap_or("")
}

fn is_version_seg(s: &str) -> bool {
    s.len() >= 2 && s.starts_with('v') && s[1..].bytes().all(|b| b.is_ascii_digit())
}

/// The versioned path segment (`v1`, `v2`...) of a boundary file, if any.
pub fn versioned_segment(rel: &str) -> Option<String> {
    let mut segs: Vec<&str> = rel.split('/').collect();
    segs.pop(); // the file name is not a path segment for eligibility
    segs.into_iter().find(|s| is_version_seg(s)).map(String::from)
}

fn marked_by(rel: &str, marks: &[String]) -> Option<String> {
    marks
        .iter()
        .find(|m| {
            let m = m.trim_end_matches('/');
            !m.is_empty() && (rel == m || rel.starts_with(&format!("{m}/")))
        })
        .cloned()
}

/// Version-family layout (R-K1-6): `v/<N>` path segments, or a const named
/// `<Name>V<N>` / `V<N>_SCHEMA` (also `...Schema`).
pub fn version_family(file: &str, shape: &str) -> Option<String> {
    let segs: Vec<&str> = file.split('/').collect();
    for w in segs.windows(2) {
        if w[0] == "v" && !w[1].is_empty() && w[1].trim_end_matches(|c: char| c == '.' || c.is_ascii_alphabetic()).bytes().all(|b| b.is_ascii_digit()) && w[1].as_bytes()[0].is_ascii_digit() {
            return Some(format!("path layout v/{}", w[1]));
        }
    }
    let base = shape
        .strip_suffix("_SCHEMA")
        .or_else(|| shape.strip_suffix("Schema"))
        .unwrap_or(shape);
    let b = base.as_bytes();
    let mut k = b.len();
    while k > 0 && b[k - 1].is_ascii_digit() {
        k -= 1;
    }
    if k < b.len() && k > 0 && b[k - 1] == b'V' && (k == 1 || !b[k - 2].is_ascii_uppercase()) {
        return Some(format!("version-numbered name {shape:?}"));
    }
    None
}

fn distinctive(fields: &[(String, usize)]) -> Vec<&(String, usize)> {
    let stop: BTreeSet<String> = STOPLIST.iter().map(|s| name_key(s)).collect();
    fields.iter().filter(|(f, _)| !stop.contains(&name_key(f))).collect()
}

/// Read `[egress]` boundary markings from an invar.toml text. Each
/// `boundary = "<path or directory>"` key of any `[egress]` section marks it.
pub fn egress_marks(toml_text: &str) -> Vec<String> {
    let mut marks = Vec::new();
    let mut in_egress = false;
    for raw in toml_text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            in_egress = line == "[egress]";
            continue;
        }
        if !in_egress {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == "boundary" {
                let v = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !v.is_empty() {
                    marks.push(v.to_string());
                }
            }
        }
    }
    marks
}

pub fn discover_values(root: &Path, files: &[String], oversized: &[String], marks: &[String]) -> ValueDiscovery {
    let mut out = ValueDiscovery::default();
    let src: Vec<&String> = files.iter().filter(|f| SRC_EXTS.contains(&ext_of(f))).collect();
    let mut w = World { files: src.iter().map(|f| (*f).clone()).collect(), pkgs: BTreeMap::new(), mods: BTreeMap::new() };
    for f in files.iter().filter(|f| f.rsplit('/').next() == Some("package.json")) {
        if let Ok(text) = std::fs::read_to_string(root.join(f)) {
            if let Some(name) = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v.get("name").and_then(|n| n.as_str().map(String::from)))
            {
                w.pkgs.entry(name).or_default().push(dir_of(f).to_string());
            }
        }
    }
    let mut unreadable: BTreeSet<String> = BTreeSet::new();
    for f in &src {
        match std::fs::read(root.join(f)).map(String::from_utf8) {
            Ok(Ok(text)) => {
                w.mods.insert((*f).clone(), scan_module(&text));
            }
            _ => {
                unreadable.insert((*f).clone());
            }
        }
    }
    let eligible = |f: &str| -> Option<String> {
        if is_test_path(f) {
            return None;
        }
        let v = versioned_segment(f).map(|s| format!("versioned-path:{s}"));
        let m = marked_by(f, marks).map(|s| format!("marked:{s}"));
        match (v, m) {
            (None, None) => None,
            (Some(a), None) | (None, Some(a)) => Some(a),
            (Some(a), Some(b)) => Some(format!("{a},{b}")),
        }
    };

    // ---------------------------------------------------------------- K1
    struct Cand {
        decl: ZodDecl,
        boundaries: Vec<BoundaryEvidence>,
    }
    let mut cands: BTreeMap<(String, String), Cand> = BTreeMap::new();
    for f in &src {
        let Some(elig) = eligible(f) else { continue };
        if unreadable.contains(*f) {
            out.abstentions.push(Abstention {
                code: "UNREADABLE",
                location: (*f).clone(),
                reason: "eligible boundary file is not valid UTF-8 or not readable; not scanned".into(),
            });
            continue;
        }
        let Some(md) = w.mods.get(*f) else { continue };
        let text = std::fs::read_to_string(root.join(f)).unwrap_or_default();
        let t = Text::new(&text);
        for imp in &md.imports {
            let loc = format!("{f}:{}", imp.line);
            let target = match w.resolve(&imp.spec, f) {
                Res::NotCandidate => continue,
                Res::Unresolved(why) => {
                    out.abstentions.push(Abstention { code: "UNRESOLVED", location: loc, reason: format!("import from {:?} in an eligible boundary file: {why}", imp.spec) });
                    continue;
                }
                Res::Ambig(why) => {
                    out.abstentions.push(Abstention { code: "AMBIG-PACKAGE", location: loc, reason: format!("import from {:?}: {why}", imp.spec) });
                    continue;
                }
                Res::File(x) => x,
            };
            for (imported, local) in &imp.names {
                match w.find(&target, imported) {
                    Found::NotShape => {}
                    Found::Barrel2 => out.abstentions.push(Abstention {
                        code: "BARREL2",
                        location: loc.clone(),
                        reason: format!("{imported} from {:?} is re-exported through more than one barrel level (from {target})", imp.spec),
                    }),
                    Found::Ambig(fs) => out.abstentions.push(Abstention {
                        code: "AMBIG-SHAPE",
                        location: loc.clone(),
                        reason: format!("{imported} from {:?} resolves to more than one declaration: {}", imp.spec, fs.join(", ")),
                    }),
                    Found::Decl { file: pf, decl, barrel } => {
                        if &pf == *f {
                            continue;
                        }
                        if let Some(why) = version_family(&pf, &decl.name) {
                            out.abstentions.push(Abstention {
                                code: "VERSION-FAMILY",
                                location: loc.clone(),
                                reason: format!("{imported} at {pf}:{} lives in a schema version family ({why}); compatibility machinery presumed, no egress lock proposed", decl.line),
                            });
                            continue;
                        }
                        if let ShapeBody::Dynamic(why) = &decl.body {
                            out.abstentions.push(Abstention {
                                code: "UNSUPPORTED-DYNAMIC",
                                location: loc.clone(),
                                reason: format!("{imported} at {pf}:{} cannot be stated statically ({why})", decl.line),
                            });
                            continue;
                        }
                        let refs = classify_refs(&t, local, imp.span);
                        let resp: Vec<usize> = refs.iter().filter(|(_, s)| *s == Side::Response).map(|(l, _)| *l).collect();
                        if resp.is_empty() {
                            let why = if refs.is_empty() {
                                "imported but never referenced".to_string()
                            } else {
                                let lines: Vec<String> = refs.iter().map(|(l, s)| format!("{l} ({})", if *s == Side::Request { "request-side" } else { "unclassified" })).collect();
                                format!("referenced only in request-side or unclassifiable positions (lines {})", lines.join(", "))
                            };
                            out.abstentions.push(Abstention {
                                code: "AMBIG-EGRESS",
                                location: loc.clone(),
                                reason: format!("{imported} from {:?}: {why}; no response-position reference, no emitted-shape contract proposed", imp.spec),
                            });
                            continue;
                        }
                        let ev = BoundaryEvidence {
                            file: (*f).clone(),
                            import_line: imp.line,
                            specifier: imp.spec.clone(),
                            resolved: target.clone(),
                            barrel,
                            eligibility: elig.clone(),
                            response_refs: resp,
                        };
                        cands
                            .entry((pf, decl.name.clone()))
                            .or_insert_with(|| Cand { decl: decl.clone(), boundaries: Vec::new() })
                            .boundaries
                            .push(ev);
                    }
                }
            }
        }
    }
    for ((pf, shape), c) in cands {
        let ShapeBody::Fields(fields) = &c.decl.body else { continue };
        if distinctive(fields).len() < MIN_FIELDS {
            out.abstentions.push(Abstention {
                code: "AMBIG-EGRESS",
                location: format!("{pf}:{}", c.decl.line),
                reason: format!("{shape} has fewer than {MIN_FIELDS} distinctive (non-stoplist) fields; insufficient evidence for an emitted-shape contract"),
            });
            continue;
        }
        let cat = category(&pf);
        out.egress.push(EgressProposal {
            producer: pf,
            shape,
            decl_line: c.decl.line,
            fields: fields.clone(),
            boundaries: c.boundaries,
            category: cat,
        });
    }

    // ---------------------------------------------------------------- K2
    let stop: BTreeSet<String> = STOPLIST.iter().map(|s| name_key(s)).collect();
    let mut pairs: BTreeMap<(String, String), K2Pair> = BTreeMap::new();
    for (cf, md) in &w.mods {
        let ext = ext_of(cf);
        let typed_ext = ext == "ts" || ext == "tsx";
        let blind_ext = !typed_ext;
        let mut cache: Option<(String, ())> = None;
        let mut text_of = |cf: &str| -> String {
            if cache.is_none() {
                cache = Some((std::fs::read_to_string(root.join(cf)).unwrap_or_default(), ()));
            }
            cache.as_ref().map(|c| c.0.clone()).unwrap_or_default()
        };
        for imp in &md.imports {
            let loc = format!("{cf}:{}", imp.line);
            let Res::File(target) = w.resolve(&imp.spec, cf) else { continue };
            for (imported, local) in &imp.names {
                let Found::Decl { file: pf, decl, barrel } = w.find(&target, imported) else { continue };
                if &pf == cf {
                    continue;
                }
                match &decl.body {
                    ShapeBody::Dynamic(why) => {
                        // Eligible boundary files were already reported by the
                        // K1 pass above (one abstention per condition).
                        if blind_ext && eligible(cf).is_none() && version_family(&pf, &decl.name).is_some() {
                            out.abstentions.push(Abstention {
                                code: "VERSION-FAMILY",
                                location: loc.clone(),
                                reason: format!("{imported} at {pf}:{} lives in a schema version family; pairing declined", decl.line),
                            });
                        } else if blind_ext && eligible(cf).is_none() {
                            out.abstentions.push(Abstention {
                                code: "UNSUPPORTED-DYNAMIC",
                                location: loc.clone(),
                                reason: format!("{imported} at {pf}:{} cannot be stated statically ({why})", decl.line),
                            });
                        }
                    }
                    ShapeBody::Fields(members) => {
                        let text = text_of(cf);
                        let t = Text::new(&text);
                        let (dot, brk) = split_reads(&t);
                        let mut fields = Vec::new();
                        let mut stoplisted = Vec::new();
                        let mut dot_hits = 0usize;
                        for (m, _) in members {
                            let k = name_key(m);
                            let d = dot.get(&k);
                            let b = brk.get(&k);
                            let line = match (d, b) {
                                (Some((_, l)), Some((_, l2))) => Some(*l.min(l2)),
                                (Some((_, l)), None) | (None, Some((_, l))) => Some(*l),
                                _ => None,
                            };
                            if let Some(l) = line {
                                if stop.contains(&k) {
                                    stoplisted.push(m.clone());
                                } else {
                                    fields.push((m.clone(), l));
                                    if d.is_some() {
                                        dot_hits += 1;
                                    }
                                }
                            }
                        }
                        fields.sort();
                        fields.dedup_by(|a, b| a.0 == b.0);
                        stoplisted.sort();
                        stoplisted.dedup();
                        if typed_ext {
                            let infers = text.contains(&format!("typeof {local}")) || text.contains(&format!("typeof {imported}"));
                            if infers || (fields.len() >= MIN_FIELDS && dot_hits > 0) {
                                out.abstentions.push(Abstention {
                                    code: "TSC-GOVERNED",
                                    location: loc.clone(),
                                    reason: format!(
                                        "{imported} ({pf}:{}) is consumed through types in {cf} ({}); the type system governs this pair, INVAR adds no claim",
                                        decl.line,
                                        if infers { "z.infer or typeof use" } else { "typed property reads" }
                                    ),
                                });
                                continue;
                            }
                        }
                        if fields.len() < MIN_FIELDS {
                            continue;
                        }
                        if let Some(why) = version_family(&pf, &decl.name) {
                            if eligible(cf).is_none() {
                                out.abstentions.push(Abstention {
                                    code: "VERSION-FAMILY",
                                    location: loc.clone(),
                                    reason: format!("{imported} at {pf}:{} lives in a schema version family ({why}); pairing declined", decl.line),
                                });
                            }
                            continue;
                        }
                        let ev = K2Shape {
                            shape: imported.clone(),
                            decl_line: decl.line,
                            import_line: imp.line,
                            specifier: imp.spec.clone(),
                            resolved: target.clone(),
                            barrel: barrel.clone(),
                            fields,
                            stoplisted,
                        };
                        let pair = pairs.entry((pf.clone(), cf.clone())).or_insert_with(|| K2Pair {
                            producer: pf.clone(),
                            consumer: cf.clone(),
                            consumer_kind: if blind_ext { "plain JavaScript file" } else { "TypeScript file reading only through bracket literals" },
                            category: category(cf),
                            shapes: Vec::new(),
                        });
                        if !pair.shapes.iter().any(|s| s.shape == ev.shape) {
                            pair.shapes.push(ev);
                        }
                    }
                }
            }
        }
    }
    for p in pairs.values_mut() {
        p.shapes.sort_by(|a, b| a.shape.cmp(&b.shape));
    }
    out.pairs = pairs.into_values().collect();
    for f in oversized.iter().filter(|f| SRC_EXTS.contains(&ext_of(f))) {
        if eligible(f).is_some() {
            out.abstentions.push(Abstention {
                code: "UNREADABLE",
                location: f.clone(),
                reason: "eligible boundary file is larger than the 1000000-byte scan limit; not scanned".into(),
            });
        }
    }
    out.abstentions.sort();
    out.abstentions.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(t: &str, n: &str) -> Vec<String> {
        match read_shape(t, n) {
            ShapeRead::Found(f) => f,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn depth_one_members_only() {
        let f = fields(
            "export const Order = z.object({\n  total_minor: z.number(),\n  meta: z.object({ owner_ref: z.string() }),\n  'quoted-key': z.string(),\n});\n",
            "Order",
        );
        assert_eq!(f, vec!["total_minor", "meta", "quoted-key"]);
    }

    #[test]
    fn dynamic_forms_abstain() {
        for (src, why) in [
            ("export const A = z.object({ a: z.string() }).extend({ b: z.string() });", ".extend"),
            ("export const A = z.object({ ...Base, a: z.string() });", "spread"),
            ("export const A = z.object({ [k]: z.string() });", "computed"),
            ("export const A = B.extend({ a: z.string() });", "derived"),
            ("export const A = z.object(shape);", "not an object literal"),
        ] {
            match read_shape(src, "A") {
                ShapeRead::Dynamic(w) => assert!(w.contains(why), "{src}: {w}"),
                other => panic!("{src}: {other:?}"),
            }
        }
    }

    #[test]
    fn non_dynamic_chains_keep_the_shape() {
        let f = fields("export const A = z.object({ a: z.string(), b: z.number() }).strict().describe('x');", "A");
        assert_eq!(f, vec!["a", "b"]);
    }

    #[test]
    fn comments_and_strings_are_masked() {
        let f = fields("// export const A = z.object({ ghost: 1 })\nexport const A = z.object({\n  a: z.string(), // b: z.number()\n  c: z.literal('d: e'),\n});\n", "A");
        assert_eq!(f, vec!["a", "c"]);
    }

    #[test]
    fn marker_sides() {
        let one = |src: &str, name: &str| {
            let t = Text::new(src);
            classify_refs(&t, name, (0, 0)).into_iter().map(|(_, s)| s).collect::<Vec<_>>()
        };
        assert_eq!(one("return NextResponse.json(Order.parse(x));", "Order"), vec![Side::Response]);
        assert_eq!(one("const p = Order.safeParse(await request.json());", "Order"), vec![Side::Request]);
        assert_eq!(one("const o = Order.parse(x);", "Order"), vec![Side::Unclassified]);
        assert_eq!(
            one("function f() {\n  const p = Order.safeParse(body);\n  return NextResponse.json(p);\n}\n", "Order"),
            vec![Side::Request]
        );
    }

    #[test]
    fn version_family_detection() {
        assert!(version_family("pkg/src/collection/v/2/index.ts", "Anything").is_some());
        assert!(version_family("src/x.ts", "OrderV2").is_some());
        assert!(version_family("src/x.ts", "V1_SCHEMA").is_some());
        assert!(version_family("src/x.ts", "UserV1Schema").is_some());
        assert!(version_family("src/x.ts", "Order").is_none());
        assert!(version_family("api/v1/x.ts", "OrderResponse").is_none());
    }

    #[test]
    fn eligibility_segments() {
        assert_eq!(versioned_segment("app/api/v1/orders/route.ts"), Some("v1".into()));
        assert_eq!(versioned_segment("app/api/orders/route.ts"), None);
        assert_eq!(versioned_segment("app/api/v1.ts"), None);
        assert_eq!(egress_marks("[egress]\nboundary = \"src/public\"\n"), vec!["src/public".to_string()]);
    }
}
