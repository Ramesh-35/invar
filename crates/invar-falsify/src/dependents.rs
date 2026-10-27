//! A4: field-level dependents graph over local invar.lock files
//! (spec: docs/architecture/A4-DEPENDENTS.md, Option A, ratified 2026-10-10).
//!
//! Zero network by construction: this module has no resolver, no transport
//! and no token. It reads lockfiles under a directory the caller aggregated
//! and answers "who depends on this field". Zero matches is an answer, not
//! an error, and the command holds no verdict.

// SECURITY-SPEC S2 (SEC-D4) sink: every terminal print in this file passes
// through `crate::term::neutralize`. These four definitions are identical in main.rs,
// gate.rs and dependents.rs.
#[allow(unused_macros)]
macro_rules! println {
    () => { std::println!() };
    ($($t:tt)*) => { std::println!("{}", crate::term::neutralize(&format!($($t)*))) };
}
#[allow(unused_macros)]
macro_rules! print {
    ($($t:tt)*) => { std::print!("{}", crate::term::neutralize(&format!($($t)*))) };
}
#[allow(unused_macros)]
macro_rules! eprintln {
    () => { std::eprintln!() };
    ($($t:tt)*) => { std::eprintln!("{}", crate::term::neutralize(&format!($($t)*))) };
}
#[allow(unused_macros)]
macro_rules! eprint {
    ($($t:tt)*) => { std::eprint!("{}", crate::term::neutralize(&format!($($t)*))) };
}

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde_json::json;

use crate::lock::{self, Lock};
use crate::surface::name_key;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependent {
    /// Lockfile path relative to the scanned directory.
    pub consumer: String,
    pub contract: String,
    /// Binding display (`Type.field` or `field`), as the lock shows it.
    pub binding: String,
    pub producer: String,
    pub ty: Option<String>,
    /// t_v: the day the consumer accepted this binding.
    pub since: String,
}

pub struct Answer {
    pub dependents: Vec<Dependent>,
    pub lockfiles_scanned: usize,
    pub lockfiles_matched: usize,
    /// Relative paths of lockfiles that did not parse (reported, never
    /// silently dropped).
    pub parse_errors: Vec<String>,
    /// For a scoped query only: bindings whose FIELD matches but that are
    /// bound without a scope (spec producers bind fields unscoped). Stated
    /// so a zero-match answer cannot mislead.
    pub unscoped_field_matches: usize,
}

/// SECURITY-SPEC S2-4: the walk bounds (measured: six real repositories
/// reach at most 29,919 entries and depth 12).
pub const MAX_WALK_DEPTH: usize = 64;
pub const MAX_WALK_ENTRIES: usize = 200_000;

/// Every `invar.lock` under `root`, never following a symbolic link (a
/// link to a directory or to a file is skipped), stopping with an error
/// past `max_depth` directory levels or past `max_entries` entries visited.
pub fn locks_under(root: &Path, max_depth: usize, max_entries: usize) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let mut seen = 0usize;
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > max_depth {
            return Err(format!("walk stopped: deeper than depth {max_depth} under {}", root.display()));
        }
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            seen += 1;
            if seen > max_entries {
                return Err(format!("walk stopped: more than {max_entries} entries under {}", root.display()));
            }
            let Ok(ft) = e.file_type() else { continue };
            let p = e.path();
            if ft.is_dir() {
                stack.push((p, depth + 1));
            } else if ft.is_file() && p.file_name().is_some_and(|n| n == "invar.lock") {
                out.push(p);
            }
        }
    }
    Ok(out)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    out.extend(locks_under(dir, MAX_WALK_DEPTH, MAX_WALK_ENTRIES).map_err(|e| anyhow::anyhow!(e))?);
    Ok(())
}

