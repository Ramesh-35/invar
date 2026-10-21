//! INIT-DISCOVERY slice 1: TypeScript code producers paired with TypeScript
//! consumers across qualifying workspace packages
//! (docs/architecture/INIT-DISCOVERY-SLICE1.md).
//!
//! HONEST SCOPE: line and bracket-depth patterns over comment- and
//! string-masked text, not a parser. A proposal establishes a named import of
//! an exported shape across two declared packages plus at least two of the
//! shape's direct members read by property access in the importing file.
//! Since INIT-DISCOVERY-SLICE5 a read counts only through a receiver declared
//! as the shape in that file (annotation, cast, explicit type argument, a
//! local function's declared return type, or an element of a declared
//! array); writes never count; types are not inferred, and receivers are
//! matched by name across the file, not by scope. Everything init cannot
//! resolve is an explicit abstention, never a guess.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::surface::name_key;

/// The frozen generic stoplist (Experiment C DEFINITION.md, spec step 4).
pub const STOPLIST: &[&str] = &[
    "id", "name", "type", "description", "data", "value", "payload", "key", "label", "title",
    "status", "error", "message", "result", "items", "url", "path", "version", "created_at",
    "updated_at",
];

/// Minimum distinctive fields per shape (ruling R3).
pub const MIN_FIELDS: usize = 2;

const MAX_DIRS: usize = 20_000;

/// One explicit refusal to propose.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Abstention {
    pub code: &'static str,
    pub location: String,
    pub reason: String,
}

impl Abstention {
    pub fn line(&self) -> String {
        format!("ABSTAIN {} {} {}", self.code, self.location, self.reason)
    }
}

/// A qualifying workspace package.
#[derive(Debug, Clone)]
pub struct Package {
    pub dir: String,
    pub name: String,
    pub marker: String,
    manifest: Value,
}

/// Evidence for one shape within a proposed pair (E2, E4, E5).
#[derive(Debug, Clone)]
pub struct ShapeEvidence {
    pub shape: String,
    pub decl_line: usize,
    pub import_line: usize,
    pub specifier: String,
    pub resolved: String,
    /// Barrel hops from `resolved` to the declaring file, in order; empty
    /// when `resolved` declares the shape.
    pub barrel: Vec<(String, usize)>,
    pub fields: Vec<(String, usize)>,
    pub stoplisted: Vec<String>,
    /// SLICE7: the imported functions whose written return type tied the
    /// receivers this shape's reads were counted through, as (function,
    /// declaring file, line); empty when no such route was needed.
    pub via: Vec<Via>,
    /// SLICE7: false when the shape is not imported by name (reached only
    /// through `via` or `props`).
    pub named: bool,
    /// SLICE8: the property paths its counted reads went through; empty
    /// when no such path was needed.
    pub props: Vec<Prop>,
}

/// SLICE7 provenance of one route: the function (as imported), its
/// declaring file and line, and the import's line and specifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Via {
    pub function: String,
    pub decl_file: String,
    pub decl_line: usize,
    pub import_line: usize,
    pub specifier: String,
}

/// A (producer declaring file, consumer file) pair with qualifying shapes.
#[derive(Debug, Clone)]
pub struct CodePair {
    pub producer: String,
    pub consumer: String,
    pub producer_pkg: Package,
    pub consumer_pkg: Package,
    pub category: &'static str,
    pub shapes: Vec<ShapeEvidence>,
}

#[derive(Debug, Default)]
pub struct CodeDiscovery {
    pub packages: Vec<Package>,
    pub repo_abstained: bool,
    pub truncated: bool,
    pub candidates: usize,
    pub pairs: Vec<CodePair>,
    pub abstentions: Vec<Abstention>,
}

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
                    s = S::Code; // unterminated string: recover at end of line
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
        Text {
            orig,
            m: mask(orig),
            line_starts,
        }
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
    /// The string literal starting at quote offset `q` (original text).
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

// ------------------------------------------------------------- per file

#[derive(Debug, Clone)]
enum Kind {
    Object(Vec<(String, usize)>),
    Indexed,
    Unsupported(String),
}

#[derive(Debug, Clone)]
struct Decl {
    name: String,
    line: usize,
    exported: bool,
    kind: Kind,
}

#[derive(Debug, Clone)]
struct Import {
    line: usize,
    spec: String,
    named: Vec<String>,
    /// SLICE7: (local binding, original name) of each braced item.
    locals: Vec<(String, String)>,
    /// SLICE3B: names imported type-only (`import type {..}`, or an inline
    /// `type X` inside the braces), by original name.
    type_only: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct ReExport {
    line: usize,
    spec: String,
    /// None = `export *`; Some = (exported name, original name).
    names: Option<Vec<(String, String)>>,
}

#[derive(Debug, Default, Clone)]
struct FileInfo {
    decls: Vec<Decl>,
    local_exports: Vec<(String, String)>,
    imports: Vec<Import>,
    reexports: Vec<ReExport>,
    /// name_key -> (spelling, first line), property access only.
    reads: BTreeMap<String, (String, usize)>,
    /// The masked text, for the per-shape receiver check (SLICE5).
    masked: Vec<u8>,
}

impl FileInfo {
    fn exported_shape(&self, name: &str) -> Option<&Decl> {
        if let Some(d) = self.decls.iter().find(|d| d.name == name && d.exported) {
            return Some(d);
        }
        let local = self
            .local_exports
            .iter()
            .find(|(e, _)| e == name)
            .map(|(_, l)| l.clone())?;
        self.decls.iter().find(|d| d.name == local)
    }
    fn exported_shapes(&self) -> Vec<(String, &Decl)> {
        let mut out: Vec<(String, &Decl)> = self
            .decls
            .iter()
            .filter(|d| d.exported)
            .map(|d| (d.name.clone(), d))
            .collect();
        for (e, l) in &self.local_exports {
            if let Some(d) = self.decls.iter().find(|d| &d.name == l) {
                out.push((e.clone(), d));
            }
        }
        out
    }
}

/// Members at exactly one nesting level inside the body opened at `open`.
/// Returns (members, offset after the closing brace).
fn members(t: &Text, open: usize) -> Option<(Vec<(String, usize)>, usize)> {
    let end = t.close(open)?;
    let mut out = Vec::new();
    let mut depth = 0i64;
    let mut seg_start = true;
    let mut j = open;
    while j < end {
        let c = t.m[j];
        match c {
            b'{' | b'(' | b'[' => {
                if depth == 1 && seg_start && c == b'[' {
                    seg_start = false; // index signature, not a field
                }
                depth += 1;
                if depth == 1 {
                    seg_start = true;
                }
            }
            b'}' | b')' | b']' => depth -= 1,
            b';' | b',' | b'\n' if depth == 1 => seg_start = true,
            _ if depth == 1 && seg_start && !c.is_ascii_whitespace() => {
                seg_start = false;
                if let Some(m) = member_at(t, j) {
                    out.push(m);
                }
            }
            _ => {}
        }
        j += 1;
    }
    Some((out, end))
}

/// `name: T`, `name?: T`, `readonly name: T`, `'quoted-name': T`.
fn member_at(t: &Text, i: usize) -> Option<(String, usize)> {
    let mut i = i;
    if t.kw(i, "readonly") {
        i = t.ws(i + "readonly".len());
    }
    let (name, mut e) = if matches!(t.m.get(i), Some(b'"') | Some(b'\'')) {
        let (s, e) = t.string_at(i)?;
        (s, e)
    } else {
        t.ident(i)?
    };
    if t.m.get(e) == Some(&b'?') {
        e += 1;
    }
    e = t.ws(e);
    if t.m.get(e) == Some(&b':') && t.m.get(e + 1) != Some(&b':') && !name.is_empty() {
        Some((name, t.line(i)))
    } else {
        None
    }
}

/// Skip a `<...>` generic list starting at `i` if present.
fn skip_generics(t: &Text, i: usize) -> usize {
    let i = t.ws(i);
    if t.m.get(i) != Some(&b'<') {
        return i;
    }
    let mut depth = 0i64;
    let mut j = i;
    while j < t.m.len() {
        match t.m[j] {
            b'<' => depth += 1,
            b'>' if j == 0 || t.m[j - 1] != b'=' => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            _ => {}
        }
        j += 1;
    }
    t.m.len()
}

fn type_kind(t: &Text, after_eq: usize) -> Kind {
    let i = t.ws(after_eq);
    let body_at = if t.m.get(i) == Some(&b'{') {
        Some(i)
    } else if t.kw(i, "Readonly") {
        let j = t.ws(i + "Readonly".len());
        if t.m.get(j) == Some(&b'<') {
            let k = t.ws(j + 1);
            (t.m.get(k) == Some(&b'{')).then_some(k)
        } else {
            None
        }
    } else {
        None
    };
    if let Some(open) = body_at {
        return match members(t, open) {
            Some((ms, end)) => {
                let mut k = t.ws(end);
                if t.m.get(k) == Some(&b'>') {
                    k = t.ws(k + 1);
                }
                if matches!(t.m.get(k), Some(b'&') | Some(b'|')) {
                    Kind::Unsupported("intersection or union with an object body".into())
                } else {
                    Kind::Object(ms)
                }
            }
            None => Kind::Unsupported("unterminated object body".into()),
        };
    }
    // Indexed alias into generated types: Ident(.Ident)*["..."]...
    let mut k = i;
    while k < t.m.len() && (is_ident(t.m[k]) || t.m[k] == b'.') {
        k += 1;
    }
    if k > i && t.m.get(k) == Some(&b'[') && matches!(t.m.get(k + 1), Some(b'"') | Some(b'\'')) {
        return Kind::Indexed;
    }
    let mut e = i;
    while e < t.m.len() && t.m[e] != b';' && t.m[e] != b'\n' {
        e += 1;
    }
    let snippet: String = t.orig[i..e].trim().chars().take(48).collect();
    Kind::Unsupported(snippet)
}

/// Names inside `{ a, type b, c as d }` as (exported-or-local, original).
fn brace_names(t: &Text, open: usize, close: usize) -> Vec<(String, String)> {
    let inner = &t.m[open + 1..close - 1];
    let inner = String::from_utf8_lossy(inner).to_string();
    let mut out = Vec::new();
    for item in inner.split(',') {
        let mut words: Vec<&str> = item.split_whitespace().collect();
        if words.first() == Some(&"type") && words.len() > 1 {
            words.remove(0);
        }
        match words.as_slice() {
            [a] => out.push((a.to_string(), a.to_string())),
            [a, "as", b] => out.push((b.to_string(), a.to_string())),
            _ => {}
        }
    }
    out.retain(|(a, b)| !a.is_empty() && !b.is_empty() && a != "default" && b != "default");
    out
}

/// SLICE3B: original names of the `type X` / `type X as Y` items in braces.
fn inline_type_names(t: &Text, open: usize, close: usize) -> BTreeSet<String> {
    let inner = String::from_utf8_lossy(&t.m[open + 1..close - 1]).to_string();
    inner
        .split(',')
        .filter_map(|item| {
            let words: Vec<&str> = item.split_whitespace().collect();
            match words.as_slice() {
                ["type", a] | ["type", a, "as", _] => Some(a.to_string()),
                _ => None,
            }
        })
        .collect()
}

/// SLICE3B: what a package's own sources declare for a name, read by a
/// plain word scan over the masked text (strings and comments blanked).
/// It uses no bracket depth, so a scanner miss cannot blind it, and it is
/// never used to propose: only to decide whether silence is earned.
#[derive(Default)]
struct DeclEvidence {
    /// First `interface X` / `type X` site, in path order.
    type_site: Option<(String, usize)>,
    /// `function X`, `const X`, `let X`, `var X`. A `class X` or `enum X`
    /// records the name with neither flag: present, not a type (S3B-D3).
    value: bool,
}

/// Returns the index and the files that could not be read (not UTF-8).
fn declaration_index<'f>(root: &Path, files: impl Iterator<Item = &'f String>) -> (BTreeMap<String, DeclEvidence>, Vec<String>) {
    let mut idx: BTreeMap<String, DeclEvidence> = BTreeMap::new();
    let mut unread = Vec::new();
    for f in files {
        let Ok(Ok(text)) = std::fs::read(root.join(f)).map(String::from_utf8) else {
            unread.push(f.clone());
            continue;
        };
        let t = Text::new(&text);
        let mut i = 0;
        while i < t.m.len() {
            if !is_ident_start(t.m[i]) || (i > 0 && is_ident(t.m[i - 1])) {
                i += 1;
                continue;
            }
            let mut e = i;
            while e < t.m.len() && is_ident(t.m[e]) {
                e += 1;
            }
            let kind = match &t.m[i..e] {
                b"interface" | b"type" => Some(0),
                b"function" | b"const" | b"let" | b"var" => Some(1),
                b"class" | b"enum" => Some(2),
                _ => None,
            };
            if let Some(k) = kind {
                let mut j = e;
                if &t.m[i..e] == b"function" && t.m.get(j) == Some(&b'*') {
                    j += 1;
                }
                let n = t.ws(j);
                if n > j {
                    if let Some((name, _)) = t.ident(n) {
                        let ev = idx.entry(name).or_default();
                        match k {
                            0 => {
                                if ev.type_site.is_none() {
                                    ev.type_site = Some((f.clone(), t.line(i)));
                                }
                            }
                            1 => ev.value = true,
                            _ => {}
                        }
                    }
                }
            }
            i = e;
        }
    }
    (idx, unread)
}

