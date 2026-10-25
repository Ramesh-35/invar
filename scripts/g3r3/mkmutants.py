#!/usr/bin/env python3
"""G3 r3 mutation operators: the r2 set M1 to M14 (M3, M4, M6 with corrected declared killers) plus M8a, M8b, M8c and M15. Original r2 docstring follows.\nG3 mutation operators M1 to M14 (docs/falsification/G3-SPEC.md section 8),
each one textual edit of the implemented tree, written as a unified diff
with the operator's DECLARED killer fixture on its first line. The patch is
computed with difflib against the current files, so it applies to a tree
holding exactly this implementation.
Usage: mkmutants.py <tree> <out dir>"""
import difflib, os, sys

tree, out = sys.argv[1], sys.argv[2]
SU = "crates/invar-falsify/src/surface.rs"
CH = "crates/invar-falsify/src/check.rs"

M = [
    ("M1", "g3_f02", "V5 extraction removed", SU,
     "    if is_comment_line(line) {\n        return Vec::new();\n    }\n    let b = line.as_bytes();",
     "    if true {\n        return Vec::new();\n    }\n    let b = line.as_bytes();"),
    ("M2", "g3_f01", "P6 window set to same-line only", SU,
     "while window.len() < 3 && ends_cont(prev) {",
     "while window.len() < 1 && ends_cont(prev) {"),
    ("M3", "g3_f19", "P6 fires without the default-word requirement", SU,
     'if !b[i].is_ascii_digit() && w.to_ascii_lowercase().contains("default") {',
     "if !b[i].is_ascii_digit() {"),
    ("M4", "g3_f18", "TypeIdent selector requirement dropped", SU,
     '        if l.contains("version") || l.contains("format") {\n            let sep',
     "        if true {\n            let sep"),
    ("M5", "g3_f03", "non-numeric variants accepted as values", SU,
     "                if !digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit()) {\n                    out.push((l, digits.to_string()));",
     "                if !var.is_empty() {\n                    out.push((l, var.to_ascii_lowercase()));"),
    ("M6", "g3_f20", "P6 key omits the type identifier", SU,
     'let key = format!("{} {ty}", trigger.to_ascii_lowercase());',
     'let key = format!("{}", trigger.to_ascii_lowercase()); let _ = &ty;'),
    ("M7", "g3_f07", "switch fires on value removal", SU,
     "        let new: Vec<String> = hv.difference(bv).cloned().collect();\n        if !new.is_empty() {\n            let old: Vec<String> = bv.iter().cloned().collect();\n            let k",
     "        let new: Vec<String> = hv.symmetric_difference(bv).cloned().collect();\n        if !new.is_empty() {\n            let old: Vec<String> = bv.iter().cloned().collect();\n            let k"),
    ("M8", "g3_f06", "comment-line skip removed from variant scanning", SU,
     "    if is_comment_line(line) {\n        return Vec::new();\n    }\n    let b = line.as_bytes();",
     "    let b = line.as_bytes();"),
    ("M9", "g3_f09", "G3 occurrences fire despite residue", CH,
     "    if residue.is_empty() && !reads.is_empty() {\n        for (name, old, new, where_, note) in &switches {",
     "    if !reads.is_empty() {\n        for (name, old, new, where_, note) in &switches {"),
    ("M10", "g3_f12", "witness renders NEW -> OLD", SU,
     'out.push((name.clone(), old.join("|"), new.join("|"), note));',
     'out.push((name.clone(), new.join("|"), old.join("|"), note));'),
    ("M11", "g3_f13", "site-key collision: P6 key drops the trigger identifier (type-only merge)", SU,
     'let key = format!("{} {ty}", trigger.to_ascii_lowercase());',
     'let key = format!("default {ty}"); let _ = trigger;'),
    ("M12", "g3_f17", "continuation gate removed (window crosses a closed statement)", SU,
     "while window.len() < 3 && ends_cont(prev) {",
     "while window.len() < 3 { let _ = ends_cont(prev);"),
    ("M13", "g3_f14", "same-key multiplicity not disclosed", SU,
     "            let note = if k >= 2 {",
     "            let note = if k >= usize::MAX {"),
    ("M14", "g3_f16", "V5 dot-path matched by substring instead of whole identifiers", SU,
     "                let digits = var.strip_prefix(['V', 'v']).unwrap_or(var);",
     "                let s = var.strip_prefix(['V', 'v']).unwrap_or(var);\n                let digits = &s[..s.bytes().take_while(|c| c.is_ascii_digit()).count()];"),
]

M += [
    ("M8a", "g3_f21", "guard A removed: default_selectors no longer skips comment lines (shared with F1)", SU,
     "        {\n            continue;\n        }\n        let b = raw.as_bytes();",
     "        {\n            let _ = t;\n        }\n        let b = raw.as_bytes();"),
    ("M8b", "g3_f22", "guard B removed: a comment line can be a P6 trigger", SU,
     "        if is_comment_line(raw) {\n            continue;\n        }\n        let Some(trigger)",
     "        let Some(trigger)"),
    ("M8c", "g3_f23", "guard C removed: comment lines enter the P6 window instead of being stepped over", SU,
     "            while j < lines.len() && is_comment_line(lines[j]) {\n                j += 1;\n            }\n",
     ""),
    ("M15", "g3_f24", "blank lines skipped like comments (window crosses a blank line)", SU,
     "while j < lines.len() && is_comment_line(lines[j]) {",
     "while j < lines.len() && (is_comment_line(lines[j]) || lines[j].trim().is_empty()) {"),
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