/// Does `query` (`Type.field` or bare `field`) match a binding with this
/// scope and field? Naming-form comparison, the engine's rule since alpha.4.
fn matches(query: &str, scope: &str, field: &str) -> bool {
    match query.rsplit_once('.') {
        Some((qscope, qfield)) => {
            name_key(qscope) == name_key(scope) && name_key(qfield) == name_key(field)
        }
        None => name_key(query) == name_key(field),
    }
}

pub fn find(root: &Path, query: &str, producer_filter: Option<&str>) -> Result<Answer> {
    if !root.is_dir() {
        bail!("--locks {}: not a directory", root.display());
    }
    let mut files = Vec::new();
    walk(root, &mut files)?;
    files.sort();
    if files.is_empty() {
        bail!("no invar.lock found under {}", root.display());
    }
    let rel = |p: &Path| {
        p.strip_prefix(root)
            .unwrap_or(p)
            .display()
            .to_string()
            .replace('\\', "/")
    };
    let mut dependents = Vec::new();
    let mut unscoped = 0usize;
    let mut parse_errors = Vec::new();
    let mut parsed = 0usize;
    let mut matched_files = 0usize;
    for f in &files {
        let text = match std::fs::read_to_string(f) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("skipping {}: {e}", rel(f));
                parse_errors.push(rel(f));
                continue;
            }
        };
        let lock: Lock = match lock::parse(&rel(f), &text) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("skipping {}: {e:#}", rel(f));
                parse_errors.push(rel(f));
                continue;
            }
        };
        parsed += 1;
        let before = dependents.len();
        for c in &lock.contracts {
            if let Some(p) = producer_filter {
                if !c.producer.contains(p) {
                    continue;
                }
            }
            for b in &c.bindings {
                if let Some((_, qfield)) = query.rsplit_once('.') {
                    if b.scope.is_empty() && name_key(qfield) == name_key(&b.field) {
                        unscoped += 1;
                    }
                }
                if matches(query, &b.scope, &b.field) {
                    dependents.push(Dependent {
                        consumer: rel(f),
                        contract: c.id.clone(),
                        binding: b.display(),
                        producer: c.producer.clone(),
                        ty: b.ty.clone(),
                        since: b.since.clone(),
                    });
                }
            }
        }
        if dependents.len() > before {
            matched_files += 1;
        }
    }
    if parsed == 0 {
        bail!(
            "none of the {} lockfile(s) under {} could be parsed",
            files.len(),
            root.display()
        );
    }
    dependents.sort();
    Ok(Answer {
        dependents,
        lockfiles_scanned: files.len(),
        lockfiles_matched: matched_files,
        parse_errors,
        unscoped_field_matches: unscoped,
    })
}

pub fn render_text(query: &str, a: &Answer) -> String {
    let mut out = String::new();
    for d in &a.dependents {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}{}\t(since {})\n",
            d.consumer,
            d.contract,
            d.binding,
            d.producer,
            d.ty.as_deref()
                .map(|t| format!(" [{t}]"))
                .unwrap_or_default(),
            d.since
        ));
    }
    out.push_str(&format!(
        "{} dependent binding(s) on {:?} in {} of {} lockfile(s)\n",
        a.dependents.len(),
        query,
        a.lockfiles_matched,
        a.lockfiles_scanned
    ));
    if a.dependents.is_empty() && a.unscoped_field_matches > 0 {
        if let Some((_, f)) = query.rsplit_once('.') {
            out.push_str(&format!(
                "note: {} unscoped binding(s) match the field; spec producers bind fields without a type scope. Query {:?} to see them.\n",
                a.unscoped_field_matches, f
            ));
        }
    }
    out
}

pub fn render_json(query: &str, producer_filter: Option<&str>, a: &Answer) -> String {
    let v = json!({
        "dependents": a.dependents.iter().map(|d| json!({
            "binding": d.binding,
            "consumer": d.consumer,
            "contract": d.contract,
            "producer": d.producer,
            "since": d.since,
            "type": d.ty,
        })).collect::<Vec<_>>(),
        "lockfiles_matched": a.lockfiles_matched,
        "lockfiles_scanned": a.lockfiles_scanned,
        "parse_errors": a.parse_errors,
        "producer_filter": producer_filter,
        "query": query,
        "unscoped_field_matches": a.unscoped_field_matches,
    });
    format!("{}\n", serde_json::to_string_pretty(&v).unwrap_or_default())
}