/// After a `{...}` or clause, find `from 'spec'`; returns (spec, end).
fn from_spec(t: &Text, i: usize) -> Option<(String, usize)> {
    let i = t.ws(i);
    if !t.kw(i, "from") {
        return None;
    }
    let q = t.ws(i + 4);
    t.string_at(q)
}

fn scan_file(text: &str) -> FileInfo {
    let t = Text::new(text);
    let mut fi = FileInfo::default();
    let n = t.m.len();
    let mut depth = 0i64;
    let mut i = 0;
    let mut stmt_start = true;
    while i < n {
        let c = t.m[i];
        if depth == 0 && stmt_start && is_ident_start(c) {
            stmt_start = false;
            if let Some(next) = top_statement(&t, i, &mut fi) {
                i = next;
                continue;
            }
        }
        match c {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => {
                depth -= 1;
                if depth == 0 {
                    stmt_start = true;
                }
            }
            b';' | b'\n' if depth == 0 => stmt_start = true,
            _ => {}
        }
        i += 1;
    }
    fi.reads = property_reads(&t);
    fi.masked = t.m.clone();
    fi
}

/// A top-level statement at `i`. Returns the offset to resume at when it
/// consumed a construct (bodies are skipped as a whole).
fn top_statement(t: &Text, i: usize, fi: &mut FileInfo) -> Option<usize> {
    let line = t.line(i);
    if t.kw(i, "import") {
        let mut j = t.ws(i + 6);
        if matches!(t.m.get(j), Some(b'(') | Some(b'"') | Some(b'\'') | Some(b'.')) {
            return None; // dynamic import, side-effect import, import.meta
        }
        let mut statement_type_only = false;
        if t.kw(j, "type") {
            let k = t.ws(j + 4);
            if t.m.get(k) == Some(&b'{') || t.ident(k).is_some() && !t.kw(k, "from") {
                j = k;
                statement_type_only = true;
            }
        }
        let mut named = Vec::new();
        let mut locals = Vec::new();
        let mut type_only = BTreeSet::new();
        // default identifier
        if let Some((_, e)) = t.ident(j) {
            if !t.kw(j, "from") {
                j = t.ws(e);
                if t.m.get(j) == Some(&b',') {
                    j = t.ws(j + 1);
                }
            }
        }
        if t.m.get(j) == Some(&b'*') {
            // * as ns
            let k = t.ws(j + 1);
            if t.kw(k, "as") {
                if let Some((_, e)) = t.ident(t.ws(k + 2)) {
                    j = e;
                }
            }
        } else if t.m.get(j) == Some(&b'{') {
            let close = t.close(j)?;
            locals = brace_names(t, j, close);
            named = locals.iter().map(|(_, orig)| orig.clone()).collect();
            type_only = if statement_type_only { named.iter().cloned().collect() } else { inline_type_names(t, j, close) };
            j = close;
        }
        let (spec, end) = from_spec(t, j)?;
        fi.imports.push(Import { line, spec, named, locals, type_only });
        return Some(end);
    }
    if t.kw(i, "export") {
        let mut j = t.ws(i + 6);
        // `export type { .. } from` and (SLICE4, R-S4-3) `export type * from`.
        if t.kw(j, "type") && matches!(t.m.get(t.ws(j + 4)), Some(b'{') | Some(b'*')) {
            j = t.ws(j + 4);
        }
        if t.m.get(j) == Some(&b'*') {
            let k = t.ws(j + 1);
            if t.kw(k, "as") {
                return None;
            }
            let (spec, end) = from_spec(t, k)?;
            fi.reexports.push(ReExport { line, spec, names: None });
            return Some(end);
        }
        if t.m.get(j) == Some(&b'{') {
            let close = t.close(j)?;
            let names = brace_names(t, j, close);
            if let Some((spec, end)) = from_spec(t, close) {
                fi.reexports.push(ReExport { line, spec, names: Some(names) });
                return Some(end);
            }
            fi.local_exports.extend(names);
            return Some(close);
        }
        if t.kw(j, "declare") {
            j = t.ws(j + 7);
        }
        return declaration(t, j, true, fi);
    }
    let mut j = i;
    if t.kw(j, "declare") {
        j = t.ws(j + 7);
    }
    if t.kw(j, "interface") || t.kw(j, "type") {
        return declaration(t, j, false, fi);
    }
    None
}

fn declaration(t: &Text, j: usize, exported: bool, fi: &mut FileInfo) -> Option<usize> {
    let line = t.line(j);
    if t.kw(j, "interface") {
        let (name, e) = t.ident(t.ws(j + 9))?;
        // body: first `{` outside generics/extends type arguments
        let mut k = skip_generics(t, e);
        let mut angle = 0i64;
        while k < t.m.len() {
            match t.m[k] {
                b'<' => angle += 1,
                b'>' if t.m[k - 1] != b'=' => angle -= 1,
                b'{' if angle == 0 => break,
                b';' if angle == 0 => return None,
                _ => {}
            }
            k += 1;
        }
        let (ms, end) = members(t, k)?;
        fi.decls.push(Decl { name, line, exported, kind: Kind::Object(ms) });
        return Some(end);
    }
    if t.kw(j, "type") {
        let (name, e) = t.ident(t.ws(j + 4))?;
        let k = skip_generics(t, e);
        let k = t.ws(k);
        if t.m.get(k) != Some(&b'=') {
            return None;
        }
        let kind = type_kind(t, k + 1);
        fi.decls.push(Decl { name, line, exported, kind });
        return None; // keep scanning normally; the body is balanced
    }
    None
}

