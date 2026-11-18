"""OPEN-ADD r0 adversarial cases. Each case: base schema, head schema, the verdict the ruling requires.
Every counterexample INVAR prints is checked by an independent validator (python-jsonschema, dialect from
$schema): it must be valid under the base and invalid under the head.
Usage: python3 adversarial.py PATH_TO_INVAR"""
import json, os, subprocess, sys, tempfile
import jsonschema
from jsonschema.validators import validator_for

INVAR = sys.argv[1]
D7 = "http://json-schema.org/draft-07/schema#"
D4 = "http://json-schema.org/draft-04/schema#"
D2020 = "https://json-schema.org/draft/2020-12/schema"
def obj(props=None, **kw):
    s = {"type": "object", "properties": props or {}}; s.update(kw); return s

CASES = [
 # (id, why, base, head, allowed verdicts)
 ("A01", "open object gains a typed optional property", obj({"id": {"type": "string"}}), obj({"id": {"type": "string"}, "status": {"type": "string"}}), {"VIOLATED"}),
 ("A02", "the founder's example: bare open object gains status:string", {"type": "object"}, obj({"status": {"type": "string"}}), {"VIOLATED"}),
 ("A03", "closed object (additionalProperties false) gains a property: widening", obj({"id": {"type": "string"}}, additionalProperties=False), obj({"id": {"type": "string"}, "status": {"type": "string"}}, additionalProperties=False), {"HOLDS"}),
 ("A04", "closed by unevaluatedProperties false (2020-12): widening", dict(obj({"id": {"type": "string"}}, unevaluatedProperties=False), **{"$schema": D2020}), dict(obj({"id": {"type": "string"}, "s": {"type": "string"}}, unevaluatedProperties=False), **{"$schema": D2020}), {"HOLDS"}),
 ("A05", "added property asserts nothing ({})", obj({"id": {"type": "string"}}), obj({"id": {"type": "string"}, "meta": {}}), {"HOLDS"}),
 ("A06", "added property carries annotations only", obj(), obj({"meta": {"description": "free form", "examples": [1]}}), {"HOLDS"}),
 ("A07", "added property is `true`", obj(), obj({"meta": True}), {"HOLDS"}),
 ("A08", "base additionalProperties string, head declares string: no instance breaks", obj(additionalProperties={"type": "string"}), obj({"s": {"type": "string"}}, additionalProperties={"type": "string"}), {"HOLDS", "NOT EVALUATED"}),
 ("A09", "base additionalProperties string, head declares string minLength 3", obj(additionalProperties={"type": "string"}), obj({"s": {"type": "string", "minLength": 3}}, additionalProperties={"type": "string"}), {"VIOLATED"}),
 ("A10", "base patternProperties may already constrain the key", obj(patternProperties={"^s": {"type": "string"}}), obj({"status": {"type": "string"}}, patternProperties={"^s": {"type": "string"}}), {"NOT EVALUATED"}),
 ("A11", "added property admits every JSON type", obj(), obj({"x": {"type": ["string", "number", "boolean", "null", "array", "object"]}}), {"HOLDS", "NOT EVALUATED"}),
 ("A12", "added integer property", obj(), obj({"n": {"type": "integer"}}), {"VIOLATED"}),
 ("A13", "added const property", obj(), obj({"v": {"const": "x"}}), {"VIOLATED"}),
 ("A14", "added enum property", obj(), obj({"v": {"enum": [1, 2]}}), {"VIOLATED"}),
 ("A15", "added property under anyOf", obj(), obj({"v": {"anyOf": [{"type": "string"}, {"type": "number"}]}}), {"VIOLATED", "NOT EVALUATED"}),
 ("A16", "nested open object gains a typed property", obj({"a": obj({"b": {"type": "string"}})}), obj({"a": obj({"b": {"type": "string"}, "c": {"type": "boolean"}})}), {"VIOLATED"}),
 ("A17", "nested closed object gains a property", obj({"a": obj({"b": {"type": "string"}}, additionalProperties=False)}), obj({"a": obj({"b": {"type": "string"}, "c": {"type": "boolean"}}, additionalProperties=False)}), {"HOLDS"}),
 ("A18", "draft-04 open object gains a typed property", dict(obj(), **{"$schema": D4}), dict(obj({"s": {"type": "string"}}), **{"$schema": D4}), {"VIOLATED"}),
 ("A19", "added property with pattern only (unchecked keyword)", obj(), obj({"p": {"pattern": "^a"}}), {"NOT EVALUATED", "VIOLATED"}),
 ("A20", "added property that is also newly required", obj(), obj({"s": {"type": "string"}}, required=["s"]), {"VIOLATED"}),
 ("A21", "root has allOf beside properties", obj(allOf=[{"required": []}]), obj({"s": {"type": "string"}}, allOf=[{"required": []}]), {"VIOLATED", "NOT EVALUATED"}),
 ("A22", "added property of type object with its own required", obj(), obj({"o": {"type": "object", "required": ["k"]}}), {"VIOLATED"}),
 ("A23", "added property is $ref to a definition", dict(obj(), definitions={"S": {"type": "string"}}), dict(obj({"s": {"$ref": "#/definitions/S"}}), definitions={"S": {"type": "string"}}), {"VIOLATED", "NOT EVALUATED"}),
 ("A24", "added property is `false`", obj(), obj({"gone": False}), {"VIOLATED"}),
 ("A25", "base additionalProperties false is removed while a property is added: widening", obj(additionalProperties=False), obj({"s": {"type": "string"}}), {"HOLDS", "NOT EVALUATED"}),
]

def run(base, head):
    with tempfile.TemporaryDirectory() as d:
        b, h = os.path.join(d, "base.json"), os.path.join(d, "head.json")
        json.dump(base, open(b, "w")); json.dump(head, open(h, "w"))
        r = subprocess.run([INVAR, "judge", b, h, "--format", "json"], capture_output=True, text=True)
        return r.returncode, json.loads(r.stdout) if r.stdout.strip() else {"verdict": "ERROR", "stderr": r.stderr}

def valid(schema, inst):
    cls = validator_for(schema, default=jsonschema.Draft7Validator)
    return cls(schema).is_valid(inst)

bad = 0
for cid, why, base, head, allowed in CASES:
    code, body = run(base, head)
    v = body.get("verdict")
    problems = []
    if v not in allowed:
        problems.append("verdict %s, allowed %s" % (v, sorted(allowed)))
    for w in body.get("witnesses", []):
        ce = w.get("counterexample")
        if ce is not None:
            if not valid(base, ce): problems.append("counterexample %s is INVALID under the base" % json.dumps(ce))
            if valid(head, ce): problems.append("counterexample %s is VALID under the head" % json.dumps(ce))
    if v == "VIOLATED" and not any(w.get("counterexample") is not None for w in body.get("witnesses", [])):
        problems.append("VIOLATED with no counterexample")
    exp = {"HOLDS": 0, "VIOLATED": 1, "NOT EVALUATED": 4}.get(v)
    if exp is not None and code != exp: problems.append("exit %d for %s" % (code, v))
    ces = [w.get("counterexample") for w in body.get("witnesses", []) if w.get("counterexample") is not None]
    print("%s %-4s %-14s %s%s" % ("FAIL" if problems else "ok  ", cid, v, why, ("  ce=" + json.dumps(ces[0])) if ces else ""))
    for p in problems: print("        " + p)
    bad += bool(problems)
print("\n%d of %d cases failed" % (bad, len(CASES)))
sys.exit(1 if bad else 0)
