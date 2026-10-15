//! INIT-DISCOVERY slice 2b: string-coupling contracts (K3)
//! (docs/architecture/INIT-DISCOVERY-SLICE2B.md, FROZEN r1).
//!
//! HONEST SCOPE: pattern extraction over comment- and string-masked text, not
//! a parser (R1: no AST, no new dependencies). A string LIST is an array
//! literal of string literals in a query position (name or call marker from
//! the frozen set). A SHAPE is the depth-1 key set of an exported zod object
//! literal or of an `attributes: { ... }` literal whose members are all
//! objects carrying a `type` key. A pair is proposed only when ALL of
//! R-K3-1..R-K3-5 hold; anything else is an explicit abstention. A proposal
//! never claims the list reaches the store.
//!
//! This is a child module of `valuepairs` so it reuses the slice 2 text
//! machinery (`Text`, `scan_module`, `World`) without widening any
//! visibility in the landed file.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::*;
use crate::codepairs::{Abstention, STOPLIST};
use crate::surface::name_key;

/// R-K3-1: minimum distinctive shared names.
pub const MIN_OVERLAP: usize = 3;
/// Frozen query vocabulary (ruling D2): word segments of a bound name or key.
pub const QUERY_SEGMENTS: &[&str] = &["select", "fields", "columns", "populate", "attributes"];
/// Frozen call markers (ruling D2).
pub const CALL_MARKERS: &[&str] = &[".select(", ".populate("];
const MAX_LIST: usize = 500;

// ------------------------------------------------------------- names

fn segments(name: &str) -> Vec<String> {
    let chars: Vec<char> = name.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_ascii_alphanumeric() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            continue;
        }
        let lower_then_upper = c.is_ascii_uppercase() && i > 0 && chars[i - 1].is_ascii_lowercase();
        let acronym_end = c.is_ascii_uppercase()
            && i > 0
            && chars[i - 1].is_ascii_uppercase()
            && chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
        if (lower_then_upper || acronym_end) && !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(c.to_ascii_lowercase());
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn is_query_name(name: &str) -> bool {
    segments(name).iter().any(|s| QUERY_SEGMENTS.contains(&s.as_str()))
}

fn stop_set() -> BTreeSet<String> {
    STOPLIST.iter().map(|s| name_key(s)).collect()
}

// ------------------------------------------------------------- lists

#[derive(Debug, Clone)]
pub struct StrList {
    /// Stable locator: `const:NAME`, `prop:NAME#n`, `call:.select(#n`.
    pub at: String,
    pub line: usize,
    pub elems: Vec<(String, usize)>,
    /// Some(reason) when any element is not a plain string literal.
    pub dynamic: Option<String>,
    pub query: bool,
}

