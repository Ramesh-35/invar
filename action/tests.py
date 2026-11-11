#!/usr/bin/env python3
"""JUDGE-TESTS r0 and r1 (docs/falsification/JUDGE-TESTS-SPEC.md,
JUDGE-TESTS-R1-SPEC.md): a document the base schema accepted and the head
schema rejects is a break, whoever wrote it, when two validators say so.

    tests.py BASE HEAD DIR [--format text|json|github] [--validators jsonschema|jsonschema,ajv]
             [--base-siblings DIR] [--head-siblings DIR]

Every file in DIR is an instance document (.json, .toml, .yaml, .yml). A
document both validators accept at the base and both reject at the head is
a witness: VIOLATED, exit 1. There is no HOLDS here: a corpus of documents
is not exhaustive, so otherwise the verdict is NOT EVALUATED, exit 4, with
its reason. A split between the validators is `disputed`, never a witness
and never a pass. With one validator only, nothing is VIOLATED: rejections
are `unconfirmed`. Exit 3 is an error, not a verdict. Python jsonschema and
Ajv 8 (through tests-ajv.mjs) on the schema's own draft; format is never
asserted. JUDGE-TESTS r3: a `$ref` to another file is read from that
side's siblings directory (default: the schema file's own directory) by the
last segment of the URI, then with `.json` added; nothing is ever fetched.
"""
import json
import os
import shutil
import subprocess
import sys

REASONS = {
    "no_instances": "no document in {d} could be read",
    "no_base_valid_instance": "no document in {d} is valid at {b}, so none is a promise",
    "disputed": "{n} document(s) get different answers from jsonschema and ajv",
    "unverifiable": "{n} document(s) valid at {b} could not be checked at {h}",
    "instances_pass": "{n} document(s) valid at {b} still pass at {h}; documents are not exhaustive",
    "unconfirmed": "{n} document(s) rejected by jsonschema alone; a break needs two validators",
}


def fail(msg):
    print(f"ERROR (not a verdict): {msg}")
    sys.exit(3)


def load_instance(path):
    """The document in `path`, or a string saying why it cannot be read."""
    ext = os.path.splitext(path)[1].lower()
    try:
        if ext == ".json":
            with open(path, "rb") as f:
                return json.load(f), None
        if ext == ".toml":
            import tomllib
            with open(path, "rb") as f:
                return tomllib.load(f), None
        if ext in (".yaml", ".yml"):
            try:
                import yaml
            except ImportError:
                return None, "no YAML reader installed (PyYAML)"
            with open(path, "rb") as f:
                return yaml.safe_load(f), None
        return None, f"unknown extension {ext or '(none)'}"
    except Exception as e:  # a parse failure of any kind is a reason, not a crash
        return None, f"{type(e).__name__}: {str(e)[:160]}"


def pointer(path):
    return "".join("/" + str(p).replace("~", "~0").replace("/", "~1") for p in path)


def kind(opinion):
    """valid, invalid or error, for an opinion from either validator."""
    if opinion == "valid":
        return "valid"
    return "invalid" if "invalid" in opinion else "error"


def sibling_path(siblings, uri):
    """The file a `$ref` URI names in `siblings`, or None: the last path
    segment of the URI (no fragment, no query), then that with `.json`.
    Only the directory itself is read."""
    target = uri.split("#", 1)[0].split("?", 1)[0].rstrip("/")
    name = os.path.basename(target.replace("\\", "/"))
    if not name or name in (".", ".."):
        return None
    for candidate in ([name, name + ".json"] if not name.endswith(".json") else [name]):
        path = os.path.join(siblings, candidate)
        if os.path.isfile(path):
            return path
    return None


def ajv_opinions(base_doc, head_doc, instances, base_siblings, head_siblings):
    """Ajv's answers per file and side, or a string saying why Ajv did not run."""
    node = shutil.which("node")
    if node is None:
        return "node is not on the PATH"
    adapter = os.path.join(os.path.dirname(os.path.abspath(__file__)), "tests-ajv.mjs")
    payload = json.dumps({"base": base_doc, "head": head_doc, "instances": instances,
                          "base_siblings": os.path.abspath(base_siblings), "head_siblings": os.path.abspath(head_siblings)})
    try:
        p = subprocess.run([node, adapter], input=payload, capture_output=True, text=True,
                           cwd=os.path.dirname(adapter), timeout=600)
    except Exception as e:
        return f"{type(e).__name__}: {str(e)[:160]}"
    if p.returncode != 0:
        return "the Ajv adapter failed: " + p.stderr.strip().splitlines()[-1][:160] if p.stderr.strip() else "the Ajv adapter failed"
    try:
        return json.loads(p.stdout)
    except Exception as e:
        return f"the Ajv adapter returned no JSON: {type(e).__name__}"