/// Field names read by property access: `.f`, `?.f`, `["f"]`, `['f']`.
/// Declarations, object-literal keys, `.get("f")`, tags and rename metadata
/// are NOT reads (contrast `surface::broad_reads`).
fn property_reads(t: &Text) -> BTreeMap<String, (String, usize)> {
    let mut out: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let b = &t.m;
    let mut push = |name: String, at: usize| {
        out.entry(name_key(&name)).or_insert((name, t.line(at)));
    };
    for i in 0..b.len() {
        match b[i] {
            b'.' if i > 0 && (is_ident(b[i - 1]) || matches!(b[i - 1], b')' | b']' | b'?')) => {
                let mut k = i;
                while k > 0 && is_ident(b[k - 1]) {
                    k -= 1;
                }
                let number = k < i && b[k].is_ascii_digit();
                if let (false, Some((id, _))) = (number, t.ident(i + 1)) {
                    push(id, i);
                }
            }
            b'[' if i > 0 && (is_ident(b[i - 1]) || matches!(b[i - 1], b')' | b']')) => {
                if let Some((s, e)) = t.string_at(i + 1) {
                    if b.get(e) == Some(&b']') && !s.is_empty() {
                        push(s, i);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

// ------------------------------------------------------------ receivers
//
// INIT-DISCOVERY-SLICE5. A field read counts for shape S only through a
// receiver declared as S in the consuming file (bounds B2 to B4): an
// annotation, a cast, an explicit type argument, a call to a function of
// this file whose declared return type is S or Promise<S>, or an element of
// a receiver declared as an array of S. Writes never count. Receivers are
// names, not scopes (no parser): a name declared as S anywhere in the file
// verifies every bare read through that name.

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Id(String),
    P(u8),
}

/// Identifiers and single punctuation bytes of masked text, with offsets.
/// Numbers are skipped; string contents and comments are already blank.
fn tokens(m: &[u8]) -> Vec<(Tok, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < m.len() {
        let c = m[i];
        if is_ident_start(c) || c.is_ascii_digit() {
            let s = i;
            while i < m.len() && is_ident(m[i]) {
                i += 1;
            }
            if is_ident_start(c) {
                out.push((Tok::Id(String::from_utf8_lossy(&m[s..i]).into_owned()), s));
            }
            continue;
        }
        if !c.is_ascii_whitespace() {
            out.push((Tok::P(c), i));
        }
        i += 1;
    }
    out
}

struct Toks<'a>(&'a [(Tok, usize)]);

impl Toks<'_> {
    fn id(&self, k: usize) -> Option<&str> {
        match self.0.get(k) {
            Some((Tok::Id(s), _)) => Some(s),
            _ => None,
        }
    }
    fn is(&self, k: usize, w: &str) -> bool {
        self.id(k) == Some(w)
    }
    fn p(&self, k: usize, c: u8) -> bool {
        matches!(self.0.get(k), Some((Tok::P(x), _)) if *x == c)
    }
    /// The index after the bracket group opened at k (`open` .. `close`).
    fn skip_group(&self, mut k: usize, open: u8, close: u8) -> usize {
        let mut d = 0i32;
        while k < self.0.len() {
            if self.p(k, open) {
                d += 1;
            } else if self.p(k, close) {
                d -= 1;
                if d == 0 {
                    return k + 1;
                }
            }
            k += 1;
        }
        k
    }
}

/// A type at token k whose head is `shape`: `[readonly] [Wrapper<] S`, with
/// optional generic arguments and an optional `[]`; never `S.x` or `S[k]`.
/// Some(true) is an array of S, Some(false) is S itself.
fn type_head(t: &Toks, mut k: usize, shape: &str) -> Option<bool> {
    if t.is(k, "readonly") {
        k += 1;
    }
    let mut arr = false;
    if let Some(w) = t.id(k) {
        if matches!(w, "Readonly" | "Partial" | "Required" | "Array" | "ReadonlyArray") && t.p(k + 1, b'<') {
            arr = matches!(w, "Array" | "ReadonlyArray");
            k += 2;
            if t.is(k, "readonly") {
                k += 1;
            }
        }
    }
    if !t.is(k, shape) || t.p(k + 1, b'.') {
        return None;
    }
    k += 1;
    if t.p(k, b'<') {
        k = t.skip_group(k, b'<', b'>');
    }
    if t.p(k, b'[') {
        return if t.p(k + 1, b']') { Some(true) } else { None };
    }
    Some(arr)
}

/// Functions of this file whose declared return type is S or Promise<S>.
fn returning(t: &Toks, shape: &str) -> BTreeSet<String> {
    let mut fns = BTreeSet::new();
    for k in 0..t.0.len() {
        if !(t.p(k, b')') && t.p(k + 1, b':')) {
            continue;
        }
        let mut j = k + 2;
        if t.is(j, "Promise") && t.p(j + 1, b'<') {
            j += 2;
        }
        if type_head(t, j, shape) != Some(false) {
            continue;
        }
        // back to the matching '(' and the name before it
        let mut d = 0i32;
        let mut o = k;
        loop {
            if t.p(o, b')') {
                d += 1;
            } else if t.p(o, b'(') {
                d -= 1;
                if d == 0 {
                    break;
                }
            }
            if o == 0 {
                break;
            }
            o -= 1;
        }
        if o == 0 {
            continue;
        }
        if t.p(o - 1, b'>') {
            // `name<T>(`: step over the type parameters
            let mut d = 0i32;
            while o > 0 {
                o -= 1;
                if t.p(o, b'>') {
                    d += 1;
                } else if t.p(o, b'<') {
                    d -= 1;
                    if d == 0 {
                        break;
                    }
                }
            }
            if o == 0 {
                continue;
            }
        }
        let name = match t.id(o - 1) {
            Some("async") if o >= 3 && t.p(o - 2, b'=') => t.id(o - 3),
            Some(n) if n != "function" => Some(n),
            None if o >= 2 && t.p(o - 1, b'=') => t.id(o - 2),
            _ => None,
        };
        if let Some(n) = name {
            fns.insert(n.to_string());
        }
    }
    fns
}

/// The shape's member fields found in `reads`: (distinctive, stoplisted).
fn collect_fields(members: &[(String, usize)], reads: &BTreeMap<String, (String, usize)>, stop: &BTreeSet<String>) -> (Vec<(String, usize)>, Vec<String>) {
    let mut fields = Vec::new();
    let mut stoplisted = Vec::new();
    for (m, _) in members {
        let k = name_key(m);
        if let Some((_, line)) = reads.get(&k) {
            if stop.contains(&k) {
                stoplisted.push(m.clone());
            } else {
                fields.push((m.clone(), *line));
            }
        }
    }
    fields.sort();
    fields.dedup_by(|a, b| a.0 == b.0);
    stoplisted.sort();
    stoplisted.dedup();
    (fields, stoplisted)
}

/// SLICE7: the routes available in one consumer file for shape `s` of
/// package `p`: local function name -> (route, provenance). A route exists
/// only when the shape's name is declared once in the package, and the
/// imported function is declared there exactly once, with a permitted
/// written return type whose head is `s` (B2).
fn route_fns(
    p: usize,
    s: &str,
    locals: &BTreeMap<usize, Vec<(String, String, usize, String)>>,
    funcs: &BTreeMap<usize, BTreeMap<String, Vec<(String, usize, Option<(String, RetForm)>)>>>,
    shape_names: &BTreeMap<usize, BTreeMap<String, usize>>,
) -> BTreeMap<String, (Route, Via)> {
    let mut out = BTreeMap::new();
    if shape_names.get(&p).and_then(|m| m.get(s)) != Some(&1) {
        return out;
    }
    let (Some(ls), Some(fs)) = (locals.get(&p), funcs.get(&p)) else { return out };
    for (local, orig, line, spec) in ls {
        let Some(ds) = fs.get(orig) else { continue };
        if let [(file, dl, Some((head, form)))] = ds.as_slice() {
            if head == s {
                let via = Via { function: orig.clone(), decl_file: file.clone(), decl_line: *dl, import_line: *line, specifier: spec.clone() };
                out.insert(local.clone(), ((*form, file.clone(), *dl), via));
            }
        }
    }
    out
}

/// The `via` list of a shape: the routes its counted fields were read
/// through, in function order.
fn used_routes(fields: &[(String, usize)], routes: &RouteReads, avail: &BTreeMap<String, (Route, Via)>) -> Vec<Via> {
    let mut used: BTreeSet<Via> = BTreeSet::new();
    for (m, _) in fields {
        for local in routes.get(&name_key(m)).into_iter().flatten() {
            if let Some((_, v)) = avail.get(local) {
                used.insert(v.clone());
            }
        }
    }
    used.into_iter().collect()
}

// --------------------------------------------------- property routes (SLICE8)
//
// A read counts for shape S through a property whose written type is S, at
// most two hops from a receiver with a written type T in the consumer file:
// `r.p.f`, `r.a.p.f`. T is declared in that file or imported by name and
// resolved by the existing resolver; every member type is read as written
// and must be exactly S, Readonly<S>, S[] or Array<S> (an inline object
// type is allowed as the middle hop), and S must resolve to the very shape,
// not to another of the same name. Types are never inferred.

/// A member's written type, as far as SLICE8 reads it.
#[derive(Debug, Clone)]
enum MType {
    /// (type name, is an array of it)
    Named(String, bool),
    /// `{ ... }`: its members, (name, line, type)
    Inline(Vec<(String, usize, MType)>),
}

/// One property path from a receiver type to the shape.
#[derive(Debug, Clone)]
struct PropPath {
    /// The receiver's type, as written in the consumer file.
    ty: String,
    chain: Vec<String>,
    end_arr: bool,
    prov: Prop,
}

/// SLICE8 provenance of one property path: `Ty.a.p`, the declaring file
/// and line of its last member, and where Ty comes from in the consumer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prop {
    pub path: String,
    pub decl_file: String,
    pub decl_line: usize,
    pub origin: String,
}

/// The written type starting at token `j`, strictly: [Readonly<|Array<] H
/// [<..>] [[]] [>], then the end of the member.
fn member_type(t: &Toks, m: &[u8], mut j: usize, depth_end: usize) -> Option<MType> {
    let ends = |k: usize| {
        k >= depth_end
            || matches!(t.0.get(k), Some((Tok::P(b';' | b',' | b'}'), _)))
            || (k > 0 && m[t.0[k - 1].1..t.0[k].1].contains(&b'\n') && !matches!(t.0.get(k), Some((Tok::P(b'|' | b'&' | b'.'), _))))
    };
    if t.p(j, b'{') {
        let close = t.skip_group(j, b'{', b'}');
        let inner = inline_members(t, m, j, close);
        return ends(close).then_some(MType::Inline(inner));
    }
    let mut wrap = 0;
    let mut arr = false;
    if matches!(t.id(j), Some("Readonly" | "Array")) && t.p(j + 1, b'<') {
        arr = t.is(j, "Array");
        j += 2;
        wrap = 1;
    }
    let head = t.id(j)?.to_string();
    if t.p(j + 1, b'.') {
        return None;
    }
    j += 1;
    if t.p(j, b'<') {
        j = t.skip_group(j, b'<', b'>');
    }
    if t.p(j, b'[') && t.p(j + 1, b']') {
        if wrap > 0 {
            return None;
        }
        arr = true;
        j += 2;
    }
    if wrap > 0 {
        if !t.p(j, b'>') {
            return None;
        }
        j += 1;
    }
    ends(j).then_some(MType::Named(head, arr))
}

/// Members `name[?]: type` directly inside the brace group `open`..`close`.
fn inline_members(t: &Toks, m: &[u8], open: usize, close: usize) -> Vec<(String, usize, MType)> {
    let mut out = Vec::new();
    let mut d = 0i32;
    let mut k = open;
    while k < close {
        if t.p(k, b'{') || t.p(k, b'(') || t.p(k, b'[') {
            d += 1;
        } else if t.p(k, b'}') || t.p(k, b')') || t.p(k, b']') {
            d -= 1;
        } else if d == 1 {
            let start = k == open + 1 || matches!(t.0.get(k - 1), Some((Tok::P(b';' | b','), _))) || m[t.0[k - 1].1..t.0[k].1].contains(&b'\n');
            let n = if t.is(k, "readonly") && t.id(k + 1).is_some() { k + 1 } else { k };
            if let (true, Some(name)) = (start, t.id(n)) {
                let colon = if t.p(n + 1, b'?') { n + 2 } else { n + 1 };
                if t.p(colon, b':') {
                    if let Some(ty) = member_type(t, m, colon + 1, close - 1) {
                        let line = m[..t.0[n].1].iter().filter(|&&b| b == b'\n').count() + 1;
                        out.push((name.to_string(), line, ty));
                    }
                }
            }
        }
        k += 1;
    }
    out
}

/// The members of an object declaration with their written types.
fn decl_member_types(fi: &FileInfo, decl: &Decl) -> Vec<(String, usize, MType)> {
    let m = &fi.masked;
    let all = tokens(m);
    let t = Toks(&all);
    let line = |k: usize| m[..all[k].1].iter().filter(|&&b| b == b'\n').count() + 1;
    for k in 0..all.len() {
        if !(t.is(k, "interface") || t.is(k, "type")) || !t.is(k + 1, &decl.name) || line(k) != decl.line {
            continue;
        }
        let mut j = k + 2;
        if t.p(j, b'<') {
            j = t.skip_group(j, b'<', b'>');
        }
        while j < all.len() && !t.p(j, b'{') && !t.p(j, b';') {
            j += 1;
        }
        if !t.p(j, b'{') {
            return Vec::new();
        }
        let close = t.skip_group(j, b'{', b'}');
        return inline_members(&t, m, j, close);
    }
    Vec::new()
}

/// Type lookups for SLICE8, memoized per (file, name).
struct PropIndex<'c, 'i> {
    ctx: &'c Ctx<'c>,
    infos: &'i BTreeMap<String, FileInfo>,
    types: BTreeMap<(String, String), Option<(String, Decl)>>,
    members: BTreeMap<(String, String), Vec<(String, usize, MType)>>,
}

impl PropIndex<'_, '_> {
    /// The object declaration a type name written in `file` denotes: a
    /// single declaration of that name in the file, else a named import of
    /// it resolved by the existing resolver to exactly one declaration.
    fn resolve(&mut self, file: &str, name: &str) -> Option<(String, Decl)> {
        let key = (file.to_string(), name.to_string());
        if let Some(r) = self.types.get(&key) {
            return r.clone();
        }
        let fi = self.infos.get(file);
        let r = fi.and_then(|fi| {
            let local: Vec<&Decl> = fi.decls.iter().filter(|d| d.name == name).collect();
            match local.as_slice() {
                [d] => return matches!(d.kind, Kind::Object(_)).then(|| (file.to_string(), (*d).clone())),
                [] => {}
                _ => return None,
            }
            let imps: Vec<(&Import, &String)> = fi.imports.iter().flat_map(|i| i.locals.iter().filter(|(l, _)| l == name).map(move |(_, o)| (i, o))).collect();
            let [(imp, orig)] = imps.as_slice() else { return None };
            let (_, Res::File(target)) = self.ctx.resolve(&imp.spec, file, false) else { return None };
            match find_shape(self.ctx, self.infos, orig, &target) {
                Found::Decl { file: d, decl, .. } if matches!(decl.kind, Kind::Object(_)) => Some((d, decl)),
                _ => None,
            }
        });
        self.types.insert(key, r.clone());
        r
    }
    fn members_of(&mut self, file: &str, decl: &Decl) -> Vec<(String, usize, MType)> {
        let key = (file.to_string(), decl.name.clone());
        if !self.members.contains_key(&key) {
            let ms = self.infos.get(file).map(|fi| decl_member_types(fi, decl)).unwrap_or_default();
            self.members.insert(key.clone(), ms);
        }
        self.members[&key].clone()
    }
    /// Every path of at most two hops from a type written in consumer `f`
    /// to the shape `s` declared in `g`.
    fn paths(&mut self, f: &str, s: &str, g: &str) -> Vec<PropPath> {
        let Some(fi) = self.infos.get(f) else { return Vec::new() };
        let mut names: BTreeMap<String, String> = BTreeMap::new();
        for d in &fi.decls {
            names.entry(d.name.clone()).or_insert_with(|| format!("{} declared at {f}:{}", d.name, d.line));
        }
        for imp in &fi.imports {
            for (local, _) in &imp.locals {
                names.entry(local.clone()).or_insert_with(|| format!("{local} imported at {f}:{} from {:?}", imp.line, imp.spec));
            }
        }
        let mut out = Vec::new();
        for (ty, origin) in names {
            let Some((tf, td)) = self.resolve(f, &ty) else { continue };
            if tf == g && td.name == s {
                continue;
            }
            for (p, pl, pt) in self.members_of(&tf, &td) {
                let mut hop2: Vec<(String, String, usize, String, bool)> = Vec::new(); // (a, p2, line, file, arr)
                match &pt {
                    MType::Named(h, arr) => {
                        if self.resolve(&tf, h).is_some_and(|(hf, hd)| hf == g && hd.name == s) {
                            out.push(PropPath {
                                ty: ty.clone(),
                                chain: vec![p.clone()],
                                end_arr: *arr,
                                prov: Prop { path: format!("{ty}.{p}"), decl_file: tf.clone(), decl_line: pl, origin: origin.clone() },
                            });
                            continue;
                        }
                        if *arr {
                            continue;
                        }
                        if let Some((uf, ud)) = self.resolve(&tf, h) {
                            for (p2, l2, t2) in self.members_of(&uf, &ud) {
                                if let MType::Named(h2, a2) = t2 {
                                    if self.resolve(&uf, &h2).is_some_and(|(hf, hd)| hf == g && hd.name == s) {
                                        hop2.push((p.clone(), p2, l2, uf.clone(), a2));
                                    }
                                }
                            }
                        }
                    }
                    MType::Inline(ms) => {
                        for (p2, l2, t2) in ms {
                            if let MType::Named(h2, a2) = t2 {
                                if self.resolve(&tf, h2).is_some_and(|(hf, hd)| hf == g && hd.name == s) {
                                    hop2.push((p.clone(), p2.clone(), *l2, tf.clone(), *a2));
                                }
                            }
                        }
                    }
                }
                for (a, p2, l2, file, a2) in hop2 {
                    out.push(PropPath {
                        ty: ty.clone(),
                        chain: vec![a.clone(), p2.clone()],
                        end_arr: a2,
                        prov: Prop { path: format!("{ty}.{a}.{p2}"), decl_file: file, decl_line: l2, origin: origin.clone() },
                    });
                }
            }
        }
        out
    }
}

/// Reads of the shape's fields through property paths in this file, and
/// per field key the paths (by index into `paths`) they were read through.
fn path_reads(fi: &FileInfo, paths: &[PropPath]) -> (BTreeMap<String, (String, usize)>, BTreeMap<String, BTreeSet<usize>>) {
    let m = &fi.masked;
    let all = tokens(m);
    let t = Toks(&all);
    let mut out: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let mut used: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    // the end of `base[..]?.a?.p` at token k, if it matches the path
    // (`?.` as the field reader sees it; `!.` is not a read there)
    let walk = |k: usize, base_arr: bool, chain: &[String]| -> Option<usize> {
        let mut j = k + 1;
        if base_arr {
            if !t.p(j, b'[') {
                return None;
            }
            j = t.skip_group(j, b'[', b']');
        }
        for c in chain {
            if t.p(j, b'?') {
                j += 1;
            }
            if !(t.p(j, b'.') && t.is(j + 1, c)) {
                return None;
            }
            j += 2;
        }
        Some(j)
    };
    let mut read = |j: usize, i: usize| {
        let mut j = j;
        if t.p(j, b'?') {
            j += 1;
        }
        if !t.p(j, b'.') {
            return;
        }
        let Some(f) = t.id(j + 1) else { return };
        let off = all[j + 1].1;
        if !written(m, off) {
            let line = m[..off].iter().filter(|&&b| b == b'\n').count() + 1;
            out.entry(name_key(f)).or_insert((f.to_string(), line));
            used.entry(name_key(f)).or_default().insert(i);
        }
    };
    for (i, pp) in paths.iter().enumerate() {
        let (one, arr, _) = receivers(&t, m, &pp.ty, &BTreeMap::new());
        // values of the path: (alias or element name, is an array of S)
        let mut vals: Vec<(String, bool)> = Vec::new();
        for k in 0..all.len() {
            let Some(r) = t.id(k) else { continue };
            if k > 0 && t.p(k - 1, b'.') {
                continue;
            }
            for base_arr in [false, true] {
                if !(if base_arr { arr.contains(r) } else { one.contains(r) }) {
                    continue;
                }
                let Some(j) = walk(k, base_arr, &pp.chain) else { continue };
                if pp.end_arr {
                    if t.p(j, b'[') {
                        read(t.skip_group(j, b'[', b']'), i);
                    }
                    // `for (const e of <path>)` and `<path>.map((e) =>`
                    if k >= 5 && t.is(k - 1, "of") && t.p(k - 4, b'(') && t.is(k - 5, "for") && t.p(j, b')') {
                        if let Some(e) = t.id(k - 2) {
                            vals.push((e.to_string(), false));
                        }
                    }
                    if let Some(e) = callback_param(&t, j) {
                        vals.push((e, false));
                    }
                } else {
                    read(j, i);
                }
                // an alias `y = <path>` that ends the expression
                if k >= 2 && t.p(k - 1, b'=') && !t.p(k - 2, b'=') && !t.p(k - 2, b'!') {
                    let ends = j >= all.len()
                        || matches!(&all[j].0, Tok::P(b';' | b',' | b')'))
                        || m[all[j - 1].1..all[j].1].contains(&b'\n');
                    if let (true, Some(y)) = (ends, t.id(k - 2)) {
                        vals.push((y.to_string(), pp.end_arr));
                    }
                }
            }
        }
        // the values' own reads: `y.f`, `y[i].f`, and their elements
        let mut more = Vec::new();
        for (y, is_arr) in &vals {
            if !*is_arr {
                continue;
            }
            for k in 0..all.len() {
                if !t.is(k, y) || (k > 0 && t.p(k - 1, b'.')) {
                    continue;
                }
                if k >= 5 && t.is(k - 1, "of") && t.p(k - 4, b'(') && t.is(k - 5, "for") && t.p(k + 1, b')') {
                    if let Some(e) = t.id(k - 2) {
                        more.push((e.to_string(), false));
                    }
                }
                if let Some(e) = callback_param(&t, k + 1) {
                    more.push((e, false));
                }
            }
        }
        vals.extend(more);
        for (y, is_arr) in &vals {
            for k in 0..all.len() {
                if !t.is(k, y) || (k > 0 && t.p(k - 1, b'.')) {
                    continue;
                }
                if *is_arr {
                    if t.p(k + 1, b'[') {
                        read(t.skip_group(k + 1, b'[', b']'), i);
                    }
                } else {
                    read(k + 1, i);
                }
            }
        }
    }
    (out, used)
}

/// The fields of a shape counted through its declared receivers, its SLICE7
/// routes and its property paths together, when they reach MIN_FIELDS:
/// (fields, stoplisted, routes used, paths used).
type WithPaths = (Vec<(String, usize)>, Vec<String>, Vec<Via>, Vec<Prop>);

fn with_paths(
    fi: &FileInfo,
    shape: &str,
    members: &[(String, usize)],
    stop: &BTreeSet<String>,
    avail: &BTreeMap<String, (Route, Via)>,
    paths: &[PropPath],
) -> Option<WithPaths> {
    let routes_only: BTreeMap<String, Route> = avail.iter().map(|(k, (r, _))| (k.clone(), r.clone())).collect();
    let (mut reads, rr) = receiver_reads(fi, shape, &routes_only);
    let (pr, used) = path_reads(fi, paths);
    for (k, (s, l)) in pr {
        let e = reads.entry(k).or_insert((s, l));
        e.1 = e.1.min(l);
    }
    let (fields, stoplisted) = collect_fields(members, &reads, stop);
    (fields.len() >= MIN_FIELDS).then(|| {
        let via = used_routes(&fields, &rr, avail);
        let props = used_props(&fields, &used, paths);
        (fields, stoplisted, via, props)
    })
}

/// `[?].map((e)` / `.filter(e =>` ... at token j: the callback's parameter.
fn callback_param(t: &Toks, j: usize) -> Option<String> {
    let dot = if t.p(j, b'?') { j + 1 } else { j };
    if !(t.p(dot, b'.')
        && matches!(t.id(dot + 1), Some("map" | "filter" | "forEach" | "find" | "findLast" | "findIndex" | "some" | "every" | "flatMap" | "sort"))
        && t.p(dot + 2, b'('))
    {
        return None;
    }
    let q = if t.p(dot + 3, b'(') { dot + 4 } else { dot + 3 };
    t.id(q).map(str::to_string)
}

/// The provenance of the paths a shape's counted fields were read through.
fn used_props(fields: &[(String, usize)], used: &BTreeMap<String, BTreeSet<usize>>, paths: &[PropPath]) -> Vec<Prop> {
    let mut out: BTreeSet<Prop> = BTreeSet::new();
    for (f, _) in fields {
        for i in used.get(&name_key(f)).into_iter().flatten() {
            out.insert(paths[*i].prov.clone());
        }
    }
    out.into_iter().collect()
}

// ------------------------------------------------------- return types (SLICE7)
//
// A function declaration's written return type, read strictly: exactly S,
// Readonly<S>, Promise<S>, S[] or Array<S> (S may carry its own type
// arguments), followed by the body, an arrow or the end of the
// declaration. Anything else (a union, Partial<S>, Promise<S[]>, Box<S>)
// is a declaration with no usable return type: it still counts toward
// uniqueness, and it ties nothing.

#[derive(Debug, Clone, Copy, PartialEq)]
enum RetForm {
    One,
    Arr,
    Promise,
}

/// A route a receiver can be tied through: (form, declaring file, line).
type Route = (RetForm, String, usize);

/// Every function declaration in this file: (name, line, written return
/// type as (head, form) when it is one of the permitted forms).
fn function_decls(fi: &FileInfo) -> Vec<(String, usize, Option<(String, RetForm)>)> {
    let m = &fi.masked;
    let all = tokens(m);
    let t = Toks(&all);
    let line = |k: usize| m[..all[k].1].iter().filter(|&&b| b == b'\n').count() + 1;
    let mut out = Vec::new();
    for k in 0..all.len() {
        // `function name(` / `function name<T>(`
        let (name, open) = if t.is(k, "function") {
            let j = if t.p(k + 1, b'*') { k + 2 } else { k + 1 };
            let Some(n) = t.id(j) else { continue };
            let o = if t.p(j + 1, b'<') { t.skip_group(j + 1, b'<', b'>') } else { j + 1 };
            (n, o)
        } else if matches!(t.id(k), Some("const" | "let")) && t.id(k + 1).is_some() && t.p(k + 2, b'=') && !t.p(k + 3, b'=') {
            // `const name = (` / `= async (` / `= <T>(`
            let mut o = k + 3;
            if t.is(o, "async") {
                o += 1;
            }
            if t.p(o, b'<') {
                o = t.skip_group(o, b'<', b'>');
            }
            (t.id(k + 1).unwrap(), o)
        } else {
            continue;
        };
        if !t.p(open, b'(') {
            continue;
        }
        let close = t.skip_group(open, b'(', b')');
        let ret = if t.p(close, b':') { return_form(&t, close + 1) } else { None };
        out.push((name.to_string(), line(k), ret));
    }
    out
}

fn return_form(t: &Toks, mut j: usize) -> Option<(String, RetForm)> {
    let mut form = RetForm::One;
    let mut wrap = 0;
    if t.is(j, "Promise") && t.p(j + 1, b'<') {
        form = RetForm::Promise;
        j += 2;
        wrap += 1;
    } else if matches!(t.id(j), Some("Readonly" | "Array")) && t.p(j + 1, b'<') {
        if t.is(j, "Array") {
            form = RetForm::Arr;
        }
        j += 2;
        wrap += 1;
    }
    let head = t.id(j)?.to_string();
    if t.p(j + 1, b'.') {
        return None;
    }
    j += 1;
    if t.p(j, b'<') {
        j = t.skip_group(j, b'<', b'>');
    }
    if t.p(j, b'[') && t.p(j + 1, b']') {
        if wrap > 0 {
            return None;
        }
        form = RetForm::Arr;
        j += 2;
    }
    for _ in 0..wrap {
        if !t.p(j, b'>') {
            return None;
        }
        j += 1;
    }
    let end = t.p(j, b'{') || t.p(j, b';') || (t.p(j, b'=') && t.p(j + 1, b'>')) || j >= t.0.len();
    end.then_some((head, form))
}

/// Receivers declared as `shape`: (single values, arrays of S).
fn receivers(t: &Toks, m: &[u8], shape: &str, imported: &BTreeMap<String, Route>) -> (BTreeSet<String>, BTreeSet<String>, BTreeMap<String, String>) {
    let fns = returning(t, shape);
    // SLICE7: receiver -> the imported function that tied it
    let mut routed: BTreeMap<String, String> = BTreeMap::new();
    let mut one: BTreeSet<String> = BTreeSet::new();
    let mut arr: BTreeSet<String> = BTreeSet::new();
    fn put(one: &mut BTreeSet<String>, arr: &mut BTreeSet<String>, name: &str, is_arr: bool) {
        if is_arr {
            arr.insert(name.to_string());
        } else {
            one.insert(name.to_string());
        }
    }
    for k in 0..t.0.len() {
        let Some(name) = t.id(k) else { continue };
        if k > 0 && t.p(k - 1, b'.') {
            continue;
        }
        // an annotation `r: S`, `r?: S`
        let colon = if t.p(k + 1, b'?') { k + 2 } else { k + 1 };
        if t.p(colon, b':') {
            if let Some(a) = type_head(t, colon + 1, shape) {
                put(&mut one, &mut arr, name, a);
            }
            continue;
        }
        // `r = <expression>`, not `==` or `=>`
        if !t.p(k + 1, b'=') || t.p(k + 2, b'=') || t.p(k + 2, b'>') {
            continue;
        }
        let start = k + 2;
        let mut end = start;
        let mut d = 0i32;
        while end < t.0.len() {
            if t.p(end, b'(') || t.p(end, b'[') || t.p(end, b'{') {
                d += 1;
            } else if t.p(end, b')') || t.p(end, b']') || t.p(end, b'}') {
                if d == 0 {
                    break;
                }
                d -= 1;
            } else if d == 0 && t.p(end, b';') {
                break;
            }
            end += 1;
        }
        // a cast `r = ... as S`
        for q in start..end {
            if t.is(q, "as") {
                if let Some(a) = type_head(t, q + 1, shape) {
                    put(&mut one, &mut arr, name, a);
                }
            }
        }
        let mut q = if t.is(start, "await") { start + 1 } else { start };
        // a local function returning S: `r = [await] [this.]f(`
        let local = if t.is(q, "this") && t.p(q + 1, b'.') { q + 2 } else { q };
        if t.id(local).is_some_and(|f| fns.contains(f)) && t.p(local + 1, b'(') {
            put(&mut one, &mut arr, name, false);
        }
        // SLICE7: `r = [await] f(...)` with f imported (B2); the call must end
        // the expression: nothing chained onto its result.
        if let Some(f) = t.id(q) {
            let awaited = q == start + 1 && t.is(start, "await");
            if let (Some((form, _, _)), true, true) = (imported.get(f), q == start || awaited, t.p(q + 1, b'(')) {
                let close = t.skip_group(q + 1, b'(', b')');
                let ends = close >= t.0.len()
                    || matches!(&t.0[close].0, Tok::P(b';') | Tok::P(b',') | Tok::P(b')') | Tok::P(b'}') | Tok::P(b']'))
                    || (matches!(&t.0[close].0, Tok::Id(_)) && m[t.0[close - 1].1..t.0[close].1].contains(&b'\n'));
                if ends && (*form != RetForm::Promise || awaited) {
                    put(&mut one, &mut arr, name, *form == RetForm::Arr);
                    routed.insert(name.to_string(), f.to_string());
                }
            }
        }
        // an explicit type argument: `r = [await] a.b.f<S ...>(`
        while t.id(q).is_some() && t.p(q + 1, b'.') {
            q += 2;
        }
        if t.id(q).is_some() && t.p(q + 1, b'<') && type_head(t, q + 2, shape) == Some(false) && t.p(t.skip_group(q + 1, b'<', b'>'), b'(') {
            put(&mut one, &mut arr, name, false);
        }
    }
    // an explicit type argument, destructured: `[r, ...] = f<S>(`
    for k in 0..t.0.len() {
        let (true, Some(r), true) = (t.p(k, b'['), t.id(k + 1), t.p(k + 2, b',')) else { continue };
        let close = t.skip_group(k, b'[', b']');
        if !t.p(close, b'=') || t.p(close + 1, b'=') {
            continue;
        }
        let mut q = close + 1;
        while t.id(q).is_some() && t.p(q + 1, b'.') {
            q += 2;
        }
        if t.id(q).is_some() && t.p(q + 1, b'<') && type_head(t, q + 2, shape) == Some(false) {
            put(&mut one, &mut arr, r, false);
        }
    }
    // elements of a receiver declared as an array of S
    let arrays = arr.clone();
    for k in 0..t.0.len() {
        if t.is(k, "for") && t.p(k + 1, b'(') && matches!(t.id(k + 2), Some("const" | "let" | "var")) && t.is(k + 4, "of") {
            if let (Some(r), Some(a)) = (t.id(k + 3), t.id(k + 5)) {
                if arrays.contains(a) && t.p(k + 6, b')') {
                    put(&mut one, &mut arr, r, false);
                    if let Some(f) = routed.get(a).cloned() {
                        routed.insert(r.to_string(), f);
                    }
                }
            }
        }
        let Some(a) = t.id(k) else { continue };
        if !arrays.contains(a) || (k > 0 && t.p(k - 1, b'.')) {
            continue;
        }
        let dot = if t.p(k + 1, b'?') { k + 2 } else { k + 1 };
        if t.p(dot, b'.')
            && matches!(
                t.id(dot + 1),
                Some("map" | "filter" | "forEach" | "find" | "findLast" | "findIndex" | "some" | "every" | "flatMap" | "sort")
            )
            && t.p(dot + 2, b'(')
        {
            let q = if t.p(dot + 3, b'(') { dot + 4 } else { dot + 3 };
            if let Some(r) = t.id(q) {
                put(&mut one, &mut arr, r, false);
                if let Some(f) = routed.get(a).cloned() {
                    routed.insert(r.to_string(), f);
                }
            }
        }
    }
    (one, arr, routed)
}

/// Is the property access whose name starts at byte `off` a write?
fn written(m: &[u8], mut i: usize) -> bool {
    while i < m.len() && is_ident(m[i]) {
        i += 1;
    }
    while i < m.len() && m[i].is_ascii_whitespace() {
        i += 1;
    }
    let r = &m[i..m.len().min(i + 3)];
    (r.first() == Some(&b'=') && !matches!(r.get(1), Some(b'=' | b'>')))
        || [&b"+="[..], b"-=", b"*=", b"/=", b"%=", b"++", b"--", b"??=", b"||=", b"&&="].iter().any(|w| r.starts_with(w))
}

/// Reads (name_key -> (spelling, first line)) of fields through receivers
/// declared as `shape` in this file: `r.f`, `r?.f`, `this.r.f`, `a[i].f`.
/// Reads per field key, and (SLICE7) per field key the imported functions
/// whose returned values it was read off.
type RouteReads = BTreeMap<String, BTreeSet<String>>;

fn receiver_reads(fi: &FileInfo, shape: &str, imported: &BTreeMap<String, Route>) -> (BTreeMap<String, (String, usize)>, RouteReads) {
    let m = &fi.masked;
    let all = tokens(m);
    let t = Toks(&all);
    let (one, arr, routed) = receivers(&t, m, shape, imported);
    let mut routes: RouteReads = BTreeMap::new();
    let mut out: BTreeMap<String, (String, usize)> = BTreeMap::new();
    for k in 0..all.len() {
        let Some(r) = t.id(k) else { continue };
        let via_this = k >= 2 && t.p(k - 1, b'.') && t.is(k - 2, "this");
        if k > 0 && t.p(k - 1, b'.') && !via_this {
            continue;
        }
        let mut j = k + 1;
        if arr.contains(r) && t.p(j, b'[') {
            j = t.skip_group(j, b'[', b']');
        } else if !one.contains(r) {
            continue;
        }
        if t.p(j, b'?') {
            j += 1;
        }
        if !t.p(j, b'.') {
            continue;
        }
        let Some(f) = t.id(j + 1) else { continue };
        let off = all[j + 1].1;
        if !written(m, off) {
            let line = m[..off].iter().filter(|&&b| b == b'\n').count() + 1;
            out.entry(name_key(f)).or_insert((f.to_string(), line));
            if !via_this {
                if let Some(func) = routed.get(r) {
                    routes.entry(name_key(f)).or_default().insert(func.clone());
                }
            }
        }
    }
    (out, routes)
}

// ------------------------------------------------------------ membership

fn norm_entry(raw: &str) -> Result<Vec<String>, String> {
    if raw.chars().any(|c| matches!(c, '{' | '}' | '[' | ']' | '(' | ')' | '!')) {
        return Err("unsupported glob syntax".into());
    }
    let mut segs: Vec<String> = Vec::new();
    for s in raw.split('/') {
        match s {
            "" | "." => {}
            ".." => {
                if segs.pop().is_none() {
                    return Err("resolves outside the repository root".into());
                }
            }
            x => segs.push(x.to_string()),
        }
    }
    Ok(segs)
}

fn wild(p: &[u8], s: &[u8]) -> bool {
    match (p.first(), s.first()) {
        (None, None) => true,
        (Some(b'*'), _) => wild(&p[1..], s) || (!s.is_empty() && wild(p, &s[1..])),
        (Some(b'?'), Some(_)) => wild(&p[1..], &s[1..]),
        (Some(a), Some(b)) if a == b => wild(&p[1..], &s[1..]),
        _ => false,
    }
}

fn subdirs(root: &Path, rel: &str) -> Vec<String> {
    let dir = if rel.is_empty() { root.to_path_buf() } else { root.join(rel) };
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            if !ft.is_dir() || ft.is_symlink() {
                continue;
            }
            let n = e.file_name().to_string_lossy().to_string();
            if n == "node_modules" || n == ".git" {
                continue;
            }
            out.push(if rel.is_empty() { n } else { format!("{rel}/{n}") });
        }
    }
    out.sort();
    out
}