/// Parse the array literal opened at `open` (a `[`).
fn parse_array(t: &Text, open: usize) -> (Vec<(String, usize)>, Option<String>) {
    let Some(end) = t.close(open) else {
        return (Vec::new(), Some("unterminated array literal".into()));
    };
    let stop = end - 1;
    let mut elems = Vec::new();
    let mut dynamic: Option<String> = None;
    let mut j = open + 1;
    loop {
        j = t.ws(j);
        if j >= stop {
            break;
        }
        if t.m[j] == b',' {
            j += 1;
            continue;
        }
        if t.m[j..].starts_with(b"...") {
            dynamic.get_or_insert_with(|| "spread element".into());
        } else if let Some((s, e)) = t.string_at(j) {
            let q = t.m[j];
            let k = t.ws(e);
            if q == b'`' && s.contains("${") {
                dynamic.get_or_insert_with(|| "template literal with interpolation".into());
            } else if k >= stop || t.m[k] == b',' {
                elems.push((s, t.line(j)));
            } else {
                dynamic.get_or_insert_with(|| "concatenation or expression element".into());
            }
        } else {
            dynamic.get_or_insert_with(|| "non-literal element".into());
        }
        // Skip to the next top-level comma.
        let mut d = 0i64;
        let mut k = j;
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
    // A method chain on the literal (`['a'].map(...)`) makes it dynamic.
    let p = t.ws(end);
    if t.m.get(p) == Some(&b'.') {
        dynamic.get_or_insert_with(|| "method chain on list literal".into());
    }
    (elems, dynamic)
}

/// Every string-array literal bound to a name or placed at a query position.
pub fn scan_lists(text: &str) -> Vec<StrList> {
    let t = Text::new(text);
    scan_lists_t(&t)
}

fn scan_lists_t(t: &Text) -> Vec<StrList> {
    let n = t.m.len();
    let mut out: Vec<StrList> = Vec::new();
    let mut prop_n: BTreeMap<String, usize> = BTreeMap::new();
    let mut call_n: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut i = 0;
    while i < n {
        let prev_ident = i > 0 && is_ident(t.m[i - 1]);
        // `const|let|var NAME [: type] = [ ... ]`
        if !prev_ident && (t.kw(i, "const") || t.kw(i, "let") || t.kw(i, "var")) {
            let kwlen = if t.kw(i, "const") { 5 } else { 3 };
            if let Some((name, e)) = t.ident(t.ws(i + kwlen)) {
                let mut p = t.ws(e);
                if t.m.get(p) == Some(&b':') {
                    // skip a type annotation up to `=` at bracket depth 0
                    let mut d = 0i64;
                    let mut k = p + 1;
                    while k < n {
                        match t.m[k] {
                            b'<' | b'(' | b'[' | b'{' => d += 1,
                            b'>' if k > 0 && t.m[k - 1] == b'=' => {}
                            b'>' | b')' | b']' | b'}' => d -= 1,
                            b'=' if d <= 0 && t.m.get(k + 1) != Some(&b'>') => break,
                            b';' | b'\n' if d <= 0 => {
                                k = n;
                                break;
                            }
                            _ => {}
                        }
                        k += 1;
                    }
                    p = k;
                }
                if t.m.get(p) == Some(&b'=') && t.m.get(p + 1) != Some(&b'=') {
                    let q = t.ws(p + 1);
                    if t.m.get(q) == Some(&b'[') {
                        let (elems, dynamic) = parse_array(t, q);
                        out.push(StrList {
                            at: format!("const:{name}"),
                            line: t.line(q),
                            elems,
                            dynamic,
                            query: is_query_name(&name),
                        });
                    }
                }
            }
        }
        // `NAME: [ ... ]` with a query-segment NAME (object property)
        if !prev_ident && is_ident_start(t.m[i]) && (i == 0 || t.m[i - 1] != b'.') {
            if let Some((name, e)) = t.ident(i) {
                let p = t.ws(e);
                if t.m.get(p) == Some(&b':') && is_query_name(&name) {
                    let q = t.ws(p + 1);
                    if t.m.get(q) == Some(&b'[') {
                        let c = prop_n.entry(name.clone()).or_insert(0);
                        *c += 1;
                        let (elems, dynamic) = parse_array(t, q);
                        out.push(StrList { at: format!("prop:{name}#{c}"), line: t.line(q), elems, dynamic, query: true });
                    }
                }
            }
        }
        // `.select([ ... ])`, `.populate([ ... ])`
        if t.m[i] == b'.' {
            for mk in CALL_MARKERS {
                if t.m[i..].starts_with(mk.as_bytes()) {
                    let q = t.ws(i + mk.len());
                    if t.m.get(q) == Some(&b'[') {
                        let c = call_n.entry(mk).or_insert(0);
                        *c += 1;
                        let (elems, dynamic) = parse_array(t, q);
                        out.push(StrList { at: format!("call:{mk}#{c}"), line: t.line(q), elems, dynamic, query: true });
                    }
                }
            }
        }
        i += 1;
    }
    // A bound name declared more than once in a file cannot be located
    // unambiguously at check time: dynamic, never guessed.
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for l in &out {
        *seen.entry(l.at.clone()).or_default() += 1;
    }
    for l in &mut out {
        if seen[&l.at] > 1 && l.dynamic.is_none() {
            l.dynamic = Some("name declared more than once in the file".into());
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListRead {
    Found(Vec<(String, usize)>),
    Dynamic(String),
    NotFound,
}

/// The current literal elements of the list located by `at` (used by `invar
/// check`: the locked list is re-extracted with this same code).
pub fn read_list(text: &str, at: &str) -> ListRead {
    match scan_lists(text).into_iter().find(|l| l.at == at) {
        Some(l) => match l.dynamic {
            Some(why) => ListRead::Dynamic(why),
            None => ListRead::Found(l.elems),
        },
        None => ListRead::NotFound,
    }
}

// ------------------------------------------------------------- shapes

/// Members of an `attributes: { ... }` literal. `None`: not an attributes
/// literal (some member value is not an object carrying a `type` key).
fn attr_members(t: &Text, open: usize) -> Option<ShapeBody> {
    let Some(end) = t.close(open) else {
        return Some(ShapeBody::Dynamic("unterminated object literal".into()));
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
            return Some(ShapeBody::Dynamic("spread member".into()));
        }
        if c == b'[' {
            return Some(ShapeBody::Dynamic("computed key".into()));
        }
        let (key, e) = if matches!(c, b'"' | b'\'') { t.string_at(j)? } else { t.ident(j)? };
        let e2 = t.ws(e);
        if t.m.get(e2) != Some(&b':') {
            return None;
        }
        let v = t.ws(e2 + 1);
        if t.m.get(v) != Some(&b'{') {
            return None;
        }
        match zod_members(t, v) {
            ShapeBody::Fields(f) if f.iter().any(|(k, _)| k == "type") => {}
            _ => return None,
        }
        if !out.iter().any(|(k, _)| *k == key) {
            out.push((key, t.line(j)));
        }
        let vend = t.close(v)?;
        let mut k = t.ws(vend);
        // Skip any trailing expression to the next top-level comma.
        let mut d = 0i64;
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
    Some(ShapeBody::Fields(out))
}

/// `attributes: { ... }` literals, each named `<enclosing top-level
/// declaration>@attributes` (`default`, `module_exports`, or the const name; no dot, which the lock's Type.field grammar cannot carry).
fn scan_attr(t: &Text) -> Vec<ZodDecl> {
    let n = t.m.len();
    let mut depth = 0i64;
    let mut cur: Option<String> = None;
    let mut cands: Vec<(String, usize, usize)> = Vec::new(); // (decl, open, line)
    let mut i = 0;
    while i < n {
        let prev_ident = i > 0 && is_ident(t.m[i - 1]);
        match t.m[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            _ => {}
        }
        if !prev_ident {
            if depth == 0 {
                if t.kw(i, "const") || t.kw(i, "let") || t.kw(i, "var") {
                    let kwlen = if t.kw(i, "const") { 5 } else { 3 };
                    if let Some((name, _)) = t.ident(t.ws(i + kwlen)) {
                        cur = Some(name);
                    }
                } else if t.kw(i, "export") {
                    let j = t.ws(i + 6);
                    if t.kw(j, "default") {
                        cur = Some("default".into());
                    }
                } else if t.m[i..].starts_with(b"module.exports") {
                    cur = Some("module_exports".into());
                }
            }
            if t.kw(i, "attributes") && (i == 0 || t.m[i - 1] != b'.') {
                let p = t.ws(i + 10);
                if t.m.get(p) == Some(&b':') {
                    let q = t.ws(p + 1);
                    if t.m.get(q) == Some(&b'{') {
                        if let Some(decl) = &cur {
                            cands.push((decl.clone(), q, t.line(i)));
                        }
                    }
                }
            }
        }
        i += 1;
    }
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for (d, _, _) in &cands {
        *count.entry(d.clone()).or_default() += 1;
    }
    let mut out = Vec::new();
    for (decl, open, line) in cands {
        let name = format!("{decl}@attributes");
        if count[&decl] > 1 {
            out.push(ZodDecl { name, line, body: ShapeBody::Dynamic("more than one attributes literal under this declaration".into()) });
            continue;
        }
        if let Some(body) = attr_members(t, open) {
            out.push(ZodDecl { name, line, body });
        }
    }
    out
}

/// All shapes (zod consts and attributes literals) of one file.
pub fn scan_shapes(text: &str) -> Vec<ZodDecl> {
    let t = Text::new(text);
    let mut v = scan_zod(text);
    v.extend(scan_attr(&t));
    v
}

/// The current depth-1 key set of shape `name` (a zod const name, or
/// `<decl>@attributes`) in `text`.
pub fn read_any_shape(text: &str, name: &str) -> ShapeRead {
    match scan_shapes(text).into_iter().find(|d| d.name == name) {
        Some(ZodDecl { body: ShapeBody::Fields(f), .. }) => ShapeRead::Found(f.into_iter().map(|(k, _)| k).collect()),
        Some(ZodDecl { body: ShapeBody::Dynamic(why), .. }) => ShapeRead::Dynamic(why),
        None => ShapeRead::NotFound,
    }
}

// ---------------------------------------------------------- discovery

#[derive(Debug, Clone)]
pub struct StringsProposal {
    pub producer: String,
    pub shape: String,
    pub shape_line: usize,
    pub shape_keys: Vec<(String, usize)>,
    pub list_file: String,
    pub list_at: String,
    pub list_line: usize,
    pub list_len: usize,
    pub distinctive: usize,
    /// (shape spelling, list spelling, list line)
    pub overlap: Vec<(String, String, usize)>,
    pub scope: &'static str,
    pub category: &'static str,
}

#[derive(Debug, Default)]
pub struct StringsDiscovery {
    pub proposals: Vec<StringsProposal>,
    pub abstentions: Vec<Abstention>,
}

struct FileInfo {
    lists: Vec<StrList>,
}

struct ShapeInfo {
    file: String,
    name: String,
    line: usize,
    keys: Vec<(String, usize)>,
    family: Option<String>,
}

fn package_dir<'a>(file: &str, pkg_dirs: &'a BTreeSet<String>) -> Option<&'a String> {
    let mut dir = dir_of(file);
    loop {
        if let Some(d) = pkg_dirs.get(dir) {
            return Some(d);
        }
        if dir.is_empty() {
            return None;
        }
        dir = dir_of(dir);
    }
}

fn word_set(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut cur = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
            cur.push(c);
        } else if !cur.is_empty() {
            out.insert(name_key(&cur));
            cur.clear();
        }
    }
    if !cur.is_empty() {
        out.insert(name_key(&cur));
    }
    out
}