def main(argv):
    import argparse

    class Usage(argparse.ArgumentParser):
        def error(self, message):
            fail("usage: tests.py BASE HEAD DIR [--format text|json|github] [--validators jsonschema|jsonschema,ajv] [--base-siblings DIR] [--head-siblings DIR]")

    p = Usage(add_help=False)
    p.add_argument("base")
    p.add_argument("head")
    p.add_argument("dir")
    p.add_argument("--format", default="text", choices=["text", "json", "github"])
    p.add_argument("--validators", default="jsonschema,ajv", choices=["jsonschema", "jsonschema,ajv"])
    p.add_argument("--base-siblings", default=None)
    p.add_argument("--head-siblings", default=None)
    a = p.parse_args(argv)
    base_siblings = a.base_siblings or (os.path.dirname(os.path.abspath(a.base)))
    head_siblings = a.head_siblings or (os.path.dirname(os.path.abspath(a.head)))
    base_path, head_path, d, fmt = a.base, a.head, a.dir, a.format
    validators_wanted = a.validators.split(",")

    try:
        import jsonschema
        from jsonschema import validators
        from referencing import Registry, Resource
        from referencing.exceptions import NoSuchResource
        from referencing.jsonschema import DRAFT7
    except ImportError:
        fail("python packages jsonschema and referencing are not installed")

    def local_registry(siblings):
        # r3: a reference to another file is read from `siblings`, never fetched.
        def retrieve(uri):
            path = sibling_path(siblings, uri)
            if path is None:
                raise NoSuchResource(ref=uri)
            try:
                with open(path, "rb") as f:
                    # A sibling without $schema is read as draft-07, as the schema itself would be.
                    return Resource.from_contents(json.load(f), default_specification=DRAFT7)
            except NoSuchResource:
                raise
            except Exception as e:
                raise NoSuchResource(ref=f"{uri} ({type(e).__name__})")
        return Registry(retrieve=retrieve)

    def schema(path, siblings):
        try:
            with open(path, "rb") as f:
                s = json.load(f)
        except Exception as e:
            fail(f"reading {path}: {type(e).__name__}: {str(e)[:160]}")
        cls = validators.validator_for(s, default=jsonschema.Draft7Validator)
        try:
            # The metaschema check asserts no format: a `pattern` Python's `re`
            # cannot compile is a valid schema, and each document it guards
            # is reported unverifiable below, never the whole change an error.
            cls.check_schema(s, format_checker=None)
        except Exception as e:
            fail(f"{path} is not a valid schema: {str(e)[:160]}")
        return s, cls(s, registry=local_registry(siblings))

    (base_doc, base), (head_doc, head) = schema(base_path, base_siblings), schema(head_path, head_siblings)

    def opinion(v, inst):
        """jsonschema's answer: valid, {invalid: {pointer, message}} or {error}.
        One error is enough: a document is a promise or not, a break or not."""
        try:
            e = next(v.iter_errors(inst), None)
        except Exception as e:
            return {"error": f"{type(e).__name__}: {str(e)[:160]}"}
        return "valid" if e is None else {"invalid": {"pointer": pointer(e.absolute_path), "message": e.message}}

    try:
        names = sorted(n for n in os.listdir(d) if os.path.isfile(os.path.join(d, n)))
    except OSError as e:
        fail(f"reading {d}: {e}")

    read = len(names)
    docs, not_checked = [], []
    unreadable = 0
    for name in names:
        inst, why = load_instance(os.path.join(d, name))
        if why is not None:
            unreadable += 1
            not_checked.append({"file": name, "reason": why})
            continue
        docs.append({"file": name, "doc": inst})

    ajv = None
    ajv_unavailable = None
    if "ajv" in validators_wanted:
        ajv = ajv_opinions(base_doc, head_doc, docs, base_siblings, head_siblings)
        if isinstance(ajv, str):
            ajv_unavailable, ajv = ajv, None
    ran = ["jsonschema", "ajv"] if ajv is not None else ["jsonschema"]

    witnesses, disputed, unconfirmed = [], [], []
    not_promises = passes = 0
    for item in docs:
        name, inst = item["file"], item["doc"]
        jb = opinion(base, inst)
        ab = ajv[name]["base"] if ajv is not None else None
        if kind(jb) == "error" or (ab is not None and kind(ab) == "error"):
            who = jb if kind(jb) == "error" else ab
            not_checked.append({"file": name, "reason": f"at {base_path}: {who['error']}"})
            continue
        if ab is not None and kind(jb) != kind(ab):
            disputed.append({"file": name, "side": "base", "jsonschema": kind(jb), "ajv": kind(ab)})
            continue
        if kind(jb) == "invalid":
            not_promises += 1
            continue
        jh = opinion(head, inst)
        ah = ajv[name]["head"] if ajv is not None else None
        if kind(jh) == "error" or (ah is not None and kind(ah) == "error"):
            who = jh if kind(jh) == "error" else ah
            not_checked.append({"file": name, "reason": f"at {head_path}: {who['error']}"})
            continue
        if ah is not None and kind(jh) != kind(ah):
            disputed.append({"file": name, "side": "head", "jsonschema": kind(jh), "ajv": kind(ah)})
            continue
        if kind(jh) == "valid":
            passes += 1
            continue
        w = {"file": name, "pointer": jh["invalid"]["pointer"], "message": jh["invalid"]["message"]}
        if ah is not None:
            w["ajv_message"] = ah["invalid"]["message"]
            witnesses.append(w)
        else:
            unconfirmed.append(w)

    promises = len(witnesses) + len(unconfirmed) + passes + sum(1 for x in disputed if x["side"] == "head") \
        + sum(1 for x in not_checked if x["reason"].startswith(f"at {head_path}:"))
    base_undecided = any(x["side"] == "base" for x in disputed) or any(x["reason"].startswith(f"at {base_path}:") for x in not_checked)
    if witnesses:
        verdict, reason = "VIOLATED", None
    elif read - unreadable == 0:
        verdict, reason = "NOT EVALUATED", "no_instances"
    elif promises == 0 and not base_undecided:
        verdict, reason = "NOT EVALUATED", "no_base_valid_instance"
    elif unconfirmed:
        verdict, reason = "NOT EVALUATED", "unconfirmed"
    elif disputed:
        verdict, reason = "NOT EVALUATED", "disputed"
    elif not_checked:
        verdict, reason = "NOT EVALUATED", "unverifiable"
    else:
        verdict, reason = "NOT EVALUATED", "instances_pass"

    counts = {"read": read, "not_promises": not_promises, "pass": passes,
              "not_checked": len(not_checked), "disputed": len(disputed)}
    if fmt == "json":
        out = {"verdict": verdict, "reason": reason, "base": base_path, "head": head_path,
               "validators": ran, "witnesses": witnesses, "disputed": disputed,
               "unconfirmed": unconfirmed, "not_checked": not_checked, "instances": counts}
        if ajv_unavailable:
            out["ajv_unavailable"] = ajv_unavailable
        print(json.dumps(out, sort_keys=True, indent=2))
    elif fmt == "github":
        for w in witnesses:
            print(f"::error file={head_path}::{w['file']} at {w['pointer']}: {w['message']} | ajv: {w['ajv_message']}")
        if unconfirmed:
            print("::warning::rejected by jsonschema alone, unconfirmed: " + "; ".join(f"{w['file']} at {w['pointer']}" for w in unconfirmed))
        if disputed:
            print("::warning::disputed between validators: " + "; ".join(f"{x['file']} (jsonschema {x['jsonschema']}, ajv {x['ajv']})" for x in disputed))
        if not_checked:
            print("::warning::not checked: " + "; ".join(f"{x['file']} ({x['reason']})" for x in not_checked))
    else:
        if verdict == "VIOLATED":
            print(f"VIOLATED: {len(witnesses)} document(s) valid at {base_path} are rejected by {head_path}.")
        else:
            n = {"instances_pass": promises, "unconfirmed": len(unconfirmed), "disputed": len(disputed)}.get(
                reason, sum(1 for x in not_checked if x["reason"].startswith("at ")))
            print(f"NOT EVALUATED ({reason}): {REASONS[reason].format(d=d, b=base_path, h=head_path, n=n)}.")
        if ajv_unavailable:
            print(f"  validators: jsonschema only ({ajv_unavailable})")
        for w in witnesses:
            print(f"  rejected: {w['file']} at {w['pointer']}: {w['message']} | ajv: {w['ajv_message']}")
        for w in unconfirmed:
            print(f"  unconfirmed: {w['file']} at {w['pointer']}: {w['message']}")
        for x in disputed:
            print(f"  disputed: {x['file']}: jsonschema {x['jsonschema']}, ajv {x['ajv']}")
        for x in not_checked:
            print(f"  not checked: {x['file']}: {x['reason']}")
        print(f"  instances: {read} read, {not_promises} not promises, {passes} pass, {len(not_checked)} not checked")
    sys.exit(1 if witnesses else 4)


if __name__ == "__main__":
    main(sys.argv[1:])
