#!/usr/bin/env python3
"""Hand-built instances for the replay's real commits that the engine
built no counterexample for, and the three commits found not to be
breaks. Checked with Python jsonschema against the schema files at each
commit and its parent. Usage: python3 hand-checks.py <SchemaStore clone>
"""
import json, subprocess, sys
import jsonschema

ss = sys.argv[1]
def show(rev, f):
    return json.loads(subprocess.run(["git", "-C", ss, "show", f"{rev}:src/schemas/json/{f}"], capture_output=True, text=True, check=True).stdout)
def valid(s, i):
    return jsonschema.validators.validator_for(s)(s).is_valid(i)
def full(c):
    return subprocess.run(["git", "-C", ss, "rev-parse", c], capture_output=True, text=True, check=True).stdout.strip()

def colors(rev):
    req = show(rev, "ttheme-palette.json")["properties"]["colors"]["required"]
    c = {k: "#000000" for k in req}
    c["ansi"] = ["#000000"] * 16
    return c

BREAKS = [
    ("18132a01a", "pnpm-workspace.json", {"include": []}),
    ("6b9673c4c", "pyproject.json", {"build-system": {"requires": [], "build-backend": ""}}),
    ("8ae3e0c8c", "zeropress-wxr-import-base-0.7.json", {"version": "0.7", "import": {"media_from": "https://a.example/x", "media_to": "https://b.example/y"}}),
    ("9497fde91", "codex-hooks.json", {"hooks": {"Stop": [{"hooks": [{"type": "command", "command": "true"}]}]}, "$schema": "x"}),
    ("ec65ccfaf", "ttheme-palette.json", None),
    ("f1b0527c4", "publiccode.json", {"publiccodeYmlVersion": "0.4", "name": "x", "url": "https://example.org/x", "platforms": [], "developmentStatus": "stable", "softwareType": "library", "description": {"en": {"shortDescription": "x"}}, "legal": {"license": "MIT"}, "maintenance": {"type": "none"}, "localisation": {"localisationReady": True, "availableLanguages": []}}),
    ("a83ce6f19", "package.json", {"version": ""}),
    # JUDGE-ITEMS r0: breaks inside array elements.
    ("2f74efc3f", "claude-code-keybindings.json", {"bindings": [{"context": "Doctor", "bindings": {}}]}),
    ("585707189", "linutil-tabs.json", {"directories": ["\t"]}),
]
NOT_BREAKS = [
    ("4836277f2", "pyproject.json", [{"dependency-groups": {"dev": ["x"]}}], "dev still admitted by patternProperties, same schema"),
    ("0091e807a", "partial-poe.json", [{"verbosity": x} for x in (-1, 0, 1)], "every integer valid at base (-1..1) is in the new enum"),
    ("b514ab55c", "mkdocs-1.0.json", [{}, {"site_name": "x"}], "site_name was required at base too (inside allOf)"),
]
bad = 0
print("confirmed breaks (valid at parent, rejected at commit):")
for c, f, inst in BREAKS:
    c = full(c)
    if inst is None:
        inst = {"meta": {"name": "a", "signature": ["background", "foreground", "cursor"]}, "colors": colors(c + "^")}
    b, p = valid(show(c + "^", f), inst), valid(show(c, f), inst)
    ok = b and not p
    bad += not ok
    print(f"  {'ok  ' if ok else 'FAIL'} {c[:9]} {f}: {json.dumps(inst, separators=(',', ':'))[:160]}")
print("not breaks (same verdict at both refs for each probe):")
for c, f, insts, why in NOT_BREAKS:
    c = full(c)
    same = all(valid(show(c + "^", f), i) == valid(show(c, f), i) or not valid(show(c + "^", f), i) for i in insts)
    bad += not same
    print(f"  {'ok  ' if same else 'FAIL'} {c[:9]} {f}: {why}")
sys.exit(1 if bad else 0)
