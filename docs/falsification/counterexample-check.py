#!/usr/bin/env python3
"""COUNTEREXAMPLE r0 independent check (docs/falsification/COUNTEREXAMPLE-SPEC.md).

Checks counterexamples with Python jsonschema, a validator INVAR does not
contain: each must be valid against the base schema and invalid against
the post schema.

  python3 counterexample-check.py fixtures
      the expected instances frozen in counterexample_test.rs
  python3 counterexample-check.py replay <invar binary> <SchemaStore clone>
      every counterexample the judge prints on the replay corpus's real
      commits (MANIFEST.tsv); prints coverage and reasons; exits 1 if any
      counterexample fails the check
"""
import json
import os
import subprocess
import sys
from collections import Counter

import jsonschema


def check(pre, post, instance):
    """(valid at base, valid at post), by the draft each schema declares."""
    def ok(schema):
        cls = jsonschema.validators.validator_for(schema)
        return cls(schema).is_valid(instance)
    return ok(pre), ok(post)


def field(f):
    return {"properties": {"f": f}}


CLOSED = {"additionalProperties": False, "properties": {"name": {"type": "string"}, "age": {"type": "integer"}}}
CLOSED_NARROWER = {"additionalProperties": False, "properties": {"name": {"type": "string"}}}
NESTED = lambda user: {"required": ["payload"], "properties": {"payload": {"type": "object", "required": ["user"], "properties": {"user": {"type": "object", "additionalProperties": False, "properties": user}}}}}

FIXTURES = [
    ("cx_f01", CLOSED, CLOSED_NARROWER, {"age": 0}),
    ("cx_f02a", field({"type": "number"}), field({"type": "integer"}), {"f": 0.5}),
    ("cx_f02b", field({"type": ["boolean", "null"]}), field({"type": "boolean"}), {"f": None}),
    ("cx_f03", field({"enum": ["a", "b", "c"]}), field({"enum": ["a", "b"]}), {"f": "c"}),
    ("cx_f04a", field({"type": "integer", "maximum": 100}), field({"type": "integer", "maximum": 10}), {"f": 11}),
    ("cx_f04b", field({"type": "integer", "minimum": 0}), field({"type": "integer", "minimum": 5}), {"f": 4}),
    ("cx_f04c", field({"type": "integer", "minimum": 0}), field({"type": "integer", "minimum": 0.5}), {"f": 0}),
    ("cx_f04d", field({"type": "string", "maxLength": 10}), field({"type": "string", "maxLength": 3}), {"f": "aaaa"}),
    ("cx_f04e", field({"type": "array", "items": {"type": "string"}}), field({"type": "array", "items": {"type": "string"}, "minItems": 2}), {"f": [""]}),
    ("cx_f05", {"properties": {"id": {"type": "string"}}}, {"required": ["id"], "properties": {"id": {"type": "string"}}}, {}),
    ("cx_f06", NESTED({"zipcode": {"type": "string"}, "name": {"type": "string"}}), NESTED({"name": {"type": "string"}}), {"payload": {"user": {"zipcode": ""}}}),
]


def fixtures():
    bad = 0
    for name, pre, post, inst in FIXTURES:
        b, p = check(pre, post, inst)
        verdict = "ok" if (b and not p) else "FAIL"
        bad += verdict == "FAIL"
        print(f"{name}: base valid={b} post valid={p} {verdict} {json.dumps(inst, separators=(',', ':'))}")
    print(f"fixtures: {len(FIXTURES) - bad} of {len(FIXTURES)} confirmed by jsonschema {jsonschema_version()}")
    return 1 if bad else 0


def jsonschema_version():
    try:
        from importlib.metadata import version
        return version("jsonschema")
    except Exception:
        return "?"


def git(repo, *args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True, check=True).stdout


def replay(invar, ss):
    here = os.path.dirname(os.path.abspath(__file__))
    manifest = os.path.join(here, "schemastore-replay", "MANIFEST.tsv")
    rows = [l.rstrip("\n").split("\t") for l in open(manifest) if not l.startswith("#") and not l.startswith("commit\t")]
    real = sorted({r[0] for r in rows if r[4] == "real"})
    work = os.path.join(os.environ.get("TMPDIR", "/tmp"), "invar-cx-worktree")
    subprocess.run(["git", "-C", ss, "worktree", "remove", "--force", work], capture_output=True)
    subprocess.run(["git", "-C", ss, "worktree", "add", "-q", "--detach", work, real[0]], check=True, capture_output=True)
    witnesses = built = failed = 0
    reasons = Counter()
    for c in real:
        subprocess.run(["git", "-C", work, "checkout", "-q", "--detach", c], check=True, capture_output=True)
        out = subprocess.run([invar, "judge", "--contracts", "src/schemas/json/*.json", "--base", c + "^", "--format", "json"], cwd=work, capture_output=True, text=True).stdout
        for k in json.loads(out)["contracts"]:
            if k["verdict"] != "VIOLATED":
                continue
            pre = json.loads(git(work, "show", f"{c}^:{k['contract']}"))
            post = json.load(open(os.path.join(work, k["contract"])))
            for w in k["witnesses"]:
                witnesses += 1
                if w.get("counterexample_reason") is not None or "counterexample" not in w:
                    reasons[w.get("counterexample_reason", "missing key")] += 1
                    continue
                built += 1
                b, p = check(pre, post, w["counterexample"])
                if not (b and not p):
                    failed += 1
                    print(f"FAIL {c[:9]} {k['contract']} {w['kind']} {w['path']}: base valid={b} post valid={p} {json.dumps(w['counterexample'])[:200]}")
    subprocess.run(["git", "-C", ss, "worktree", "remove", "--force", work], capture_output=True)
    print(f"real commits {len(real)}: VIOLATED witnesses {witnesses}, counterexamples built {built}, failed the independent check {failed}")
    for r, n in sorted(reasons.items(), key=lambda x: (-x[1], x[0])):
        print(f"  no counterexample built ({n}): {r}")
    print("COUNTEREXAMPLE CHECK: " + ("PASS" if failed == 0 else "FAIL"))
    return 1 if failed else 0


if __name__ == "__main__":
    if len(sys.argv) >= 2 and sys.argv[1] == "fixtures":
        sys.exit(fixtures())
    if len(sys.argv) == 4 and sys.argv[1] == "replay":
        sys.exit(replay(os.path.abspath(sys.argv[2]), os.path.abspath(sys.argv[3])))
    print(__doc__)
    sys.exit(3)
