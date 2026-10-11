//! A4: field-level dependents graph over local invar.lock files
//! (spec: docs/architecture/A4-DEPENDENTS.md, Option A, ratified 2026-10-10).
//!
//! Zero network by construction: this module has no resolver, no transport
//! and no token. It reads lockfiles under a directory the caller aggregated
//! and answers "who depends on this field". Zero matches is an answer, not
//! an error, and the command holds no verdict.

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

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.file_name().is_some_and(|n| n == "invar.lock") {
            out.push(p);
        }
    }
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
    walk(root, &mut files);
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
