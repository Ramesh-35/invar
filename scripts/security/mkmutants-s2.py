#!/usr/bin/env python3
"""SECURITY S2 mutation operators S2-M1 to S2-M9 (docs/falsification/
SECURITY-SPEC.md r2, sections I and J), each a textual edit of the
implemented tree written as a unified diff with the operator's DECLARED
killer on its first line. An edit names how many places it changes (the
sink macros repeat four times per file). Usage: mkmutants-s2.py <tree> <out dir>"""
import difflib, os, sys

tree, out = sys.argv[1], sys.argv[2]
CH = "crates/invar-falsify/src/check.rs"; TE = "crates/invar-falsify/src/term.rs"
MA = "crates/invar-falsify/src/main.rs"; DE = "crates/invar-falsify/src/dependents.rs"; MC = "crates/invar-falsify/src/mcp.rs"

M = [
    ("S2-M1", "s2_f05", "main.rs sink macros removed", [(MA, "invar_falsify::term::neutralize(&format!", "std::convert::identity(&format!", 4)]),
    ("S2-M2", "s2_f02", "renderer value escaping removed", [
        (CH, "    let v = crate::term::escape_value;", "    let v = |x: &str| x.to_string();", 1),
        (CH, "witness_text(w, &o.kind, crate::term::escape_value)", "witness_text(w, &o.kind, |x: &str| x.to_string())", 1)]),
    ("S2-M3", "s2_f04", "bidirectional characters left out of the escape set", [(TE,
        "    c.is_control() || matches!(c, '\\u{202a}'..='\\u{202e}' | '\\u{2066}'..='\\u{2069}')", "    c.is_control()", 1)]),
    ("S2-M4", "s2_f08", "markdown code spans not escaped", [(CH, "    let cleaned = crate::term::escape_value(&cleaned);", "    let cleaned = cleaned;", 1)]),
    ("S2-M5", "s2_f11", "the walk follows symbolic links", [(DE,
        "            let Ok(ft) = e.file_type() else { continue };",
        "            let Ok(ft) = std::fs::metadata(e.path()).map(|m| m.file_type()) else { continue };", 1)]),
    ("S2-M6", "s2_f12", "depth bound removed", [(DE, "        if depth > max_depth {", "        if false && depth > max_depth {", 1)]),
    ("S2-M7", "s2_f12", "entry bound removed", [(DE, "            if seen > max_entries {", "            if false && seen > max_entries {", 1)]),
    ("S2-M8", "s2_f13", "MCP path restriction removed", [(MC, "        if !inside {", "        if false && !inside {", 1)]),
    ("S2-M9", "s2_f14", "dependents.rs sink macros removed", [(DE, "crate::term::neutralize(&format!", "std::convert::identity(&format!", 4)]),
]

os.makedirs(out, exist_ok=True)
for mid, killer, what, edits in M:
    by_file = {}
    for path, old, new, n in edits:
        by_file.setdefault(path, []).append((old, new, n))
    text = ""
    for path, pairs in by_file.items():
        src = open(os.path.join(tree, path)).read(); mut = src
        for old, new, n in pairs:
            assert mut.count(old) == n, (mid, path, old, mut.count(old))
            mut = mut.replace(old, new)
        text += "".join(difflib.unified_diff(src.splitlines(True), mut.splitlines(True), "a/" + path, "b/" + path))
    with open(os.path.join(out, f"{mid}.patch"), "w") as f:
        f.write(f"# {mid} declared-killer={killer}: {what}\n" + text)
    print(mid, killer, what)