pub fn discover_strings(root: &Path, files: &[String], oversized: &[String]) -> StringsDiscovery {
    let mut out = StringsDiscovery::default();
    let stop = stop_set();
    let src: Vec<&String> = files
        .iter()
        .filter(|f| SRC_EXTS.contains(&ext_of(f)) && !oversized.contains(f))
        .collect();
    let mut w = World { files: src.iter().map(|f| (*f).clone()).collect(), pkgs: BTreeMap::new(), mods: BTreeMap::new() };
    let mut pkg_dirs: BTreeSet<String> = BTreeSet::new();
    for f in files.iter().filter(|f| f.rsplit('/').next() == Some("package.json")) {
        pkg_dirs.insert(dir_of(f).to_string());
        if let Ok(text) = std::fs::read_to_string(root.join(f)) {
            if let Some(name) = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v.get("name").and_then(|n| n.as_str().map(String::from)))
            {
                w.pkgs.entry(name).or_default().push(dir_of(f).to_string());
            }
        }
    }
    let mut shapes: Vec<ShapeInfo> = Vec::new();
    let mut infos: BTreeMap<String, FileInfo> = BTreeMap::new();
    for f in &src {
        let Ok(bytes) = std::fs::read(root.join(f)) else { continue };
        let Ok(text) = String::from_utf8(bytes) else { continue };
        let t = Text::new(&text);
        let md = scan_module(&text);
        for d in md.decls.iter().cloned().chain(scan_attr(&t)) {
            if let ShapeBody::Fields(keys) = d.body {
                let family = version_family(f, d.name.trim_end_matches("@attributes"));
                shapes.push(ShapeInfo { file: (*f).clone(), name: d.name, line: d.line, keys, family });
            }
        }
        let lists: Vec<StrList> = scan_lists_t(&t)
            .into_iter()
            .filter(|l| !l.elems.is_empty() && l.elems.len() <= MAX_LIST)
            .collect();
        w.mods.insert((*f).clone(), md);
        if !lists.is_empty() {
            infos.insert((*f).clone(), FileInfo { lists });
        }
    }
    // name_key -> shape indices
    let mut index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, s) in shapes.iter().enumerate() {
        let mut seen = BTreeSet::new();
        for (k, _) in &s.keys {
            let nk = name_key(k);
            if seen.insert(nk.clone()) {
                index.entry(nk).or_default().push(i);
            }
        }
    }
    // Files reachable from a file through one import (and one barrel level).
    let mut reach_cache: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let reach = |w: &World, file: &str, cache: &mut BTreeMap<String, BTreeSet<String>>| -> BTreeSet<String> {
        if let Some(r) = cache.get(file) {
            return r.clone();
        }
        let mut set = BTreeSet::new();
        if let Some(md) = w.mods.get(file) {
            for imp in &md.imports {
                if let Res::File(t) = w.resolve(&imp.spec, file) {
                    if let Some(tm) = w.mods.get(&t) {
                        for r in &tm.reexports {
                            if let Res::File(t2) = w.resolve(&r.spec, &t) {
                                set.insert(t2);
                            }
                        }
                    }
                    set.insert(t);
                }
            }
        }
        cache.insert(file.to_string(), set.clone());
        set
    };
    let scope_of = |w: &World, cache: &mut BTreeMap<String, BTreeSet<String>>, list_file: &str, shape_file: &str| -> Option<&'static str> {
        if list_file == shape_file {
            return Some("same-file");
        }
        if reach(w, list_file, cache).contains(shape_file) || reach(w, shape_file, cache).contains(list_file) {
            return Some("import");
        }
        if dir_of(list_file) == dir_of(shape_file) {
            return Some("same-directory");
        }
        match (package_dir(list_file, &pkg_dirs), package_dir(shape_file, &pkg_dirs)) {
            (Some(a), Some(b)) if a == b => Some("same-package"),
            _ => None,
        }
    };
    let mut orm_cache: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let orm_files: Vec<&String> = files.iter().filter(|f| matches!(ext_of(f), "prisma" | "sql")).collect();

    for (file, info) in &infos {
        for list in &info.lists {
            // distinct list names (name_key -> (spelling, first line))
            let mut raw: BTreeSet<String> = BTreeSet::new();
            let mut ld: BTreeMap<String, (String, usize)> = BTreeMap::new();
            for (s, line) in &list.elems {
                let nk = name_key(s);
                raw.insert(nk.clone());
                if !stop.contains(&nk) {
                    ld.entry(nk).or_insert((s.clone(), *line));
                }
            }
            if ld.len() < 2 && raw.len() < 3 {
                continue;
            }
            let loc = format!("{file}:{}", list.line);
            let label = list.at.clone();
            // per-shape overlap counts
            let mut counts: BTreeMap<usize, (usize, usize)> = BTreeMap::new();
            for nk in &raw {
                if let Some(v) = index.get(nk) {
                    for &si in v {
                        let c = counts.entry(si).or_default();
                        c.0 += 1;
                        if !stop.contains(nk) {
                            c.1 += 1;
                        }
                    }
                }
            }
            let qualifies = |c: &(usize, usize)| c.1 >= MIN_OVERLAP && c.1 * 2 >= ld.len();
            let mut inscope: Vec<(usize, &'static str, (usize, usize))> = Vec::new();
            let mut outscope: Vec<(usize, (usize, usize))> = Vec::new();
            for (&si, &c) in &counts {
                match scope_of(&w, &mut reach_cache, file, &shapes[si].file) {
                    Some(sc) => inscope.push((si, sc, c)),
                    None => outscope.push((si, c)),
                }
            }
            let q: Vec<&(usize, &'static str, (usize, usize))> = inscope.iter().filter(|(_, _, c)| qualifies(c)).collect();
            let names = |v: &[&(usize, &'static str, (usize, usize))]| -> String {
                v.iter().map(|(si, _, _)| format!("{} ({})", shapes[*si].name, shapes[*si].file)).collect::<Vec<_>>().join(", ")
            };
            if let Some(why) = &list.dynamic {
                if list.query && !q.is_empty() {
                    out.abstentions.push(Abstention {
                        code: "UNSUPPORTED-STRING-DYNAMIC",
                        location: loc,
                        reason: format!("list {label}: {why}; its literal part alone overlaps {}; the list is unrepresentable and never paired", names(&q)),
                    });
                }
                continue;
            }
            if !list.query {
                if !q.is_empty() {
                    out.abstentions.push(Abstention {
                        code: "AMBIG-RECEIVER",
                        location: loc,
                        reason: format!("list {label} is not in a query position (no select, fields, columns, populate or attributes name or call) but overlaps {}; not paired", names(&q)),
                    });
                }
                continue;
            }
            if q.is_empty() {
                // near miss among in-scope shapes
                let near = inscope.iter().filter(|(_, _, c)| c.0 >= 3 || c.1 >= 2).max_by_key(|(_, _, c)| (c.1, c.0));
                if let Some((si, _, c)) = near {
                    out.abstentions.push(Abstention {
                        code: "NON-DISTINCTIVE-KEY",
                        location: loc,
                        reason: format!(
                            "list {label} overlaps {} ({}) on {} names of which {} are distinctive after the stoplist; needs at least {MIN_OVERLAP} distinctive and at least half of the list's {} distinctive entries",
                            shapes[*si].name,
                            shapes[*si].file,
                            c.0,
                            c.1,
                            ld.len()
                        ),
                    });
                    continue;
                }
                let oq: Vec<&(usize, (usize, usize))> = outscope.iter().filter(|(_, c)| qualifies(c)).collect();
                if !oq.is_empty() {
                    let n = oq.iter().map(|(si, _)| format!("{} ({})", shapes[*si].name, shapes[*si].file)).collect::<Vec<_>>().join(", ");
                    out.abstentions.push(Abstention {
                        code: "AMBIG-RECEIVER",
                        location: loc,
                        reason: format!("list {label} overlaps {n} but no import path, directory or package links them; not paired"),
                    });
                    continue;
                }
                // overlap with a textual .prisma / .sql declaration only
                let pk = package_dir(file, &pkg_dirs).cloned();
                let mut hit: Option<String> = None;
                for of in &orm_files {
                    if package_dir(of, &pkg_dirs).cloned() != pk {
                        continue;
                    }
                    let words = orm_cache.entry((*of).clone()).or_insert_with(|| {
                        std::fs::read_to_string(root.join(of)).map(|t| word_set(&t)).unwrap_or_default()
                    });
                    let n = ld.keys().filter(|k| words.contains(*k)).count();
                    if n >= MIN_OVERLAP && n * 2 >= ld.len() {
                        hit = Some((*of).clone());
                        break;
                    }
                }
                if let Some(of) = hit {
                    out.abstentions.push(Abstention {
                        code: "UNSUPPORTED-ORM-SOURCE",
                        location: loc,
                        reason: format!("list {label} names fields declared in {of}, an ORM or DDL source that slice 2b does not parse; not paired"),
                    });
                }
                continue;
            }
            if q.iter().any(|(si, _, _)| shapes[*si].family.is_some()) {
                let fam = q.iter().find_map(|(si, _, _)| shapes[*si].family.clone()).unwrap_or_default();
                out.abstentions.push(Abstention {
                    code: "VERSION-FAMILY",
                    location: loc,
                    reason: format!("list {label} overlaps a shape in a version family ({fam}): {}; pairing across versions is refused", names(&q)),
                });
                continue;
            }
            if q.len() > 1 {
                out.abstentions.push(Abstention {
                    code: "AMBIG-RECEIVER",
                    location: loc,
                    reason: format!("list {label} qualifies against more than one shape: {}; the receiver cannot be told", names(&q)),
                });
                continue;
            }
            let (si, scope, _) = *q[0];
            let s = &shapes[si];
            let shape_keys: BTreeMap<String, &String> = s.keys.iter().map(|(k, _)| (name_key(k), k)).collect();
            let mut overlap: Vec<(String, String, usize)> = ld
                .iter()
                .filter_map(|(nk, (sp, line))| shape_keys.get(nk).map(|k| ((*k).clone(), sp.clone(), *line)))
                .collect();
            overlap.sort();
            out.proposals.push(StringsProposal {
                producer: s.file.clone(),
                shape: s.name.clone(),
                shape_line: s.line,
                shape_keys: s.keys.clone(),
                list_file: file.clone(),
                list_at: list.at.clone(),
                list_line: list.line,
                list_len: list.elems.len(),
                distinctive: ld.len(),
                overlap,
                scope,
                category: category(file),
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

    #[test]
    fn query_names() {
        for n in ["SELECT_FIELDS", "userColumns", "fields", "populateList", "ApiTokenAttributes"] {
            assert!(is_query_name(n), "{n}");
        }
        for n in ["USER_KEYS", "pick", "projection", "items", "Selection"] {
            assert!(!is_query_name(n), "{n}");
        }
    }

    #[test]
    fn lists_bound_prop_and_call() {
        let src = "export const SELECT_FIELDS = ['id', \"name\", `lastUsed`] as const;\nconst keys = ['a','b'];\nfoo({ fields: ['x','y'] });\nq.select(['p','q']);\nq.select(['r']);\n";
        let l = scan_lists(src);
        let ats: Vec<&str> = l.iter().map(|x| x.at.as_str()).collect();
        assert_eq!(ats, vec!["const:SELECT_FIELDS", "const:keys", "prop:fields#1", "call:.select(#1", "call:.select(#2"]);
        assert!(l[0].query && !l[1].query);
        assert_eq!(l[0].elems.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>(), vec!["id", "name", "lastUsed"]);
        assert!(l.iter().all(|x| x.dynamic.is_none()));
    }

    #[test]
    fn dynamic_lists() {
        for (src, why) in [
            ("const FIELDS = ['a', ...BASE];", "spread"),
            ("const FIELDS = ['last' + 'Used', 'b'];", "concatenation"),
            ("const FIELDS = [`${p}Used`, 'b'];", "template"),
            ("const FIELDS = ['a', x];", "non-literal"),
            ("const FIELDS = ['a','b'].map(f => f);", "method chain"),
        ] {
            let l = scan_lists(src);
            assert_eq!(l.len(), 1, "{src}");
            assert!(l[0].dynamic.as_deref().unwrap_or("").contains(why), "{src}: {:?}", l[0].dynamic);
        }
    }

    #[test]
    fn typed_const_and_duplicate() {
        let l = scan_lists("const FIELDS: string[] = ['a','b'];");
        assert_eq!(l.len(), 1);
        assert!(l[0].dynamic.is_none());
        let l = scan_lists("function f(){ const FIELDS = ['a']; }\nfunction g(){ const FIELDS = ['b']; }");
        assert!(l.iter().all(|x| x.dynamic.as_deref() == Some("name declared more than once in the file")));
    }

    #[test]
    fn attribute_shapes() {
        let src = "module.exports = {\n  attributes: {\n    name: { type: 'string' },\n    lastUsed: { type: 'datetime', x: 1 },\n  },\n};\n";
        let s = scan_shapes(src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].name, "module_exports@attributes");
        match &s[0].body {
            ShapeBody::Fields(f) => assert_eq!(f.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(), vec!["name", "lastUsed"]),
            b => panic!("{b:?}"),
        }
        // not an attributes literal: members are not objects with a type key
        assert!(scan_shapes("const x = { attributes: { id: 'a', class: 'b' } };").is_empty());
        // spread member
        let s = scan_shapes("export default { attributes: { ...base, a: { type: 'x' } } };");
        assert!(matches!(&s[0].body, ShapeBody::Dynamic(w) if w.contains("spread")));
    }

    #[test]
    fn read_roundtrip() {
        let t = "export const Tok = z.object({ id: z.string(), lastUsed: z.string() });";
        assert_eq!(read_any_shape(t, "Tok"), ShapeRead::Found(vec!["id".into(), "lastUsed".into()]));
        assert_eq!(read_any_shape(t, "Nope"), ShapeRead::NotFound);
        assert_eq!(read_list("const F = ['a','b'];", "const:F"), ListRead::Found(vec![("a".into(), 1), ("b".into(), 1)]));
        assert_eq!(read_list("const F = ['a','b'];", "const:G"), ListRead::NotFound);
    }
}
