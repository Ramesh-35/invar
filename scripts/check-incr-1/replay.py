#!/usr/bin/env python3
"""CHECK-INCR-1 Gate 5: replay a repository's history with and without reuse.

For the last N first-parent commits of `pin` in <clone>, oldest first, a
small workspace holds the contract config and every input file (producer,
consumer, list) at that commit. The lock is taken once, at the first commit
where every input exists (the "lock taken long ago" case). At each commit:
  A = `invar check --no-reuse` (the reference: full evaluation)
  B = `invar check`            (reuse: lock records, and the cache B wrote
                                at the previous commit)
Required: equal exit codes, an equal tag for every contract, and equal
output once the reuse suffix, the reuse count and notes are removed.

Usage: replay.py <invar> <clone> <invar.toml> [N]   (prints one TSV row per
commit, then a summary line; exit 1 on any disagreement)"""
import os, re, shutil, statistics, subprocess, sys, tempfile, time

B, clone, toml = sys.argv[1:4]
N = int(sys.argv[4]) if len(sys.argv) > 4 else 100
TAG = re.compile(r"^  \[(HOLDS|VIOLATED|SKIPPED|NOT EVALUATED)\]\s+(\S+) \(")


def git(*a):
    return subprocess.run(["git", "-C", clone, *a], capture_output=True, text=True, check=True).stdout


def inputs(cfg):
    keys = ("producer", "consumer", "list")
    return sorted({m.group(2) for m in re.finditer(r'^(%s) = "(.*)"$' % "|".join(keys), cfg, re.M)})


def materialize(ws, commit, paths, blobs):
    listing = git("ls-tree", "-r", commit, "--", *paths)
    have = {}
    for line in listing.splitlines():
        meta, path = line.split("\t", 1)
        have[path] = meta.split()[2]
    for p in paths:
        dst = os.path.join(ws, p)
        if p not in have:
            if os.path.exists(dst):
                os.remove(dst)
            continue
        sha = have[p]
        if sha not in blobs:
            blobs[sha] = subprocess.run(["git", "-C", clone, "cat-file", "blob", sha], capture_output=True, check=True).stdout
        os.makedirs(os.path.dirname(dst) or ".", exist_ok=True)
        if not os.path.exists(dst) or open(dst, "rb").read() != blobs[sha]:
            open(dst, "wb").write(blobs[sha])


def run(ws, *args):
    t = time.perf_counter()
    o = subprocess.run([B, *args], cwd=ws, capture_output=True, text=True)
    return o.returncode, o.stdout + o.stderr, (time.perf_counter() - t) * 1e3


def tags(out):
    return {m.group(2): m.group(1) for m in map(TAG.match, out.splitlines()) if m}


def normal(out):
    lines = []
    for l in out.splitlines():
        if l.strip().startswith("note: "):
            continue
        if "[HOLDS]" in l and " unchanged since " in l:
            l = l[: l.index(" unchanged since ")]
        l = re.sub(r"^(summary: held \d+) \(\d+ reused\)", r"\1", l)
        lines.append(l)
    return "\n".join(lines)


cfg = open(toml).read()
paths = inputs(cfg)
commits = git("rev-list", "--first-parent", "-n", str(N), "pin").split()[::-1]
ws = tempfile.mkdtemp(prefix="incr1-replay-")
blobs = {}
open(os.path.join(ws, "invar.toml"), "w").write(cfg)
locked_at = None
held = 0

bad = 0
pairs = 0
reused_lock = reused_run = 0
ta, tb = [], []
print("commit\tcontracts\texit_full\texit_reuse\treused_lock\treused_last_run\tagree\tms_full\tms_reuse")
for c in commits:
    materialize(ws, c, paths, blobs)
    if locked_at is None:
        # The lock is taken at the first commit where it can be (every input
        # file exists); earlier commits run unlocked in both modes.
        lc, lout, _ = run(ws, "lock")
        if lc == 0:
            locked_at = c
            held = open(os.path.join(ws, "invar.lock")).read().count("\nheld = ")
        else:
            print(f"lock at {c[:12]} not possible (exit {lc}): {lout.strip()[:200]}", file=sys.stderr)
    ca, oa, ma = run(ws, "check", "--no-reuse")
    cb, ob, mb = run(ws, "check")
    ga, gb = tags(oa), tags(ob)
    rl = sum(1 for l in ob.splitlines() if "[HOLDS]" in l and " unchanged since lock " in l)
    rr = sum(1 for l in ob.splitlines() if "[HOLDS]" in l and " unchanged since last run" in l)
    ok = ca == cb and ga == gb and normal(oa) == normal(ob)
    if not ok:
        bad += 1
        print(f"DISAGREE at {c}:\n--- full\n{oa}\n--- reuse\n{ob}", file=sys.stderr)
    pairs += len(ga)
    reused_lock += rl
    reused_run += rr
    ta.append(ma)
    tb.append(mb)
    print(f"{c[:12]}\t{len(ga)}\t{ca}\t{cb}\t{rl}\t{rr}\t{'yes' if ok else 'NO'}\t{ma:.1f}\t{mb:.1f}")
shutil.rmtree(ws)
print(
    f"SUMMARY {os.path.basename(toml)}: {len(commits)} commits, {pairs} contract-commit pairs, "
    f"locked at commit {commits.index(locked_at) + 1 if locked_at else 'never'}, held records at lock {held}, disagreements {bad}, reused from lock {reused_lock}, "
    f"reused from last run {reused_run} ({100 * (reused_lock + reused_run) / max(pairs, 1):.1f}%), "
    f"median ms full {statistics.median(ta):.1f} reuse {statistics.median(tb):.1f}"
)
sys.exit(1 if bad else 0)