// ---------------------------------------------------------------------------
// A4.1 (ratified 2026-10-13): producer-side sibling awareness for `invar
// check`. Awareness, not modification detection: check has no baseline to
// know what changed, so this states which sibling lockfiles bind which of
// THIS repository's producer files. Read-only, zero network, never leaves
// `locks_dir`, and the caller must never let it change check's exit code.
// ---------------------------------------------------------------------------

/// One producer file of this repository, bound by one sibling lockfile.
pub struct SiblingRow {
    /// Producer path relative to this repository's base_dir, as resolved.
    pub producer: String,
    /// Sibling lockfile path relative to the scanned directory.
    pub lockfile: String,
    pub bindings: usize,
}

pub struct Siblings {
    pub rows: Vec<SiblingRow>,
    pub lockfiles_scanned: usize,
    /// Sibling lockfiles that did not parse (stated, never dropped).
    pub parse_errors: Vec<String>,
    /// Local producer refs in sibling locks that could not be resolved on
    /// disk (moved or deleted files); counted, never guessed at.
    pub unresolved_producers: usize,
    pub own_lock_excluded: bool,
}

/// Which sibling lockfiles under `locks_dir` bind producer files inside
/// `base_dir`? The repository's own invar.lock is excluded.
pub fn siblings(base_dir: &Path, locks_dir: &Path) -> Result<Siblings> {
    let base = std::fs::canonicalize(base_dir)
        .map_err(|e| anyhow::anyhow!("resolving {}: {e}", base_dir.display()))?;
    let own_lock = std::fs::canonicalize(base_dir.join("invar.lock")).ok();
    let mut files = Vec::new();
    walk(locks_dir, &mut files)?;
    files.sort();
    let mut rows: std::collections::BTreeMap<(String, String), usize> = Default::default();
    let mut parse_errors = Vec::new();
    let mut unresolved = 0usize;
    let mut scanned = 0usize;
    let mut own_excluded = false;
    for lf in &files {
        if own_lock.is_some() && std::fs::canonicalize(lf).ok() == own_lock {
            own_excluded = true;
            continue;
        }
        scanned += 1;
        let rel_lock = lf
            .strip_prefix(locks_dir)
            .unwrap_or(lf)
            .display()
            .to_string();
        let text = match std::fs::read_to_string(lf) {
            Ok(t) => t,
            Err(e) => {
                parse_errors.push(format!("{rel_lock}: {e}"));
                continue;
            }
        };
        let lock = match lock::parse(&lf.display().to_string(), &text) {
            Ok(l) => l,
            Err(e) => {
                parse_errors.push(format!("{rel_lock}: {e}"));
                continue;
            }
        };
        let lock_dir = lf.parent().unwrap_or(Path::new("."));
        for c in &lock.contracts {
            if crate::resolver::is_remote(&c.producer) {
                continue;
            }
            let resolved = match std::fs::canonicalize(lock_dir.join(&c.producer)) {
                Ok(p) => p,
                Err(_) => {
                    unresolved += 1;
                    continue;
                }
            };
            if let Ok(inside) = resolved.strip_prefix(&base) {
                *rows
                    .entry((inside.display().to_string(), rel_lock.clone()))
                    .or_insert(0) += c.bindings.len();
            }
        }
    }
    Ok(Siblings {
        rows: rows
            .into_iter()
            .map(|((producer, lockfile), bindings)| SiblingRow { producer, lockfile, bindings })
            .collect(),
        lockfiles_scanned: scanned,
        parse_errors,
        unresolved_producers: unresolved,
        own_lock_excluded: own_excluded,
    })
}

