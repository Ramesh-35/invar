//! `invar fix --suggest`: propose a consumer patch for a renamed field.
//!
//! A suggestion is made only when it is unambiguous: in the type that held the
//! locked field, exactly one field was removed (the locked one) and exactly
//! one was added, by every producer source that binds it. Then the consumer's
//! reads of the old name are shown rewritten to the new one, as a unified
//! diff. Nothing is ever written; type changes and anything ambiguous get a
//! reason instead of a patch.

use std::collections::BTreeSet;

use crate::lock::{qualified, LockedContract};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// `Type.field` as reported by `invar check`.
    pub residue: String,
    /// The new field name, when the rename is unambiguous.
    pub rename_to: Option<String>,
    /// Unified diff (zero context) of the consumer, when there is one.
    pub patch: Option<String>,
    /// Why there is no patch, or what the patch leaves out.
    pub note: Option<String>,
}

fn members<'a>(
    m: &'a [(String, String, Vec<String>)],
    src: &str,
    scope: &str,
) -> Option<&'a Vec<String>> {
    m.iter()
        .find(|(s, sc, _)| s == src && sc == scope)
        .map(|(_, _, f)| f)
}

/// Rewrite whole-identifier occurrences of `old` to `new` on one line.
fn rewrite(line: &str, old: &str, new: &str) -> String {
    let b = line.as_bytes();
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < line.len() {
        if line[i..].starts_with(old)
            && (i == 0 || !ident(b[i - 1]))
            && b.get(i + old.len()).is_none_or(|&c| !ident(c))
        {
            out.push_str(new);
            i += old.len();
        } else {
            let ch = line[i..].chars().next().unwrap_or(' ');
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Suggestions for the missing fields of one locked contract.
pub fn suggest(
    entry: &LockedContract,
    live_members: &[(String, String, Vec<String>)],
    residue: &[(String, bool)],
    consumer_name: &str,
    consumer_text: &str,
) -> Vec<Suggestion> {
    let mut out = Vec::new();
    for (shown, missing) in residue {
        let mut s = Suggestion {
            residue: shown.clone(),
            rename_to: None,
            patch: None,
            note: None,
        };
        if !missing {
            s.note = Some("type change, not a rename: no patch".into());
            out.push(s);
            continue;
        }
        let binds: Vec<_> = entry
            .bindings
            .iter()
            .filter(|b| &b.display() == shown)
            .collect();
        let mut targets = BTreeSet::new();
        let mut why = None;
        for b in &binds {
            let (Some(was), Some(now)) = (
                members(&entry.members, &b.source, &b.scope),
                members(live_members, &b.source, &b.scope),
            ) else {
                why = Some(format!(
                    "{} no longer exists in the {}",
                    qualified(&b.scope, ""),
                    b.source
                ));
                continue;
            };
            let removed: Vec<_> = was.iter().filter(|f| !now.contains(f)).collect();
            let added: Vec<_> = now.iter().filter(|f| !was.contains(f)).collect();
            if removed.len() == 1 && added.len() == 1 && removed[0] == &b.field {
                targets.insert(added[0].clone());
            } else {
                why = Some(format!(
                    "{} {}: {} removed, {} added; not a 1:1 rename",
                    b.source,
                    if b.scope.is_empty() {
                        "(unscoped)"
                    } else {
                        &b.scope
                    },
                    removed.len(),
                    added.len()
                ));
            }
        }
        let field = binds.first().map(|b| b.field.clone()).unwrap_or_default();
        if why.is_some() || targets.len() != 1 {
            s.note = Some(why.unwrap_or_else(|| "sources disagree on the new name".into()));
            out.push(s);
            continue;
        }
        let to = targets.into_iter().next().unwrap_or_default();
        s.rename_to = Some(to.clone());
        let mut patch = String::new();
        for (i, line) in consumer_text.lines().enumerate() {
            let new = rewrite(line, &field, &to);
            if new != line {
                patch.push_str(&format!("@@ -{n} +{n} @@\n-{line}\n+{new}\n", n = i + 1));
            }
        }
        if patch.is_empty() {
            s.note = Some(format!(
                "the consumer reads `{field}` in another naming form; rewrite it by hand to `{to}`"
            ));
        } else {
            s.patch = Some(format!(
                "--- a/{consumer_name}\n+++ b/{consumer_name}\n{patch}"
            ));
        }
        out.push(s);
    }
    out
}
