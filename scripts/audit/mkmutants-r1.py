#!/usr/bin/env python3
"""AUDIT mutation operators AU-M1 to AU-M9 and defect injections DI-1, DI-2
(docs/falsification/AUDIT-SPEC.md r1, sections F and I; the r0 generator
scripts/audit/mkmutants.py is preserved as run), each a textual edit of the
implemented tree written as a unified diff; the first line names the
operator and its DECLARED killer (DI patches name "suite": every conviction
fixture must turn into an exit-3 self-audit failure). Usage:
mkmutants.py <tree> <out dir>"""
import difflib, os, sys

tree, out = sys.argv[1], sys.argv[2]
CH = "crates/invar-falsify/src/check.rs"
LK = "crates/invar-falsify/src/lock.rs"

M = [
    ("AU-M1", "au_f02", "a missing line passes", [(CH,
     'None => Err(format!("AU-1 {site} is not a line of {consumer}")),', "None => Ok(()),")]),
    ("AU-M2", "au_f05", "token match replaced by substring match", [(CH,
     "Some(l) if line_tokens(l).iter().any(|t| forms.contains(t)) => Ok(()),",
     "Some(l) if forms.iter().any(|f| l.contains(f)) => Ok(()),")]),
    ("AU-M3", "au_f04", "read as forms ignored", [(CH,
     "    forms.extend(w.read_as.iter().map(String::as_str));", "    let _ = &w.read_as;")]),
    ("AU-M4", "au_f06", "site file name not checked", [(CH,
     ".filter(|(file, _)| *file == consumer)", ".filter(|_| true)")]),
    ("AU-M5", "au_f07b", "string-literal contents not tokenized", [(CH,
     "let mut out: Vec<&str> = crate::surface::string_literals(line).into_iter().map(|(_, _, c)| c).collect();",
     "let mut out: Vec<&str> = Vec::new();")]),
    ("AU-M6", "au_f08", "missing_from producer not checked", [(CH,
     "        if !names_producer(m, producers) {", "        if false {")]),
    ("AU-M7", "au_f09", "type-change place not checked", [(CH,
     "        if !names_producer(place, producers) {", "        if false {")]),
    ("AU-M8", "au_f11", "RS-1 write lines not checked", [(CH,
     '                holds(&format!("{consumer}:{n}"), &[field])?;', "                let _ = n;")]),
    ("AU-M9", "au_f03", "line-content check removed", [(CH,
     'Some(_) => Err(format!("AU-1 {site} does not hold {}", forms.join(" or "))),', "Some(_) => Ok(()),")]),
    ("DI-1", "suite", "every witness site line and RS-1 write line off by one", [
     (CH, '            .map(|(_, l)| format!("{cname}:{l}"))', '            .map(|(_, l)| format!("{cname}:{}", l + 1))'),
     (CH, '                .map(|(_, l)| format!("{name}:{l}"))', '                .map(|(_, l)| format!("{name}:{}", l + 1))'),
     (CH, "        let at: Vec<String> = wl.iter().map(|l| l.to_string()).collect();", "        let at: Vec<String> = wl.iter().map(|l| (l + 1).to_string()).collect();"),
     (LK, '        let mut s: Vec<String> = sites.iter().map(|(_, l)| format!("{cname}:{l}")).collect();', '        let mut s: Vec<String> = sites.iter().map(|(_, l)| format!("{cname}:{}", l + 1)).collect();')]),
    ("DI-2", "suite", "every witness site and RS-1 write list names the wrong file", [
     (CH, '            .map(|(_, l)| format!("{cname}:{l}"))', '            .map(|(_, l)| format!("wrong.file:{l}"))'),
     (CH, '                .map(|(_, l)| format!("{name}:{l}"))', '                .map(|(_, l)| format!("wrong.file:{l}"))'),
     (CH, '"{head_name} at head: no inbound read and no literal of {k} remain (inbound read at {bname}:{bline} at base); written at {cname}:{}",', '"{head_name} at head: no inbound read and no literal of {k} remain (inbound read at {bname}:{bline} at base); written at wrong.file:{}",'),
     (LK, '        let mut s: Vec<String> = sites.iter().map(|(_, l)| format!("{cname}:{l}")).collect();', '        let mut s: Vec<String> = sites.iter().map(|(_, l)| format!("wrong.file:{l}")).collect();')]),
]

os.makedirs(out, exist_ok=True)
for mid, killer, what, edits in M:
    diffs = []
    for path, old, new in edits:
        src = open(os.path.join(tree, path)).read()
        assert src.count(old) == 1, (mid, path, old)
        mut = src.replace(old, new)
        diffs.append("".join(difflib.unified_diff(src.splitlines(True), mut.splitlines(True), "a/" + path, "b/" + path)))
    # Edits to the same file are applied together.
    by_file = {}
    for path, old, new in edits:
        by_file.setdefault(path, []).append((old, new))
    text = ""
    for path, pairs in by_file.items():
        src = open(os.path.join(tree, path)).read(); mut = src
        for old, new in pairs: mut = mut.replace(old, new)
        text += "".join(difflib.unified_diff(src.splitlines(True), mut.splitlines(True), "a/" + path, "b/" + path))
    with open(os.path.join(out, f"{mid}.patch"), "w") as f:
        f.write(f"# {mid} declared-killer={killer}: {what}\n" + text)
    print(mid, killer, what)