fn expand(root: &Path, segs: &[String], truncated: &mut bool) -> Vec<String> {
    let mut cur: Vec<String> = vec![String::new()];
    for seg in segs {
        let mut next = Vec::new();
        if seg == "**" {
            for c in &cur {
                let mut stack = vec![c.clone()];
                while let Some(d) = stack.pop() {
                    if next.len() >= MAX_DIRS {
                        *truncated = true;
                        break;
                    }
                    for s in subdirs(root, &d) {
                        let leaf = s.rsplit('/').next().unwrap_or("");
                        if !leaf.starts_with('.') {
                            stack.push(s);
                        }
                    }
                    next.push(d);
                }
            }
        } else if seg.contains('*') || seg.contains('?') {
            for c in &cur {
                for s in subdirs(root, c) {
                    let leaf = s.rsplit('/').next().unwrap_or("");
                    if leaf.starts_with('.') && !seg.starts_with('.') {
                        continue;
                    }
                    if wild(seg.as_bytes(), leaf.as_bytes()) {
                        next.push(s);
                    }
                }
            }
        } else {
            for c in &cur {
                let d = if c.is_empty() { seg.clone() } else { format!("{c}/{seg}") };
                let p = root.join(&d);
                if std::fs::symlink_metadata(&p).map(|m| m.is_dir()).unwrap_or(false) {
                    next.push(d);
                }
            }
        }
        next.sort();
        next.dedup();
        cur = next;
    }
    cur
}

