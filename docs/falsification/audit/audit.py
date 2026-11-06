#!/usr/bin/env python3
"""INVAR audit suite: replay a repository's schema history, keep only proof.

For every commit that changed a JSON schema matched by the pathspec, the
parent's and the commit's bytes are judged as a file pair
(`invar judge <parent.json> <commit.json> --format json`, v0.2.12 and
later). Every printed counterexample is then checked independently with
Python jsonschema: valid under the parent's schema, invalid under the
commit's. A counterexample that fails that check is not discarded: it is
a defect in INVAR, it is printed, and the gate fails (exit 1).

The report puts proof first: commits with a confirmed counterexample,
each with the instance and a command to reproduce it. VIOLATED findings
without an instance are listed after, with INVAR's stated reason, never
dropped. Files that are not JSON are counted and named as not judged.

  audit.py <invar> <clone> <pathspec> --github OWNER/REPO --out report.md
           [--rev HEAD] [--commits 500 | --since D --until D | --commits-from FILE]
"""
import argparse
import json
import os
import subprocess
import sys
import tempfile

import jsonschema

ap = argparse.ArgumentParser()
ap.add_argument("invar")
ap.add_argument("clone")
ap.add_argument("pathspec")
ap.add_argument("--github", required=True, help="OWNER/REPO, for commit links")
ap.add_argument("--out", required=True)
ap.add_argument("--rev", default="HEAD")
ap.add_argument("--commits", type=int, default=500)
ap.add_argument("--since")
ap.add_argument("--until")
ap.add_argument("--commits-from", help="file with one commit SHA per line (first tab field)")
a = ap.parse_args()
invar, clone = os.path.abspath(a.invar), os.path.abspath(a.clone)


def git(*args):
    return subprocess.run(["git", "-C", clone, *args], capture_output=True, check=True).stdout


def valid(schema, inst):
    return jsonschema.validators.validator_for(schema)(schema).is_valid(inst)


if a.commits_from:
    rows = [l.split("\t")[0].strip() for l in open(a.commits_from) if l.strip() and not l.startswith("#") and not l.startswith("commit\t")]
    commits = sorted(set(rows))
else:
    cmd = ["log", "--no-merges", "--format=%H", a.rev]
    if a.since or a.until:
        cmd[3:3] = [f"--since={a.since}"] * bool(a.since) + [f"--until={a.until}"] * bool(a.until)
    else:
        cmd.insert(3, f"-n{a.commits}")
    commits = git(*cmd, "--", a.pathspec).decode().split()

head_sha = git("rev-parse", a.rev).decode().strip()
confirmed, stated, failed, printed = [], [], [], 0
violated_commits, judged_pairs, not_json, added, deleted, skipped_root = set(), 0, [], 0, 0, 0
for c in commits:
    try:
        changes = git("diff", "--name-status", "--no-renames", "-z", f"{c}^", c, "--", a.pathspec).split(b"\0")
    except subprocess.CalledProcessError:
        skipped_root += 1  # the repository's first commit has no parent
        continue
    pairs = [(changes[i].decode(), changes[i + 1].decode()) for i in range(0, len(changes) - 1, 2)]
    for status, path in pairs:
        if status == "A":
            added += 1
            continue
        if status == "D":
            deleted += 1
            continue
        pre_b, post_b = git("show", f"{c}^:{path}"), git("show", f"{c}:{path}")
        try:
            pre, post = json.loads(pre_b), json.loads(post_b)
        except ValueError:
            not_json.append((c, path))
            continue
        with tempfile.TemporaryDirectory() as d:
            open(os.path.join(d, "base.json"), "wb").write(pre_b)
            open(os.path.join(d, "head.json"), "wb").write(post_b)
            p = subprocess.run([invar, "judge", "base.json", "head.json", "--format", "json"], cwd=d, capture_output=True, text=True)
        if p.returncode not in (0, 1, 4):
            print(f"ERROR {c[:9]} {path}: exit {p.returncode}: {(p.stderr or p.stdout)[:200]}")
            sys.exit(3)
        judged_pairs += 1
        r = json.loads(p.stdout)
        if r["verdict"] != "VIOLATED":
            continue
        violated_commits.add(c)
        for w in r["witnesses"]:
            if w["counterexample"] is None:
                stated.append((c, path, w))
                continue
            printed += 1
            if valid(pre, w["counterexample"]) and not valid(post, w["counterexample"]):
                confirmed.append((c, path, w))
            else:
                failed.append((c, path, w))

proved = sorted({c for c, _, _ in confirmed})
gh = f"https://github.com/{a.github}/commit/"
with open(a.out, "w") as f:
    f.write(f"# INVAR audit: {a.github}\n\n")
    f.write(f"At `{head_sha[:9]}`, pathspec `{a.pathspec}`, {len(commits)} commits that touched it.\n\n")
    f.write(f"- **{len(proved)} commits** changed a schema so that a document valid before is rejected after, each shown by an instance checked with Python `jsonschema`.\n")
    f.write(f"- {len(violated_commits - set(proved))} more commits were judged VIOLATED without an instance; they are listed at the end with the reason.\n")
    f.write(f"- {judged_pairs} schema changes judged; {added} files added and {deleted} deleted (a new or removed file has no before and after to compare); {len(not_json)} not JSON, so not judged.\n")
    f.write(f"- Instances that failed the independent check: {len(failed)}.\n\n")
    f.write("## Confirmed, with an instance\n\n")
    for c in proved:
        subj = git("log", "-1", "--format=%s", c).decode().strip()
        f.write(f"### [{c[:9]}]({gh}{c}) {subj}\n\n")
        for cc, path, w in confirmed:
            if cc != c:
                continue
            inst = json.dumps(w["counterexample"], separators=(",", ":"))
            f.write(f"`{path}`, {w['kind']} `{w['path']}`. Valid before, rejected after:\n\n```json\n{inst}\n```\n\n")
            f.write("Reproduce:\n\n```\n")
            f.write(f"git show {c[:9]}^:{path} > before.json && git show {c[:9]}:{path} > after.json\n")
            f.write(f"python3 -c 'import json,jsonschema as j; i=json.loads(r\"\"\"{inst}\"\"\"); v=lambda p: j.validators.validator_for(s:=json.load(open(p)))(s).is_valid(i); print(v(\"before.json\"), v(\"after.json\"))'  # True False\n```\n\n")
    f.write("## Judged VIOLATED, no instance built\n\n")
    for c in sorted(violated_commits - set(proved)):
        for cc, path, w in stated:
            if cc == c:
                f.write(f"- [{c[:9]}]({gh}{c}) `{path}` {w['kind']} `{w['path']}`: {w['counterexample_reason']}\n")
    if not_json:
        f.write("\n## Not JSON, not judged\n\n")
        for c, path in not_json:
            f.write(f"- {c[:9]} `{path}`\n")

print(f"commits {len(commits)}: VIOLATED {len(violated_commits)}, confirmed by a checked counterexample {len(proved)}, unconfirmed {len(violated_commits - set(proved))}")
print(f"counterexamples printed {printed}, failed the independent check {len(failed)}")
print(f"pairs judged {judged_pairs}; added {added}; deleted {deleted}; not JSON {len(not_json)}; root commits skipped {skipped_root}")
for c, path, w in failed:
    print(f"FAIL {c[:9]} {path} {w['kind']} {w['path']}: {json.dumps(w['counterexample'])[:200]}")
print("AUDIT GATE: " + ("PASS" if not failed else "FAIL"))
sys.exit(1 if failed else 0)
