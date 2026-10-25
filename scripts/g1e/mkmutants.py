#!/usr/bin/env python3
"""G1-E mutation operators G1E-M1 to G1E-M12 (docs/falsification/G1-E-SPEC.md
section H), each one textual edit of the implemented tree, written as a
unified diff with the operator's DECLARED killer fixture on its first line.
The patch is computed with difflib against the current files, so it applies
to a tree holding exactly this implementation.
Usage: mkmutants.py <tree> <out dir>"""
import difflib, os, sys

tree, out = sys.argv[1], sys.argv[2]
SU = "crates/invar-falsify/src/surface.rs"
CH = "crates/invar-falsify/src/check.rs"

M = [
    ("G1E-M1", "g1e_f02", "condition 1 removed (literal loss)", CH,
     "                if lh >= lb || opq.len() <= ob {",
     "                if opq.len() <= ob {"),
    ("G1E-M2", "g1e_f03", "condition 2 removed (opaque gain)", CH,
     "                if lh >= lb || opq.len() <= ob {",
     "                if lh >= lb {"),
    ("G1E-M3", "g1e_f04", "condition 3 removed (consumer read)", CH,
     "let hit: Vec<&String> = ks.iter().filter(|k| read_keys.contains(&name_key(k))).collect();",
     "let hit: Vec<&String> = ks.iter().collect();"),
    ("G1E-M4", "g1e_f05", "comment skip removed from E1", SU,
     "        if is_comment_line(line) {\n            continue;\n        }\n        for form in EMISSION_FORMS {",
     "        for form in EMISSION_FORMS {"),
    ("G1E-M5", "g1e_f06", "statement added despite residue (the residue gate G1-E shares with RS-2 and RS-0)", CH,
     "    if residue.is_empty() {\n        for (_, btext, hname, htext) in &rs_pairs {",
     "    if true {\n        for (_, btext, hname, htext) in &rs_pairs {"),
    ("G1E-M6", "g1e_f07", "payload always the first argument", SU,
     'let idx = usize::from(form == "send_result(");',
     "let idx = 0;"),
    ("G1E-M7", "g1e_f08", "OPAQUE accepts any non-literal expression", SU,
     "} else if ident_at(arg, 0).is_some_and(|id| id.len() == arg.len()) && !KEYWORDS.contains(&arg) {",
     "} else if !arg.is_empty() && !KEYWORDS.contains(&arg) {"),
    ("G1E-M8", "g1e_f09", "keyword exclusion removed", SU,
     "} else if ident_at(arg, 0).is_some_and(|id| id.len() == arg.len()) && !KEYWORDS.contains(&arg) {",
     "} else if ident_at(arg, 0).is_some_and(|id| id.len() == arg.len()) {"),
    ("G1E-M9", "g1e_f10", "whole-name boundary removed", SU,
     "let whole = form.starts_with('.') || at == 0 || !is_ident_byte(line.as_bytes()[at - 1]);",
     "let whole = true;"),
    ("G1E-M10", "g1e_f11", "argument scan limited to the call line", SU,
     "let Some(args) = call_args(text, here + at + form.len()) else { continue };",
     "let Some(args) = call_args(line, at + form.len()) else { continue };"),
    ("G1E-M11", "g1e_f12", "keys taken at any depth", SU,
     "        if depth == 0 && entry_start && (c.is_ascii_alphabetic() || c == b'_') {",
     "        if c.is_ascii_alphabetic() || c == b'_' {"),
    ("G1E-M12", "g1e_f15", "code companion pair not scanned", CH,
     "        for (bname, btext, hname, htext) in &rs_pairs {\n            let (be, he) = (emissions(btext), emissions(htext));",
     "        for (bname, btext, hname, htext) in rs_pairs.iter().take(1) {\n            let (be, he) = (emissions(btext), emissions(htext));"),
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