/// Text for the end of `invar check`. Informational, never a verdict.
pub fn render_siblings(locks_display: &str, s: &Siblings) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "sibling dependents ({locks_display}: {} lockfile(s) scanned{}):\n",
        s.lockfiles_scanned,
        if s.own_lock_excluded { ", own lock excluded" } else { "" }
    ));
    if s.rows.is_empty() {
        out.push_str("  none bind this repository's producer files\n");
    }
    for r in &s.rows {
        out.push_str(&format!(
            "  {} <- {} ({} binding(s))\n",
            r.producer, r.lockfile, r.bindings
        ));
    }
    for e in &s.parse_errors {
        out.push_str(&format!("  unreadable (stated, never dropped): {e}\n"));
    }
    if s.unresolved_producers > 0 {
        out.push_str(&format!(
            "  {} sibling producer ref(s) did not resolve on disk (stated)\n",
            s.unresolved_producers
        ));
    }
    if !s.rows.is_empty() {
        out.push_str(&format!(
            "  detail: invar dependents <Type.field> --locks {locks_display}\n"
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// SIB-1 (P5, docs/falsification/SIBLING-EVAL-SPEC.md r1): evaluation mode.
// Run in a producer repository: every signed consumer lock under `locks_dir`
// whose remote producer names `repo` is evaluated against the working tree.
// Advisory: broken fields never fail the run (0); anything not evaluated is
// a stated blind spot (4). Zero network: remote refs are matched by name.
// ---------------------------------------------------------------------------

pub const SIBLING_NOTE: &str =
    "note: a lock records the fields a consumer depends on, not the lines that read them.";

/// SB-3: the principal that signed `bytes`, when the signature names a
/// principal in `signers` and verifies over exactly these bytes.
fn signer(bytes: &[u8], sig: &Path, signers: &Path) -> Result<Option<String>> {
    let found = std::process::Command::new("ssh-keygen")
        .args(["-Y", "find-principals", "-s"])
        .arg(sig)
        .arg("-f")
        .arg(signers)
        .stderr(std::process::Stdio::null())
        .output()
        .map_err(|e| anyhow::anyhow!("running ssh-keygen: {e}"))?;
    let principal = String::from_utf8_lossy(&found.stdout).lines().next().unwrap_or("").trim().to_string();
    if !found.status.success() || principal.is_empty() {
        return Ok(None);
    }
    let ok = crate::receipt::verify_ssh(bytes, sig, signers, &principal, "invar-lock")?;
    Ok(ok.then_some(principal))
}

/// SB-1: is this producer ref a file of `repo` (case-insensitive)? Its path.
fn our_path(producer: &str, repo: &str) -> Option<String> {
    let r = crate::resolver::parse_ref(producer, None).ok()?;
    format!("{}/{}", r.owner, r.repo).eq_ignore_ascii_case(repo).then_some(r.path)
}

/// SB-2 for one contract: (broken lines, provided count), or why not.
fn evaluate_contract(root: &Path, c: &lock::LockedContract, path: &str) -> std::result::Result<(Vec<String>, usize), String> {
    let file = root.join(path);
    let text = std::fs::read_to_string(&file).map_err(|_| format!("{path} not found in this repository"))?;
    let fields = crate::backtest::surface(c, &text).ok_or_else(|| format!("{path}: fields cannot be extracted"))?;
    let scoped: std::collections::BTreeSet<String> =
        c.bindings.iter().filter(|b| !b.scope.is_empty()).map(|b| name_key(&b.field)).collect();
    let (mut broken, mut provided) = (Vec::new(), 0usize);
    for b in &c.bindings {
        if b.scope.is_empty() && scoped.contains(&name_key(&b.field)) {
            continue;
        }
        let locked = b.ty.as_deref().unwrap_or("?");
        if !crate::backtest::present(&fields, b) {
            broken.push(format!("{} absent (locked {})", b.display(), locked));
            continue;
        }
        let now = fields
            .iter()
            .find(|((s, n), _)| {
                name_key(n) == name_key(&b.field) && (b.scope.is_empty() || name_key(s) == name_key(&b.scope))
            })
            .and_then(|(_, t)| t.clone());
        match (&b.ty, now) {
            (Some(was), Some(now)) if *was != now => {
                broken.push(format!("{} retyped (locked {was}, now {now})", b.display()))
            }
            _ => provided += 1,
        }
    }
    Ok((broken, provided))
}

/// The evaluation report and its exit code (0 answered, 4 blind spots).
pub fn evaluate(locks_dir: &Path, root: &Path, repo: &str, signers: Option<&Path>) -> Result<(String, i32)> {
    let esc = crate::term::escape_value;
    let mut files = Vec::new();
    walk(locks_dir, &mut files)?;
    files.sort();
    let (mut out, mut broken_n, mut provided_n, mut locks_ne, mut contracts_ne) = (String::new(), 0, 0, 0, 0);
    for lf in &files {
        let rel = esc(&lf.strip_prefix(locks_dir).unwrap_or(lf).display().to_string());
        // One read: the bytes verified are the bytes parsed.
        let bytes = match std::fs::read(lf) {
            Ok(b) => b,
            Err(e) => {
                out.push_str(&format!("{rel}: not evaluated: unreadable: {}\n", esc(&e.to_string())));
                locks_ne += 1;
                continue;
            }
        };
        let parsed = std::str::from_utf8(&bytes)
            .map_err(|e| e.to_string())
            .and_then(|t| lock::parse(&lf.display().to_string(), t).map_err(|e| e.to_string()));
        let l = match parsed {
            Ok(l) => l,
            Err(e) => {
                out.push_str(&format!("{rel}: not evaluated: unreadable: {}\n", esc(&e)));
                locks_ne += 1;
                continue;
            }
        };
        let ours: Vec<(&lock::LockedContract, String)> = l
            .contracts
            .iter()
            .filter_map(|c| our_path(&c.producer, repo).map(|p| (c, p)))
            .collect();
        if ours.is_empty() {
            continue;
        }
        let sig = lf.with_extension("lock.sig");
        let principal = match signers {
            None => Err("no signers configured (--lock-signers)"),
            Some(_) if !sig.exists() => Err("unsigned (no invar.lock.sig)"),
            Some(s) => signer(&bytes, &sig, s)?.ok_or("signature does not verify"),
        };
        let principal = match principal {
            Ok(p) => esc(&p),
            Err(why) => {
                out.push_str(&format!("{rel}: not evaluated: {why}\n"));
                locks_ne += 1;
                continue;
            }
        };
        for (c, path) in ours {
            let head = format!("{rel} [verified as {principal}] {}:", esc(&c.id));
            match evaluate_contract(root, c, &path) {
                Ok((broken, provided)) => {
                    out.push_str(&format!("{head}\n"));
                    for b in &broken {
                        out.push_str(&format!("    {}\n", esc(b)));
                    }
                    out.push_str(&format!("    {provided} binding(s) provided\n"));
                    broken_n += broken.len();
                    provided_n += provided;
                }
                Err(why) => {
                    out.push_str(&format!("{head} not evaluated: {}\n", esc(&why)));
                    contracts_ne += 1;
                }
            }
        }
    }
    let mut report = format!(
        "dependents of {} (locks: {}, {} lockfile(s) scanned)\n{out}summary: {broken_n} broken | {provided_n} provided | {locks_ne} lockfile(s) not evaluated",
        esc(repo),
        esc(&locks_dir.display().to_string()),
        files.len()
    );
    if contracts_ne > 0 {
        report.push_str(&format!(" | {contracts_ne} contract(s) not evaluated"));
    }
    report.push_str(&format!("\n{SIBLING_NOTE}\n"));
    Ok((report, if locks_ne + contracts_ne > 0 { 4 } else { 0 }))
}
