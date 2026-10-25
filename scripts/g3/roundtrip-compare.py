#!/usr/bin/env python3
"""G3: EVAL-007/008 round trips on two binaries (baseline vs candidate).

For each accepted round-trip config (docs/evidence/stage3/<eval>/m-s-5/
<repo>.accepted.invar.toml) and its clone (checked out at `pin`), each
binary runs: lock, check twice, backtest twice, each in its own fresh copy
of the checkout. The outputs and the written invar.lock are compared after
normalizing ONLY what differs between any two runs by design:
  - RFC 3339 times (lock records and "unchanged since lock <time>")
  - the engine identity token in `held` lines (CHECK-INCR-1 I-D5: any
    source change yields a new engine id, so two binaries never share one)
Everything else must be byte-identical. Usage:
  roundtrip-compare.py <baseline invar> <candidate invar> <clones dir> <out dir>"""
import os, re, shutil, subprocess, sys, tempfile

BASE, CAND, CLONES, OUT = sys.argv[1:5]
root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TIME = re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")
ENGINE = re.compile(r"\d+\.\d+\.\d+-[a-z0-9.]+-[0-9a-f]{64}")


def norm(s):
    return ENGINE.sub("ENGINE", TIME.sub("TIME", s))


def run(binary, ws, args):
    o = subprocess.run([binary, *args], cwd=ws, capture_output=True, text=True)
    return o.stdout + o.stderr + f"exit {o.returncode}\n"


def trip(binary, checkout, cfg):
    ws = tempfile.mkdtemp(prefix="g3rt-")
    shutil.rmtree(ws)
    shutil.copytree(checkout, ws, symlinks=True, ignore=shutil.ignore_patterns(".invar"))
    shutil.copy(cfg, os.path.join(ws, "invar.toml"))
    for f in ("invar.lock",):
        p = os.path.join(ws, f)
        if os.path.exists(p):
            os.remove(p)
    res = {"lock.out": run(binary, ws, ["lock"])}
    lp = os.path.join(ws, "invar.lock")
    res["invar.lock"] = open(lp).read() if os.path.exists(lp) else "(none)\n"
    for i in (1, 2):
        res[f"check{i}.out"] = run(binary, ws, ["check"])
        res[f"backtest{i}.out"] = run(binary, ws, ["backtest"])
    shutil.rmtree(ws)
    return res


os.makedirs(OUT, exist_ok=True)
bad = 0
total = 0
for ev in ("eval007", "eval008"):
    d = os.path.join(root, "docs/evidence/stage3", ev, "m-s-5")
    for f in sorted(os.listdir(d)):
        if not f.endswith(".accepted.invar.toml"):
            continue
        repo = f[: -len(".accepted.invar.toml")]
        clone = os.path.join(CLONES, repo)
        checkout = tempfile.mkdtemp(prefix=f"g3co-{repo}-")
        shutil.rmtree(checkout)
        subprocess.run(["git", "-C", clone, "worktree", "add", "-f", "--detach", checkout, "pin"], check=True, capture_output=True)
        a = trip(BASE, checkout, os.path.join(d, f))
        b = trip(CAND, checkout, os.path.join(d, f))
        subprocess.run(["git", "-C", clone, "worktree", "remove", "--force", checkout], capture_output=True)
        for k in a:
            total += 1
            same = norm(a[k]) == norm(b[k])
            raw_same = a[k] == b[k]
            if not same:
                bad += 1
                open(os.path.join(OUT, f"{ev}.{repo}.{k}.baseline"), "w").write(a[k])
                open(os.path.join(OUT, f"{ev}.{repo}.{k}.candidate"), "w").write(b[k])
            print(f"{ev}\t{repo}\t{k}\t{'IDENTICAL' if raw_same else ('IDENTICAL-AFTER-NORMALIZATION' if same else 'DIFFERENT')}\t{a[k].splitlines()[-1]}")
print(f"SUMMARY: {total} artifacts compared, {bad} different after normalization")
sys.exit(1 if bad else 0)
