//! INIT-DISCOVERY slice 1: TypeScript code producers paired with TypeScript
//! consumers across qualifying workspace packages
//! (docs/architecture/INIT-DISCOVERY-SLICE1.md).
//!
//! HONEST SCOPE: line and bracket-depth patterns over comment- and
//! string-masked text, not a parser. A proposal establishes a named import of
//! an exported shape across two declared packages plus at least two of the
//! shape's direct members read by property access in the importing file. It
//! does NOT establish that those reads are performed on values of the shape
//! (the receiver is not verified). Everything init cannot resolve is an
//! explicit abstention, never a guess.

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
    pub barrel: Option<(String, usize)>,
    pub fields: Vec<(String, usize)>,
    pub stoplisted: Vec<String>,
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
        if t.kw(j, "type") {
            let k = t.ws(j + 4);
            if t.m.get(k) == Some(&b'{') || t.ident(k).is_some() && !t.kw(k, "from") {
                j = k;
            }
        }
        let mut named = Vec::new();
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
            named = brace_names(t, j, close).into_iter().map(|(_, orig)| orig).collect();
            j = close;
        }
        let (spec, end) = from_spec(t, j)?;
        fi.imports.push(Import { line, spec, named });
        return Some(end);
    }
    if t.kw(i, "export") {
        let mut j = t.ws(i + 6);
        if t.m.get(j) == Some(&b'*') {
            let k = t.ws(j + 1);
            if t.kw(k, "as") {
                return None;
            }
            let (spec, end) = from_spec(t, k)?;
            fi.reexports.push(ReExport { line, spec, names: None });
            return Some(end);
        }
        if t.kw(j, "type") && t.m.get(t.ws(j + 4)) == Some(&b'{') {
            j = t.ws(j + 4);
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
        if p.manifest.get("exports").is_some() {
            return (
                Some(pkg),
                Res::Unresolved(format!("package {name} declares \"exports\"; not supported in slice 1")),
            );
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

enum Found {
    Decl { file: String, decl: Decl, barrel: Option<(String, usize)> },
    Ambig(Vec<String>),
    Barrel2,
    NotShape,
}

fn find_shape(ctx: &Ctx, infos: &BTreeMap<String, FileInfo>, name: &str, file: &str) -> Found {
    let Some(fi) = infos.get(file) else { return Found::NotShape };
    if let Some(d) = fi.exported_shape(name) {
        return Found::Decl { file: file.to_string(), decl: d.clone(), barrel: None };
    }
    let mut found: Vec<(String, Decl, usize)> = Vec::new();
    let mut deeper = false;
    for re in &fi.reexports {
        let orig = match &re.names {
            None => name.to_string(),
            Some(ns) => match ns.iter().find(|(e, _)| e == name) {
                Some((_, o)) => o.clone(),
                None => continue,
            },
        };
        let (_, Res::File(t2)) = ctx.resolve(&re.spec, file, false) else { continue };
        let Some(fi2) = infos.get(&t2) else { continue };
        if let Some(d) = fi2.exported_shape(&orig) {
            if !found.iter().any(|(f, _, _)| f == &t2) {
                found.push((t2, d.clone(), re.line));
            }
        } else if shape_one_level_down(ctx, infos, &orig, &t2) {
            deeper = true;
        }
    }
    match found.len() {
        1 => {
            let (f, d, line) = found.remove(0);
            Found::Decl { file: f, decl: d, barrel: Some((file.to_string(), line)) }
        }
        0 if deeper => Found::Barrel2,
        0 => Found::NotShape,
        _ => Found::Ambig(found.into_iter().map(|(f, _, _)| f).collect()),
    }
}

/// Whether `file` re-exports an exported shape `name` declared one level
/// further down (used only to classify BARREL2; never followed).
fn shape_one_level_down(ctx: &Ctx, infos: &BTreeMap<String, FileInfo>, name: &str, file: &str) -> bool {
    let Some(fi) = infos.get(file) else { return false };
    fi.reexports.iter().any(|re| {
        let orig = match &re.names {
            None => name.to_string(),
            Some(ns) => match ns.iter().find(|(e, _)| e == name) {
                Some((_, o)) => o.clone(),
                None => return false,
            },
        };
        match ctx.resolve(&re.spec, file, false) {
            (_, Res::File(t3)) => infos.get(&t3).map_or(false, |f3| f3.exported_shape(&orig).is_some()),
            _ => false,
        }
    })
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
    let ctx = Ctx { pkgs: &pkgs, files: &file_set, by_name };

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
    for (f, fi) in &infos {
        let Some(c) = ctx.pkg_of(f) else { continue };
        let mut imported_from: BTreeMap<usize, usize> = BTreeMap::new(); // pkg -> first import line
        let mut imported_shapes: BTreeSet<(String, String)> = BTreeSet::new();
        // Names imported by name from each package, resolved or not: R6 is
        // about shapes NOT imported by name.
        let mut imported_names: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
        for imp in &fi.imports {
            let (pkg, res) = ctx.resolve(&imp.spec, f, true);
            if matches!(res, Res::NotCandidate) {
                continue;
            }
            if let Some(p) = pkg {
                let e = imported_from.entry(p).or_insert(imp.line);
                *e = (*e).min(imp.line);
                imported_names.entry(p).or_default().extend(imp.named.iter().cloned());
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
                    Found::NotShape => {}
                    Found::Barrel2 => abstain(
                        "BARREL2",
                        format!("{name} from {:?} is re-exported through more than one barrel level (from {target})", imp.spec),
                    ),
                    Found::Ambig(files) => abstain(
                        "AMBIG-SHAPE",
                        format!("{name} from {:?} resolves to more than one declaration: {}", imp.spec, files.join(", ")),
                    ),
                    Found::Decl { file: d, decl, barrel } => {
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
                                let mut fields = Vec::new();
                                let mut stoplisted = Vec::new();
                                for (m, _) in members {
                                    let k = name_key(m);
                                    if let Some((_, line)) = fi.reads.get(&k) {
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