fn string_list(v: &Value) -> Option<Vec<String>> {
    v.as_array()?
        .iter()
        .map(|x| x.as_str().map(str::to_string))
        .collect()
}

/// Qualifying packages (committed H2 rule, rows 1 and 2), or Err after a
/// repository abstention was recorded.
fn membership(
    root: &Path,
    skip_dirs: &[&str],
    ab: &mut Vec<Abstention>,
    truncated: &mut bool,
) -> Result<Vec<Package>, Vec<Package>> {
    let repo = |reason: String| Abstention { code: "H2", location: ".".into(), reason };
    let mut entries: Vec<(String, String)> = Vec::new(); // (marker, raw entry)
    let pj = root.join("package.json");
    if pj.is_file() {
        let text = std::fs::read_to_string(&pj).unwrap_or_default();
        let v: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                ab.push(repo(format!("unparsable marker package.json ({e})")));
                return Err(Vec::new());
            }
        };
        if let Some(w) = v.get("workspaces") {
            let list = match w {
                Value::Object(o) => o.get("packages").and_then(string_list),
                other => string_list(other),
            };
            match list {
                Some(l) => entries.extend(l.into_iter().map(|e| ("package.json \"workspaces\"".into(), e))),
                None => {
                    ab.push(repo(
                        "unparsable marker package.json (\"workspaces\" is not a list of strings)".into(),
                    ));
                    return Err(Vec::new());
                }
            }
        }
    }
    let pn = root.join("pnpm-workspace.yaml");
    if pn.is_file() {
        let text = std::fs::read_to_string(&pn).unwrap_or_default();
        match crate::yaml::parse_yaml(&text) {
            Err(e) => {
                ab.push(repo(format!("unparsable marker pnpm-workspace.yaml ({e})")));
                return Err(Vec::new());
            }
            Ok(v) => {
                if let Some(p) = v.get("packages") {
                    // Slice 1 contract (U10): only a block list under
                    // `packages:` is accepted. A flow list or any other
                    // same-line form is abstained, never interpreted.
                    let same_line = text.lines().any(|l| {
                        let l = l.split('#').next().unwrap_or("");
                        l.strip_prefix("packages:").is_some_and(|rest| !rest.trim().is_empty())
                    });
                    if same_line {
                        ab.push(repo(
                            "unsupported marker pnpm-workspace.yaml (packages is not a block list)".into(),
                        ));
                        return Err(Vec::new());
                    }
                    match string_list(p) {
                        Some(l) => entries.extend(l.into_iter().map(|e| ("pnpm-workspace.yaml".into(), e))),
                        None => {
                            ab.push(repo(
                                "unparsable marker pnpm-workspace.yaml (\"packages\" is not a list of strings)".into(),
                            ));
                            return Err(Vec::new());
                        }
                    }
                }
            }
        }
    }
    if entries.is_empty() {
        let mut other = Vec::new();
        for f in ["lerna.json", "rush.json", "go.work"] {
            if root.join(f).is_file() {
                other.push(f.to_string());
            }
        }
        if std::fs::read_to_string(root.join("Cargo.toml"))
            .map(|t| t.lines().any(|l| l.trim() == "[workspace]"))
            .unwrap_or(false)
        {
            other.push("Cargo.toml [workspace]".into());
        }
        if !other.is_empty() {
            ab.push(Abstention {
                code: "SLICE-SCOPE",
                location: ".".into(),
                reason: format!(
                    "workspace marker(s) {} are outside slice 1 (package.json \"workspaces\" and pnpm-workspace.yaml only)",
                    other.join(", ")
                ),
            });
            return Err(Vec::new());
        }
    }
    let mut included: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut excluded: BTreeSet<String> = BTreeSet::new();
    for (marker, raw) in &entries {
        let (neg, pat) = match raw.strip_prefix('!') {
            Some(p) => (true, p),
            None => (false, raw.as_str()),
        };
        let file = if marker.starts_with("package.json") { "package.json" } else { "pnpm-workspace.yaml" };
        let segs = match norm_entry(pat) {
            Ok(s) => s,
            Err(why) => {
                ab.push(Abstention {
                    code: "H2-ENTRY",
                    location: file.into(),
                    reason: format!("entry {raw:?}: {why}; not expanded"),
                });
                continue;
            }
        };
        for d in expand(root, &segs, truncated) {
            if neg {
                excluded.insert(d);
            } else {
                included.entry(d).or_insert_with(|| (marker.clone(), raw.clone()));
            }
        }
    }
    let mut pkgs = Vec::new();
    for (dir, (marker, raw)) in included {
        if excluded.contains(&dir) || dir.is_empty() {
            continue; // the root never counts (H2 table)
        }
        let pjp = root.join(&dir).join("package.json");
        if !pjp.is_file() {
            continue;
        }
        if let Some(seg) = dir.split('/').find(|s| skip_dirs.contains(s)) {
            ab.push(Abstention {
                code: "H2-MEMBER",
                location: dir.clone(),
                reason: format!("declared member is inside a skipped directory ({seg}); its files are not scanned"),
            });
            continue;
        }
        let v: Value = match std::fs::read_to_string(&pjp).ok().and_then(|t| serde_json::from_str(&t).ok()) {
            Some(v) => v,
            None => {
                ab.push(Abstention {
                    code: "H2-MEMBER",
                    location: dir.clone(),
                    reason: "package.json is not valid JSON; not a package".into(),
                });
                continue;
            }
        };
        let name = v.get("name").and_then(Value::as_str).unwrap_or("").to_string();
        if name.is_empty() {
            ab.push(Abstention {
                code: "H2-MEMBER",
                location: dir.clone(),
                reason: "package.json has no \"name\"; not a package".into(),
            });
            continue;
        }
        pkgs.push(Package {
            dir,
            name,
            marker: format!("{marker} entry {raw:?}"),
            manifest: v,
        });
    }
    if pkgs.len() < 2 {
        let names: Vec<&str> = pkgs.iter().map(|p| p.dir.as_str()).collect();
        ab.push(repo(format!(
            "{} qualifying package(s){}; a multi-package workspace needs at least 2",
            pkgs.len(),
            if names.is_empty() { String::new() } else { format!(" ({})", names.join(", ")) }
        )));
        return Err(pkgs);
    }
    Ok(pkgs)
}

