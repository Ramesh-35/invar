#!/usr/bin/env python3
"""JUDGE-TESTS r0 (docs/falsification/JUDGE-TESTS-SPEC.md): a document the
base schema accepted and the head schema rejects is a break, whoever wrote
it.

    tests.py BASE HEAD DIR [--format text|json|github]

Every file in DIR is an instance document (.json, .toml, .yaml, .yml). One
the base accepts and the head rejects is a witness: VIOLATED, exit 1. There
is no HOLDS here: a corpus of documents is not exhaustive, so otherwise the
verdict is NOT EVALUATED, exit 4, with its reason. Exit 3 is an error, not
a verdict. Validation is Python jsonschema on the schema's own draft; format
is never asserted; nothing outside a schema document is resolved.
"""
import json
import os
import sys

REASONS = {
    "no_instances": "no document in {d} could be read",
    "no_base_valid_instance": "no document in {d} is valid at {b}, so none is a promise",
    "unverifiable": "{n} document(s) valid at {b} could not be checked at {h}",
    "instances_pass": "{n} document(s) valid at {b} still pass at {h}; documents are not exhaustive",
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


def main(argv):
    import argparse

    class Usage(argparse.ArgumentParser):
        def error(self, message):
            fail("usage: tests.py BASE HEAD DIR [--format text|json|github]")

    p = Usage(add_help=False)
    p.add_argument("base")
    p.add_argument("head")
    p.add_argument("dir")
    p.add_argument("--format", default="text", choices=["text", "json", "github"])
    a = p.parse_args(argv)
    base_path, head_path, d, fmt = a.base, a.head, a.dir, a.format

    try:
        import jsonschema
        from jsonschema import validators
    except ImportError:
        fail("python package jsonschema is not installed")

    def schema(path):
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
        return cls(s)

    base, head = schema(base_path), schema(head_path)

    def errors(v, inst):
        """The first validation error as a one-element list, [] when valid, or
        a string saying why validation raised. One error is enough: a
        document is a promise or not, a break or not."""
        try:
            e = next(v.iter_errors(inst), None)
        except Exception as e:
            return f"{type(e).__name__}: {str(e)[:160]}"
        return [] if e is None else [e]

    try:
        names = sorted(n for n in os.listdir(d) if os.path.isfile(os.path.join(d, n)))
    except OSError as e:
        fail(f"reading {d}: {e}")

    witnesses, not_checked = [], []
    read = len(names)
    not_promises = passes = unreadable = 0
    for name in names:
        inst, why = load_instance(os.path.join(d, name))
        if why is not None:
            unreadable += 1
            not_checked.append({"file": name, "reason": why})
            continue
        eb = errors(base, inst)
        if isinstance(eb, str):
            not_checked.append({"file": name, "reason": f"at {base_path}: {eb}"})
            continue
        if eb:
            not_promises += 1
            continue
        eh = errors(head, inst)
        if isinstance(eh, str):
            not_checked.append({"file": name, "reason": f"at {head_path}: {eh}"})
            continue
        if eh:
            witnesses.append({"file": name, "pointer": pointer(eh[0].absolute_path), "message": eh[0].message})
        else:
            passes += 1

    base_valid = len(witnesses) + passes + sum(1 for x in not_checked if x["reason"].startswith(f"at {head_path}:"))
    if witnesses:
        verdict, reason = "VIOLATED", None
    elif read - unreadable == 0:
        verdict, reason = "NOT EVALUATED", "no_instances"
    elif base_valid == 0 and not any(x["reason"].startswith(f"at {base_path}:") for x in not_checked):
        verdict, reason = "NOT EVALUATED", "no_base_valid_instance"
    elif not_checked:
        verdict, reason = "NOT EVALUATED", "unverifiable"
    else:
        verdict, reason = "NOT EVALUATED", "instances_pass"

    counts = {"read": read, "not_promises": not_promises, "pass": passes, "not_checked": len(not_checked)}
    if fmt == "json":
        out = {"verdict": verdict, "reason": reason, "base": base_path, "head": head_path,
               "witnesses": witnesses, "not_checked": not_checked, "instances": counts}
        print(json.dumps(out, sort_keys=True, indent=2))
    elif fmt == "github":
        for w in witnesses:
            print(f"::error file={head_path}::{w['file']} at {w['pointer']}: {w['message']}")
        if not_checked:
            print("::warning::not checked: " + "; ".join(f"{x['file']} ({x['reason']})" for x in not_checked))
    else:
        if verdict == "VIOLATED":
            print(f"VIOLATED: {len(witnesses)} document(s) valid at {base_path} are rejected by {head_path}.")
        else:
            n = base_valid if reason == "instances_pass" else sum(1 for x in not_checked if x["reason"].startswith("at "))
            print(f"NOT EVALUATED ({reason}): {REASONS[reason].format(d=d, b=base_path, h=head_path, n=n)}.")
        for w in witnesses:
            print(f"  rejected: {w['file']} at {w['pointer']}: {w['message']}")
        for x in not_checked:
            print(f"  not checked: {x['file']}: {x['reason']}")
        print(f"  instances: {read} read, {not_promises} not promises, {passes} pass, {len(not_checked)} not checked")
    sys.exit(1 if witnesses else 4)


if __name__ == "__main__":
    main(sys.argv[1:])
