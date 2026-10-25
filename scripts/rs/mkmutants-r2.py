#!/usr/bin/env python3
"""REQUEST-SIDE mutation operators RS-M1 to RS-M16
(docs/falsification/REQUEST-SIDE-SPEC.md r2, sections I and J; the r1 generator
scripts/rs/mkmutants.py is preserved as run), each one textual edit
of the implemented tree, written as a unified diff with the operator's
DECLARED killer fixture on its first line. The patch is computed with
difflib against the current files, so it applies to a tree holding exactly
this implementation.
Usage: mkmutants.py <tree> <out dir>"""
import difflib, os, sys

tree, out = sys.argv[1], sys.argv[2]
SU = "crates/invar-falsify/src/surface.rs"
CH = "crates/invar-falsify/src/check.rs"

M = [
    ("RS-M1", "rs_f01", "B2 builder-word readers removed", SU,
     "            let go = GO_ACCESSORS.iter().any(|w| pre.ends_with(w))\n                || GO_BUILDERS.iter().any(|w| ends_with_word(pre, w));",
     "            let go = GO_ACCESSORS.iter().any(|w| pre.ends_with(w)) || GO_BUILDERS.is_empty();"),
    ("RS-M2", "rs_f02b", "RS-1 condition 2 removed (head B2 reads ignored)", CH,
     "                || head_reads.iter().any(|r| r.contains_key(&k))\n",
     "                || head_reads.is_empty()\n"),
    ("RS-M3", "rs_f09b", "RS-1 condition 3 removed (literal guard)", CH,
     "                || rs_heads.iter().any(|t| has_string_literal(t, &k))\n",
     "                || rs_heads.is_empty()\n"),
    ("RS-M4", "rs_f04", "comment skip removed from B2", SU,
     "        if is_comment_line(line) {\n            continue;\n        }\n        for (s, _, k) in string_literals(line) {",
     "        for (s, _, k) in string_literals(line) {"),
    ("RS-M5", "rs_f05", "comment skip removed from B1", SU,
     "        if is_comment_line(line) {\n            continue;\n        }\n        for (s, e, k) in string_literals(line) {",
     "        for (s, e, k) in string_literals(line) {"),
    ("RS-M6", "rs_f03", "B1 subscript form no longer requires =", SU,
     "                && assigns(&post[1..]);",
     "                && !post.is_empty();"),
    ("RS-M7", "rs_f10", "RS-1 condition 4 removed (consumer write)", CH,
     "        let Some(wl) = writes.get(k) else { continue };",
     "        let empty = Vec::new();\n        let wl = writes.get(k).unwrap_or(&empty);"),
    ("RS-M8", "rs_f17", "code companion head not consulted", CH,
     "        rs_heads.push(cr_head.clone());\n",
     ""),
    ("RS-M9", "rs_f15", "B2 JavaScript member form removed", SU,
     "        for o in JS_OBJS {\n            let pat",
     "        for o in JS_OBJS.iter().take(0) {\n            let pat"),
    ("RS-M10", "rs_f16", "B1 builder-setter forms removed", SU,
     "            let setter = SETTERS.iter().any(|w| pre.ends_with(w));",
     "            let setter = SETTERS.is_empty();"),
    ("RS-M11", "rs_f12", "RS-2 convicts instead of stating", CH,
     "                if let Some(cl) = word_lines(&cr, &f).first() {\n                    unevaluated.push(format!(",
     "                if let Some(cl) = word_lines(&cr, &f).first() {\n                    residue.push(f.clone());\n                    unevaluated.push(format!("),
    ("RS-M12", "rs_f13", "RS-2 base requirement dropped", CH,
     "base_f.get(&f).is_none_or(|(bro, _)| *bro)",
     "base_f.get(&f).is_some_and(|(bro, _)| *bro)"),
    ("RS-M13", "rs_f11", "RS-2 and RS-0 added despite residue", CH,
     "    if residue.is_empty() {\n        for (_, btext, hname, htext) in &rs_pairs {",
     "    if true {\n        for (_, btext, hname, htext) in &rs_pairs {"),
    ("RS-M14", "rs_f06", "RS-0 opaque detection removed", CH,
     "        if let Some(ol) = opaque_payload(&cr) {",
     "        if let Some(ol) = opaque_payload(&cr).filter(|_| false) {"),
    ("RS-M15", "rs_f08c", "data/json whole-word requirement dropped", SU,
     "                    && !matches!(prev, Some(c) if is_ident_byte(c))\n                    && assigns(&rest[w.len()..])",
     "                    && assigns(&rest[w.len()..])"),
    ("RS-M16", "rs_f01", "RS-1 witness omits the base read line", CH,
     "(inbound read at {bname}:{bline} at base); written at {cname}",
     "(inbound read at {bname} at base); written at {cname}"),
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