// ------------------------------------------------------------ resolution

enum Res {
    File(String),
    Unresolved(String),
    AmbigPkg(String),
    NotCandidate,
}

struct Ctx<'a> {
    pkgs: &'a [Package],
    files: &'a BTreeSet<String>,
    by_name: BTreeMap<&'a str, Vec<usize>>,
    /// INIT-DISCOVERY-SLICE3: each package's declared compiler back-maps,
    /// read once (empty for packages without "exports").
    backmaps: Vec<Vec<BackMap>>,
}

// ------------------------------------------------- package "exports" (SLICE3)
//
// Rulings S3-D1 to S3-D8. Resolution answers "which source file declares
// this export", following only what the package itself declares: its
// exports map and its own tsconfig compiler mapping. Conditions are
// followed order-free (serde_json stores objects sorted, so a package's own
// key order is not available) and every branch that reaches a source must
// name the same file; anything else is UNRESOLVED with an exact reason.

/// One declared compiler mapping from emitted output back to sources,
/// repository-relative: outDir (for .js and .d.ts) or declarationDir (for
/// .d.ts only), each to rootDir. rootDir is never computed (S3-D5).
struct BackMap {
    declarations_only: bool,
    out: String,
    root: String,
}

const EXPORTS_CONDITIONS: [&str; 4] = ["default", "import", "require", "types"];

/// The package's own tsconfig.json and tsconfig.build.json, both read;
/// `extends` is not followed. An unreadable or unparsable file adds nothing.
fn read_backmaps(root: &Path, pkg_dir: &str) -> Vec<BackMap> {
    let mut maps = Vec::new();
    for name in ["tsconfig.json", "tsconfig.build.json"] {
        let Ok(text) = std::fs::read_to_string(root.join(pkg_dir).join(name)) else { continue };
        let Ok(cfg) = serde_json::from_str::<Value>(&strip_jsonc(&text)) else { continue };
        let co = &cfg["compilerOptions"];
        let Some(rd) = co["rootDir"].as_str() else { continue };
        for (key, declarations_only) in [("outDir", false), ("declarationDir", true)] {
            let Some(od) = co[key].as_str() else { continue };
            if let (Ok(o), Ok(r)) = (norm_path(&format!("{pkg_dir}/{od}")), norm_path(&format!("{pkg_dir}/{rd}"))) {
                maps.push(BackMap { declarations_only, out: o.join("/"), root: r.join("/") });
            }
        }
    }
    maps
}

