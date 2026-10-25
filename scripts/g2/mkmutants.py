#!/usr/bin/env python3
"""G2 mutation operators G2-M1 to G2-M11 (docs/falsification/G2-SPEC.md
section G), each one textual edit of the implemented tree, written as a
unified diff with the operator's DECLARED killer fixture on its first line.
The patch is computed with difflib against the current files, so it applies
to a tree holding exactly this implementation.
Usage: mkmutants.py <tree> <out dir>"""
import difflib, os, sys

tree, out = sys.argv[1], sys.argv[2]
SU = "crates/invar-falsify/src/surface.rs"
CH = "crates/invar-falsify/src/check.rs"
RC = 'pub fn rename_candidates(base: &Fields, head: &Fields) -> BTreeMap<String, (String, String)> {\n    let mut out: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();\n    for ((scope, key), ty) in base {\n        let Some(ty) = ty else { continue };\n        if head.contains_key(&(scope.clone(), key.clone())) {\n            continue;\n        }\n        let added: Vec<&String> = head\n            .iter()\n            .filter(|((s, k), t)| s == scope && t.as_ref() == Some(ty) && !base.contains_key(&(s.clone(), k.clone())))\n            .map(|((_, k), _)| k)\n            .collect();\n        let rivals = base\n            .iter()\n            .filter(|((s, k), t)| s == scope && t.as_ref() == Some(ty) && !head.contains_key(&(s.clone(), k.clone())))\n            .count();\n        if added.len() == 1 && rivals == 1 {\n            out.entry(key.clone()).or_insert_with(Vec::new).push((added[0].clone(), ty.clone()));\n        }\n    }\n    // A key removed from two scopes with candidates is ambiguous: no pairing.\n    out.into_iter().filter(|(_, v)| v.len() == 1).map(|(k, mut v)| (k, v.remove(0))).collect()\n}\n'
M10 = 'pub fn rename_candidates(base: &Fields, head: &Fields) -> BTreeMap<String, (String, String)> {\n    let mut out: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();\n    for ((scope, key), ty) in base {\n        if head.contains_key(&(scope.clone(), key.clone())) {\n            continue;\n        }\n        let added: Vec<&String> = head\n            .iter()\n            .filter(|((s, k), t)| s == scope && *t == ty && !base.contains_key(&(s.clone(), k.clone())))\n            .map(|((_, k), _)| k)\n            .collect();\n        let rivals = base\n            .iter()\n            .filter(|((s, k), t)| s == scope && *t == ty && !head.contains_key(&(s.clone(), k.clone())))\n            .count();\n        if added.len() == 1 && rivals == 1 {\n            out.entry(key.clone()).or_insert_with(Vec::new).push((added[0].clone(), ty.clone().unwrap_or_default()));\n        }\n    }\n    // A key removed from two scopes with candidates is ambiguous: no pairing.\n    out.into_iter().filter(|(_, v)| v.len() == 1).map(|(k, mut v)| (k, v.remove(0))).collect()\n}\n'
ADDED = ".filter(|((s, k), t)| s == scope && t.as_ref() == Some(ty) && !base.contains_key(&(s.clone(), k.clone())))"

M = [
    ("G2-M1", "g2_f01", "G2-A disabled (Go name never added)", CH,
     'let go_name = go_names.get(name).filter(|_| c.ends_with(".go") && tc.is_none());',
     "let go_name: Option<&String> = None;"),
    ("G2-M2", "g2_f02", "Go-consumer requirement removed", CH,
     '.filter(|_| c.ends_with(".go") && tc.is_none());',
     ".filter(|_| tc.is_none());"),
    ("G2-M3", "g2_f03", "exported-type requirement removed", SU,
     "let exported = scope.starts_with(|c: char| c.is_ascii_uppercase());",
     "let exported = true;"),
    ("G2-M4", "g2_f04", "Go-name survival check removed", SU,
     "let gone = !head_d.iter().any(|(_, _, n, _)| *n == name);",
     "let gone = true;"),
    ("G2-M5", "g2_f05", "Go name matched by name_key", CH,
     "|| go_name == Some(f))",
     "|| go_name.is_some_and(|g| name_key(g) == name_key(f)))"),
    ("G2-M6", "g2_f07", "candidate annotation dropped", CH,
     "Some(match cands.get(name) {",
     "Some(match cands.get(name).filter(|_| false) {"),
    ("G2-M7", "g2_f08", "reverse uniqueness removed", SU,
     "if added.len() == 1 && rivals == 1 {",
     "if added.len() == 1 {"),
    ("G2-M8", "g2_f09", "forward uniqueness removed", SU,
     "if added.len() == 1 && rivals == 1 {",
     "if !added.is_empty() && rivals == 1 {"),
    ("G2-M9", "g2_f10", "type equality removed", SU,
     ADDED,
     ADDED.replace("t.as_ref() == Some(ty)", "t.is_some()")),
    ("G2-M10", "g2_f11", "known-type requirement removed (untyped fields pair)", SU,
     RC, M10),
    ("G2-M11", "g2_f12", "scope equality removed", SU,
     ADDED,
     ADDED.replace("s == scope && ", "")),
]

os.makedirs(out, exist_ok=True)
for mid, killer, what, path, old, new in M:
    src = open(os.path.join(tree, path)).read()
    assert src.count(old) == 1, (mid, src.count(old))
    mut = src.replace(old, new)
    diff = difflib.unified_diff(src.splitlines(True), mut.splitlines(True), "a/" + path, "b/" + path)
    with open(os.path.join(out, f"{mid}.patch"), "w") as f:
        f.write(f"# {mid} declared-killer={killer}: {what}\n" + "".join(diff))
    print(mid, killer, what)