/// tsconfig is JSONC: drop comments outside strings and trailing commas
/// before `}` or `]`, so serde_json can read it. No other change.
fn strip_jsonc(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let (mut i, mut in_str) = (0, false);
    while i < c.len() {
        let ch = c[i];
        if in_str {
            out.push(ch);
            if ch == '\\' && i + 1 < c.len() {
                out.push(c[i + 1]);
                i += 2;
                continue;
            }
            in_str = ch != '"';
            i += 1;
            continue;
        }
        match ch {
            '"' => {
                in_str = true;
                out.push(ch);
            }
            '/' if c.get(i + 1) == Some(&'/') => {
                while i < c.len() && c[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            '/' if c.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < c.len() && !(c[i] == '*' && c[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            '}' | ']' => {
                let kept = out.trim_end().len();
                if out[..kept].ends_with(',') {
                    out.truncate(kept - 1);
                }
                out.push(ch);
            }
            _ => out.push(ch),
        }
        i += 1;
    }
    out
}

enum Leaf {
    Target(String),
    Fail(String),
}

/// The leaves of the map entry for `sub` (`.` or `./...`).
fn exports_match(exp: &Value, sub: &str) -> Vec<Leaf> {
    let is_sub = |k: &String| k.starts_with('.');
    let map = match exp {
        Value::Object(m) if !m.is_empty() && m.keys().all(is_sub) => m,
        Value::Object(m) if m.keys().any(is_sub) => {
            return vec![Leaf::Fail("the exports map mixes subpath keys and condition keys".into())]
        }
        Value::Object(_) | Value::String(_) | Value::Array(_) => {
            if sub != "." {
                return vec![Leaf::Fail(format!("no exports entry matches {sub:?}"))];
            }
            let mut acc = Vec::new();
            exports_leaves(exp, None, &mut acc);
            return acc;
        }
        _ => return vec![Leaf::Fail("the exports value is not a string, array or object".into())],
    };
    let mut acc = Vec::new();
    if let Some(t) = map.get(sub) {
        if t.is_null() {
            return vec![Leaf::Fail(format!("exports excludes {sub:?} (null)"))];
        }
        exports_leaves(t, None, &mut acc);
        return acc;
    }
    // One `*`: longest prefix wins, ties to the longer key (PATTERN_KEY_COMPARE).
    let mut best: Option<(&String, usize, usize)> = None;
    for k in map.keys() {
        let Some((pre, suf)) = k.split_once('*') else { continue };
        if suf.contains('*') || sub.len() < pre.len() + suf.len() || !sub.starts_with(pre) || !sub.ends_with(suf) {
            continue;
        }
        if best.is_none_or(|(bk, bp, _)| pre.len() > bp || (pre.len() == bp && k.len() > bk.len())) {
            best = Some((k, pre.len(), suf.len()));
        }
    }
    match best {
        Some((k, p, s)) => {
            if map[k].is_null() {
                return vec![Leaf::Fail(format!("exports excludes {sub:?} (null)"))];
            }
            exports_leaves(&map[k], Some(&sub[p..sub.len() - s]), &mut acc);
            acc
        }
        None => vec![Leaf::Fail(format!("no exports entry matches {sub:?}"))],
    }
}

/// Strings are leaves, arrays are fallbacks (first valid wins), objects are
/// conditions (every active branch, order-free), null excludes.
fn exports_leaves(t: &Value, star: Option<&str>, acc: &mut Vec<Leaf>) {
    match t {
        Value::Null => acc.push(Leaf::Fail("exports excludes this entry (null)".into())),
        Value::String(s) => acc.push(exports_target(s, star)),
        Value::Array(items) => {
            for x in items {
                let mut sub = Vec::new();
                exports_leaves(x, star, &mut sub);
                if sub.iter().any(|l| matches!(l, Leaf::Target(_))) {
                    acc.extend(sub);
                    return;
                }
            }
            acc.push(Leaf::Fail("no valid fallback in the exports array".into()));
        }
        Value::Object(m) => {
            let mut hit = false;
            for (k, v) in m {
                if EXPORTS_CONDITIONS.contains(&k.as_str()) {
                    hit = true;
                    exports_leaves(v, star, acc);
                }
            }
            if !hit {
                acc.push(Leaf::Fail("no exports condition among types, import, require, default".into()));
            }
        }
        _ => acc.push(Leaf::Fail("an exports target is not a string".into())),
    }
}

/// Every `*` takes the capture; a target must start with `./` and may not
/// hold a `..` or `node_modules` segment after substitution.
fn exports_target(t: &str, star: Option<&str>) -> Leaf {
    if !t.starts_with("./") {
        return Leaf::Fail(format!("exports target {t:?} does not start with \"./\""));
    }
    let v = match star {
        Some(cap) => t.replace('*', cap),
        None => t.to_string(),
    };
    if v.split('/').any(|seg| seg == ".." || seg == "node_modules") {
        return Leaf::Fail(format!("exports target {v:?} escapes the package"));
    }
    Leaf::Target(v)
}

impl<'a> Ctx<'a> {
    fn pkg_of(&self, path: &str) -> Option<usize> {
        self.pkgs
            .iter()
            .enumerate()
            .filter(|(_, p)| path.starts_with(&format!("{}/", p.dir)))
            .max_by_key(|(_, p)| p.dir.len())
            .map(|(i, _)| i)
    }
    fn probe(&self, base: &str) -> Option<String> {
        let cands: Vec<String> = if let Some(stem) = base.strip_suffix(".js") {
            vec![format!("{stem}.ts"), format!("{stem}.tsx")]
        } else if base.ends_with(".ts") || base.ends_with(".tsx") {
            vec![base.to_string()]
        } else {
            vec![
                format!("{base}.ts"),
                format!("{base}.tsx"),
                format!("{base}/index.ts"),
                format!("{base}/index.tsx"),
            ]
        };
        cands.into_iter().find(|c| self.files.contains(c))
    }
    /// SLICE3: resolve `sub` through package `pkg`'s exports map. A `.d.ts`
    /// leaf is never a source itself; any other leaf is probed as init
    /// always probes, then mapped back through the declared back-maps.
    /// Exactly one source across all leaves resolves; none or several is
    /// UNRESOLVED with an exact reason, never a pick (S3-D2).
    fn resolve_exports(&self, pkg: usize, exp: &Value, sub: &str) -> Res {
        let dir = &self.pkgs[pkg].dir;
        let mut sources = BTreeSet::new();
        let mut why = BTreeSet::new();
        for leaf in exports_match(exp, sub) {
            let t = match leaf {
                Leaf::Fail(r) => {
                    why.insert(r);
                    continue;
                }
                Leaf::Target(t) => t,
            };
            let Ok(segs) = norm_path(&format!("{dir}/{}", &t[2..])) else {
                why.insert(format!("exports target {t:?} leaves the repository"));
                continue;
            };
            let full = segs.join("/");
            let decl = full.ends_with(".d.ts");
            let mut found = BTreeSet::new();
            if !decl {
                found.extend(self.probe(&full));
            }
            if found.is_empty() {
                for m in &self.backmaps[pkg] {
                    if m.declarations_only && !decl {
                        continue;
                    }
                    let Some(rest) = full.strip_prefix(&format!("{}/", m.out)) else { continue };
                    let back = if m.root.is_empty() { rest.to_string() } else { format!("{}/{rest}", m.root) };
                    let back = match back.strip_suffix(".d.ts") {
                        Some(stem) => format!("{stem}.js"),
                        None => back,
                    };
                    found.extend(self.probe(&back));
                }
            }
            if found.is_empty() {
                why.insert(format!(
                    "exports target {t:?} is not a TypeScript source and no declared rootDir/outDir pair maps it back; not guessed"
                ));
            }
            sources.extend(found);
        }
        match sources.len() {
            1 => Res::File(sources.into_iter().next().unwrap_or_default()),
            0 => Res::Unresolved(why.into_iter().next().unwrap_or_else(|| "the exports map yields no target".into())),
            _ => Res::Unresolved(format!(
                "exports branches name different sources ({}); not guessed",
                sources.into_iter().collect::<Vec<_>>().join(", ")
            )),
        }
    }
    /// Resolve `spec` imported from `from`. With `cross`, only specifiers
    /// naming a different qualifying package are candidates.
    fn resolve(&self, spec: &str, from: &str, cross: bool) -> (Option<usize>, Res) {
        let from_pkg = self.pkg_of(from);
        if spec.starts_with("./") || spec.starts_with("../") || spec == "." || spec == ".." {
            let dir = from.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
            let joined = if dir.is_empty() { spec.to_string() } else { format!("{dir}/{spec}") };
            let Ok(segs) = norm_path(&joined) else { return (None, Res::NotCandidate) };
            let base = segs.join("/");
            let Some(pkg) = self.pkg_of(&format!("{base}/x")).or_else(|| self.pkg_of(&base)) else {
                return (None, Res::NotCandidate);
            };
            if cross && Some(pkg) == from_pkg {
                return (None, Res::NotCandidate);
            }
            return match self.probe(&base) {
                Some(f) => (Some(pkg), Res::File(f)),
                None => (Some(pkg), Res::Unresolved(format!("no TypeScript file for {spec:?}"))),
            };
        }
        let Some((name, idxs)) = self
            .by_name
            .iter()
            .filter(|(n, _)| spec == **n || spec.starts_with(&format!("{n}/")))
            .max_by_key(|(n, _)| n.len())
        else {
            return (None, Res::NotCandidate);
        };
        if idxs.len() > 1 {
            let dirs: Vec<&str> = idxs.iter().map(|i| self.pkgs[*i].dir.as_str()).collect();
            return (None, Res::AmbigPkg(format!("package name {name} is declared by {}", dirs.join(", "))));
        }
        let pkg = idxs[0];
        if cross && Some(pkg) == from_pkg {
            return (None, Res::NotCandidate);
        }
        let p = &self.pkgs[pkg];
        if let Some(exp) = p.manifest.get("exports") {
            let sub = format!(".{}", &spec[name.len()..]);
            return (Some(pkg), self.resolve_exports(pkg, exp, &sub));
        }
        let sub = spec[name.len()..].trim_start_matches('/');
        if sub.is_empty() {
            let fields: Vec<&str> = ["types", "typings", "main"]
                .iter()
                .filter_map(|k| p.manifest.get(*k).and_then(Value::as_str))
                .collect();
            let ts = fields
                .iter()
                .find(|f| (f.ends_with(".ts") || f.ends_with(".tsx")) && !f.ends_with(".d.ts"));
            let entry = match (ts, fields.is_empty()) {
                (Some(f), _) => {
                    let Ok(segs) = norm_path(&format!("{}/{}", p.dir, f)) else {
                        return (Some(pkg), Res::Unresolved(format!("entry {f:?} leaves the package")));
                    };
                    let e = segs.join("/");
                    if !self.files.contains(&e) {
                        return (Some(pkg), Res::Unresolved(format!("entry {f:?} does not exist")));
                    }
                    e
                }
                (None, true) => match self.probe(&format!("{}/index", p.dir)) {
                    Some(e) => e,
                    None => return (Some(pkg), Res::Unresolved("no index.ts or index.tsx".into())),
                },
                (None, false) => {
                    return (
                        Some(pkg),
                        Res::Unresolved(format!(
                            "entry {:?} is not a TypeScript source file; not guessed",
                            fields[0]
                        )),
                    )
                }
            };
            return (Some(pkg), Res::File(entry));
        }
        let Ok(segs) = norm_path(&format!("{}/{}", p.dir, sub)) else {
            return (Some(pkg), Res::Unresolved(format!("{spec:?} leaves the package")));
        };
        match self.probe(&segs.join("/")) {
            Some(f) => (Some(pkg), Res::File(f)),
            None => (Some(pkg), Res::Unresolved(format!("no TypeScript file for {spec:?}"))),
        }
    }
}

fn norm_path(p: &str) -> Result<Vec<String>, ()> {
    let mut segs: Vec<String> = Vec::new();
    for s in p.split('/') {
        match s {
            "" | "." => {}
            ".." => {
                segs.pop().ok_or(())?;
            }
            x => segs.push(x.to_string()),
        }
    }
    Ok(segs)
}

/// Barrel levels code-pair discovery follows from an import's resolved entry
/// (SLICE4, S4-D2: the smallest limit covering every chain measured).
const MAX_BARREL_DEPTH: usize = 5;

/// A barrel hop: the re-exporting file and the line of its re-export.
type Chain = Vec<(String, usize)>;

/// Chains compare by their files first, so moving a re-export line never
/// changes which route is named.
fn by_path(c: &Chain) -> (Vec<&str>, Vec<usize>) {
    c.iter().map(|(f, l)| (f.as_str(), *l)).unzip()
}

enum Found {
    /// `chain` lists the barrel hops from the entry; empty when the entry
    /// declares the shape itself.
    Decl { file: String, decl: Decl, chain: Chain },
    Ambig(Vec<String>),
    Barrel2,
    NotShape,
}

/// Follows the re-exports the files declare, breadth first from `file`, each
/// (file, name) once so a cycle ends (SLICE4). A file that declares the shape
/// is not followed further. Exactly one distinct declaring file within
/// MAX_BARREL_DEPTH levels resolves; several are ambiguous; one reached only
/// deeper is BARREL2. Each level is keyed by path and keeps the smallest
/// chain, so neither the answer nor the chain depends on re-export order.
fn find_shape(ctx: &Ctx, infos: &BTreeMap<String, FileInfo>, name: &str, file: &str) -> Found {
    let start = (file.to_string(), name.to_string());
    let mut seen = BTreeSet::from([start.clone()]);
    let mut level: BTreeMap<(String, String), Chain> = BTreeMap::from([(start, Vec::new())]);
    let mut within: BTreeMap<String, (Decl, Chain)> = BTreeMap::new();
    let mut beyond = false;
    let mut depth = 0;
    while !level.is_empty() {
        let mut next: BTreeMap<(String, String), Chain> = BTreeMap::new();
        for ((f, n), chain) in level {
            let Some(fi) = infos.get(&f) else { continue };
            if let Some(d) = fi.exported_shape(&n) {
                if depth <= MAX_BARREL_DEPTH {
                    within.entry(f).or_insert((d.clone(), chain));
                } else {
                    beyond = true;
                }
                continue;
            }
            for re in &fi.reexports {
                let orig = match &re.names {
                    None => n.clone(),
                    Some(ns) => match ns.iter().find(|(e, _)| *e == n) {
                        Some((_, o)) => o.clone(),
                        None => continue,
                    },
                };
                let (_, Res::File(t)) = ctx.resolve(&re.spec, &f, false) else { continue };
                let key = (t, orig);
                if seen.contains(&key) {
                    continue;
                }
                let mut c = chain.clone();
                c.push((f.clone(), re.line));
                let slot = next.entry(key).or_insert_with(|| c.clone());
                if by_path(&c) < by_path(slot) {
                    *slot = c;
                }
            }
        }
        seen.extend(next.keys().cloned());
        level = next;
        depth += 1;
    }
    match within.len() {
        0 if beyond => Found::Barrel2,
        0 => Found::NotShape,
        1 => {
            let (f, (d, chain)) = within.into_iter().next().expect("one entry");
            Found::Decl { file: f, decl: d, chain }
        }
        _ => Found::Ambig(within.into_keys().collect()),
    }
}

fn category(rel: &str) -> &'static str {
    let segs: Vec<&str> = rel.split('/').collect();
    let (dirs, file) = segs.split_at(segs.len().saturating_sub(1));
    let file = file.first().copied().unwrap_or("");
    let has = |set: &[&str]| dirs.iter().any(|d| set.contains(d));
    let test_file = [".test.ts", ".test.tsx", ".spec.ts", ".spec.tsx"].iter().any(|s| file.ends_with(s));
    if has(&["test", "tests", "__tests__"]) || test_file {
        "test"
    } else if has(&["example", "examples", "demo"]) {
        "example"
    } else if has(&["scripts", "tools"]) {
        "tooling"
    } else if rel.starts_with("src/") || rel.starts_with("lib/") {
        "production"
    } else {
        "unknown"
    }
}

/// Run slice 1 discovery over the walked files (`files`, relative paths,
/// each at most the walk's size cap) and the files the walk dropped as
/// oversized.
pub fn discover_code(root: &Path, files: &[String], oversized: &[String], skip_dirs: &[&str]) -> CodeDiscovery {
    let mut out = CodeDiscovery::default();
    let pkgs = match membership(root, skip_dirs, &mut out.abstentions, &mut out.truncated) {
        Ok(p) => p,
        Err(found) => {
            out.packages = found;
            out.repo_abstained = true;
            out.abstentions.sort();
            return out;
        }
    };
    let is_ts = |f: &str| f.ends_with(".ts") || f.ends_with(".tsx");
    let file_set: BTreeSet<String> = files.iter().filter(|f| is_ts(f)).cloned().collect();
    let mut by_name: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, p) in pkgs.iter().enumerate() {
        by_name.entry(p.name.as_str()).or_default().push(i);
    }
    let backmaps = pkgs
        .iter()
        .map(|p| if p.manifest.get("exports").is_some() { read_backmaps(root, &p.dir) } else { Vec::new() })
        .collect();
    let ctx = Ctx { pkgs: &pkgs, files: &file_set, by_name, backmaps };

    let mut infos: BTreeMap<String, FileInfo> = BTreeMap::new();
    for f in &file_set {
        if ctx.pkg_of(f).is_none() {
            continue;
        }
        match std::fs::read(root.join(f)).map(String::from_utf8) {
            Ok(Ok(text)) => {
                infos.insert(f.clone(), scan_file(&text));
            }
            _ => out.abstentions.push(Abstention {
                code: "UNREADABLE",
                location: f.clone(),
                reason: "not valid UTF-8 or not readable; not scanned".into(),
            }),
        }
    }
    for f in oversized.iter().filter(|f| is_ts(f)) {
        if ctx.pkg_of(f).is_some() {
            out.abstentions.push(Abstention {
                code: "UNREADABLE",
                location: f.clone(),
                reason: "larger than the 1000000-byte scan limit; not scanned".into(),
            });
        }
    }

    let mut pairs: BTreeMap<(String, String), CodePair> = BTreeMap::new();
    let stop: BTreeSet<String> = STOPLIST.iter().map(|s| name_key(s)).collect();
    // SLICE3B: each target package's declaration evidence, built on first need.
    // The second element is the package's first unreadable source file, if
    // any: evidence is incomplete, so silence cannot be earned there.
    let mut decl_index: BTreeMap<usize, (BTreeMap<String, DeclEvidence>, Option<String>)> = BTreeMap::new();
    // Exported object shapes per package: (file, exported name, decl).
    let mut pkg_shapes: BTreeMap<usize, Vec<(String, String, Decl)>> = BTreeMap::new();
    for (g, gi) in &infos {
        if let Some(p) = ctx.pkg_of(g) {
            for (ename, d) in gi.exported_shapes() {
                if matches!(d.kind, Kind::Object(_)) {
                    pkg_shapes.entry(p).or_default().push((g.clone(), ename, d.clone()));
                }
            }
        }
    }
    // SLICE7: per package, every function declaration (with its permitted
    // written return type, if any), and how many exported shapes carry each name.
    let mut pkg_funcs: BTreeMap<usize, BTreeMap<String, Vec<(String, usize, Option<(String, RetForm)>)>>> = BTreeMap::new();
    let mut pkg_shape_names: BTreeMap<usize, BTreeMap<String, usize>> = BTreeMap::new();
    for (g, gi) in &infos {
        if let Some(p) = ctx.pkg_of(g) {
            for (n, line, ret) in function_decls(gi) {
                pkg_funcs.entry(p).or_default().entry(n).or_default().push((g.clone(), line, ret));
            }
            for (ename, _) in gi.exported_shapes() {
                *pkg_shape_names.entry(p).or_default().entry(ename).or_insert(0) += 1;
            }
        }
    }
    // SLICE8: type lookups for property paths, shared by every consumer.
    let mut pidx = PropIndex { ctx: &ctx, infos: &infos, types: BTreeMap::new(), members: BTreeMap::new() };
    for (f, fi) in &infos {
        let Some(c) = ctx.pkg_of(f) else { continue };
        let mut imported_from: BTreeMap<usize, usize> = BTreeMap::new(); // pkg -> first import line
        let mut imported_shapes: BTreeSet<(String, String)> = BTreeSet::new();
        // Names imported by name from each package, resolved or not: R6 is
        // about shapes NOT imported by name.
        let mut imported_names: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
        // SLICE7: (local, original, import line, specifier) per package.
        let mut imported_locals: BTreeMap<usize, Vec<(String, String, usize, String)>> = BTreeMap::new();
        for imp in &fi.imports {
            let (pkg, res) = ctx.resolve(&imp.spec, f, true);
            if matches!(res, Res::NotCandidate) {
                continue;
            }
            if let Some(p) = pkg {
                let e = imported_from.entry(p).or_insert(imp.line);
                *e = (*e).min(imp.line);
                imported_names.entry(p).or_default().extend(imp.named.iter().cloned());
                for (local, orig) in &imp.locals {
                    imported_locals.entry(p).or_default().push((local.clone(), orig.clone(), imp.line, imp.spec.clone()));
                }
            }
            for name in &imp.named {
                out.candidates += 1;
                let loc = format!("{f}:{}", imp.line);
                let mut abstain = |code: &'static str, reason: String| {
                    out.abstentions.push(Abstention { code, location: loc.clone(), reason })
                };
                let target = match &res {
                    Res::AmbigPkg(why) => {
                        abstain("AMBIG-PACKAGE", format!("{name} from {:?}: {why}", imp.spec));
                        continue;
                    }
                    Res::Unresolved(why) => {
                        abstain("UNRESOLVED", format!("{name} from {:?}: {why}", imp.spec));
                        continue;
                    }
                    Res::File(t) => t.clone(),
                    Res::NotCandidate => continue,
                };
                match find_shape(&ctx, &infos, name, &target) {
                    // SLICE3B (S3B-D1): silence must be earned. Silent only
                    // when the target package declares the name as a value,
                    // class or enum and nowhere as a type; otherwise abstain
                    // with a reason that says where to look (S3B-D2).
                    Found::NotShape => {
                        let Some(tp) = ctx.pkg_of(&target) else { continue };
                        let (idx, unread) = decl_index.entry(tp).or_insert_with(|| {
                            let (idx, bad) = declaration_index(root, file_set.iter().filter(|g| ctx.pkg_of(g) == Some(tp)));
                            let over = oversized.iter().filter(|g| is_ts(g) && ctx.pkg_of(g) == Some(tp)).cloned();
                            (idx, over.chain(bad).min())
                        });
                        let pkg = &ctx.pkgs[tp].name;
                        match idx.get(name.as_str()) {
                            None => abstain(
                                "UNRESOLVED",
                                format!("{name} from {:?}: not declared in the sources of package {pkg}; not guessed", imp.spec),
                            ),
                            Some(ev) => match &ev.type_site {
                                None => {
                                    if let Some(u) = unread {
                                        abstain(
                                            "UNRESOLVED",
                                            format!("{name} from {:?}: package {pkg} has a source file init could not read ({u}); silence is not earned; not guessed", imp.spec),
                                        );
                                    }
                                }
                                Some(_) if ev.value && !imp.type_only.contains(name) => abstain(
                                    "UNRESOLVED",
                                    format!("{name} from {:?}: declared both as a type and as a value in package {pkg}; not guessed", imp.spec),
                                ),
                                Some((sf, sl)) => abstain(
                                    "UNRESOLVED",
                                    format!(
                                        "{name} from {:?}: declared as a type at {sf}:{sl}, which init could not reach from {target}; not guessed",
                                        imp.spec
                                    ),
                                ),
                            },
                        }
                    }
                    Found::Barrel2 => abstain(
                        "BARREL2",
                        format!("{name} from {:?} is re-exported through more than {MAX_BARREL_DEPTH} barrel levels (from {target})", imp.spec),
                    ),
                    Found::Ambig(files) => abstain(
                        "AMBIG-SHAPE",
                        format!("{name} from {:?} resolves to more than one declaration: {}", imp.spec, files.join(", ")),
                    ),
                    Found::Decl { file: d, decl, chain: barrel } => {
                        imported_shapes.insert((d.clone(), decl.name.clone()));
                        let Some(p) = ctx.pkg_of(&d) else { continue };
                        if p == c {
                            continue;
                        }
                        match &decl.kind {
                            Kind::Indexed => abstain(
                                "R5",
                                format!("{name} resolves to an indexed alias into generated types at {d}:{} (out of v1)", decl.line),
                            ),
                            Kind::Unsupported(what) => abstain(
                                "UNSUPPORTED-SHAPE",
                                format!("{name} at {d}:{} is not an object type ({what})", decl.line),
                            ),
                            Kind::Object(members) => {
                                // SLICE5: only reads through a receiver declared as the shape.
                                let (declared, _) = receiver_reads(fi, name, &BTreeMap::new());
                                let accessed = members
                                    .iter()
                                    .filter(|(m, _)| {
                                        let k = name_key(m);
                                        fi.reads.contains_key(&k) && !stop.contains(&k)
                                    })
                                    .count();
                                let (mut fields, mut stoplisted) = collect_fields(members, &declared, &stop);
                                // SLICE7: the imported-return-type route, only for a shape
                                // that does not qualify without it (existing evidence never moves).
                                let mut via = Vec::new();
                                if fields.len() < MIN_FIELDS {
                                    let avail = route_fns(p, name, &imported_locals, &pkg_funcs, &pkg_shape_names);
                                    if !avail.is_empty() {
                                        let routes_only: BTreeMap<String, Route> = avail.iter().map(|(k, (r, _))| (k.clone(), r.clone())).collect();
                                        let (r1, rr) = receiver_reads(fi, name, &routes_only);
                                        let (f1, s1) = collect_fields(members, &r1, &stop);
                                        if f1.len() >= MIN_FIELDS {
                                            via = used_routes(&f1, &rr, &avail);
                                            fields = f1;
                                            stoplisted = s1;
                                        }
                                    }
                                }
                                // SLICE8: property paths, only for a shape that still does not qualify.
                                let mut props = Vec::new();
                                if fields.len() < MIN_FIELDS {
                                    let paths = pidx.paths(f, &decl.name, &d);
                                    if !paths.is_empty() {
                                        let avail = route_fns(p, name, &imported_locals, &pkg_funcs, &pkg_shape_names);
                                        if let Some((f1, s1, v1, p1)) = with_paths(fi, name, members, &stop, &avail, &paths) {
                                            fields = f1;
                                            stoplisted = s1;
                                            via = v1;
                                            props = p1;
                                        }
                                    }
                                }
                                if fields.len() < MIN_FIELDS && accessed >= MIN_FIELDS {
                                    let why = format!(
                                        "{name}: {accessed} of its fields are accessed in this file, {} read on a receiver declared as {name}; the others are writes or are read off values init cannot tie to {name}; not guessed",
                                        fields.len()
                                    );
                                    abstain("RECV", why);
                                }
                                if fields.len() >= MIN_FIELDS {
                                    let ev = ShapeEvidence {
                                        shape: name.clone(),
                                        decl_line: decl.line,
                                        import_line: imp.line,
                                        specifier: imp.spec.clone(),
                                        resolved: target.clone(),
                                        barrel,
                                        fields,
                                        stoplisted,
                                        via,
                                        named: true,
                                        props,
                                    };
                                    let rel = f[pkgs[c].dir.len() + 1..].to_string();
                                    let pair = pairs
                                        .entry((d.clone(), f.clone()))
                                        .or_insert_with(|| CodePair {
                                            producer: d.clone(),
                                            consumer: f.clone(),
                                            producer_pkg: pkgs[p].clone(),
                                            consumer_pkg: pkgs[c].clone(),
                                            category: category(&rel),
                                            shapes: Vec::new(),
                                        })
                                        ;
                                    if !pair.shapes.iter().any(|s| s.shape == ev.shape) {
                                        pair.shapes.push(ev);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // R6: shapes of an imported-from package read but not imported by name.
        for (p, first_line) in &imported_from {
            if *p == c {
                continue;
            }
            for (g, ename, d) in pkg_shapes.get(p).into_iter().flatten() {
                {
                    if imported_shapes.contains(&(g.clone(), d.name.clone()))
                        || imported_names.get(p).is_some_and(|n| n.contains(ename))
                    {
                        continue;
                    }
                    let Kind::Object(members) = &d.kind else { continue };
                    let mut read: Vec<&str> = members
                        .iter()
                        .map(|(m, _)| m.as_str())
                        .filter(|m| {
                            let k = name_key(m);
                            fi.reads.contains_key(&k) && !stop.contains(&k)
                        })
                        .collect();
                    read.sort();
                    read.dedup();
                    // SLICE7: a shape not imported by name, read through receivers
                    // its package's functions are declared to return.
                    if read.len() >= MIN_FIELDS {
                        let avail = route_fns(*p, ename, &imported_locals, &pkg_funcs, &pkg_shape_names);
                        if !avail.is_empty() {
                            let routes_only: BTreeMap<String, Route> = avail.iter().map(|(k, (r, _))| (k.clone(), r.clone())).collect();
                            let (r1, rr) = receiver_reads(fi, ename, &routes_only);
                            let (fields, stoplisted) = collect_fields(members, &r1, &stop);
                            if fields.len() >= MIN_FIELDS {
                                let via = used_routes(&fields, &rr, &avail);
                                let ev = ShapeEvidence {
                                    shape: ename.clone(),
                                    decl_line: d.line,
                                    import_line: via[0].import_line,
                                    specifier: via[0].specifier.clone(),
                                    resolved: g.clone(),
                                    barrel: Vec::new(),
                                    fields,
                                    stoplisted,
                                    via,
                                    named: false,
                                    props: Vec::new(),
                                };
                                let rel = f[pkgs[c].dir.len() + 1..].to_string();
                                let pair = pairs.entry((g.clone(), f.clone())).or_insert_with(|| CodePair {
                                    producer: g.clone(),
                                    consumer: f.clone(),
                                    producer_pkg: pkgs[*p].clone(),
                                    consumer_pkg: pkgs[c].clone(),
                                    category: category(&rel),
                                    shapes: Vec::new(),
                                });
                                if !pair.shapes.iter().any(|s| s.shape == ev.shape) {
                                    pair.shapes.push(ev);
                                }
                                continue;
                            }
                        }
                    }
                    // SLICE8: a shape not imported by name, read through a property
                    // declared as it, at most two hops from a typed receiver.
                    if read.len() >= MIN_FIELDS {
                        let paths = pidx.paths(f, &d.name, g);
                        if !paths.is_empty() {
                            let avail = route_fns(*p, ename, &imported_locals, &pkg_funcs, &pkg_shape_names);
                            if let Some((fields, stoplisted, via, props)) = with_paths(fi, ename, members, &stop, &avail, &paths) {
                                let ev = ShapeEvidence {
                                    shape: ename.clone(),
                                    decl_line: d.line,
                                    import_line: 0,
                                    specifier: String::new(),
                                    resolved: g.clone(),
                                    barrel: Vec::new(),
                                    fields,
                                    stoplisted,
                                    via,
                                    named: false,
                                    props,
                                };
                                let rel = f[pkgs[c].dir.len() + 1..].to_string();
                                let pair = pairs.entry((g.clone(), f.clone())).or_insert_with(|| CodePair {
                                    producer: g.clone(),
                                    consumer: f.clone(),
                                    producer_pkg: pkgs[*p].clone(),
                                    consumer_pkg: pkgs[c].clone(),
                                    category: category(&rel),
                                    shapes: Vec::new(),
                                });
                                if !pair.shapes.iter().any(|s| s.shape == ev.shape) {
                                    pair.shapes.push(ev);
                                }
                                continue;
                            }
                        }
                    }
                    if read.len() >= MIN_FIELDS {
                        out.abstentions.push(Abstention {
                            code: "R6",
                            location: format!("{f}:{first_line}"),
                            reason: format!(
                                "reads {} of {ename} ({g}:{}) without importing it by name (out of v1)",
                                read.join(", "),
                                d.line
                            ),
                        });
                    }
                }
            }
        }
    }
    for p in pairs.values_mut() {
        p.shapes.sort_by(|a, b| a.shape.cmp(&b.shape));
    }
    out.pairs = pairs.into_values().collect();
    out.packages = pkgs;
    out.abstentions.sort();
    out.abstentions.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_members_only() {
        let fi = scan_file(
            "export interface Order {\n  total_minor: number;\n  metadata: {\n    owner_ref: string;\n  };\n  cb: (x: number) => void;\n  [k: string]: unknown;\n}\n",
        );
        let Kind::Object(ms) = &fi.decls[0].kind else { panic!() };
        let names: Vec<&str> = ms.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["total_minor", "metadata", "cb"]);
    }

    #[test]
    fn reads_exclude_declarations() {
        let fi = scan_file("const o = {\n  a_b: 1,\n};\nx.c_d; y?.e_f; z[\"g_h\"]; // q.r\n");
        let keys: Vec<&String> = fi.reads.keys().collect();
        assert_eq!(keys, vec!["cd", "ef", "gh"]);
    }

    #[test]
    fn category_rules() {
        assert_eq!(category("src/tests/x.ts"), "test");
        assert_eq!(category("examples/src/x.ts"), "example");
        assert_eq!(category("scripts/x.ts"), "tooling");
        assert_eq!(category("src/x.ts"), "production");
        assert_eq!(category("x.ts"), "unknown");
        assert_eq!(category("src/x.spec.ts"), "test");
    }
}
