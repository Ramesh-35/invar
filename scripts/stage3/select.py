#!/usr/bin/env python3
"""STAGE3-EVAL-001 selection driver.

Walks the one recorded search (selection-search-raw.json) in rank order,
applies the ratified filters R-S-2 mechanically, samples every 2nd
qualifying repository until N = 10 (S3-INT-1: qualifying indices
1, 3, 5, ... in walk order), pins each selected repository's default
branch HEAD SHA, and writes repos.txt plus a walk log. Resumable: state
lives in selection-state.json; re-run until it prints DONE.

Operationalizations, recorded here because the ratified text names the
intent and the driver must name the mechanics:
  S3-INT-1 every 2nd qualifying = the 1st, 3rd, 5th ... qualifying repo
           in star-rank order.
  S3-INT-2 the 100-file floor counts .ts and .tsx files EXCLUDING .d.ts
           declaration files, which mechanically excludes type-definition
           corpora instead of excluding them by name after peeking.
  S3-INT-3 a truncated tree listing that already shows >= 100 counted
           files qualifies; one truncated below 100 is recorded as
           unverifiable and does not qualify.
  S3-INT-4 OSI set used: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause,
           ISC, MPL-2.0, GPL-2.0, GPL-3.0, LGPL-2.1, LGPL-3.0, AGPL-3.0,
           Unlicense, 0BSD (and their -only/-or-later SPDX variants).
"""
import json, os, sys, time, urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
EV = os.path.join(HERE, "..", "..", "docs", "evidence", "stage3")
RAW = os.path.join(EV, "selection-search-raw.json")
STATE = os.path.join(EV, "selection-state.json")
WALK = os.path.join(EV, "selection-walk.jsonl")
MANIFEST = os.path.join(HERE, "repos.txt")
N = 10
BUDGET_S = 140

OSI = {"MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "MPL-2.0",
       "GPL-2.0", "GPL-3.0", "LGPL-2.1", "LGPL-3.0", "AGPL-3.0", "Unlicense", "0BSD"}

def osi(spdx):
    if not spdx or spdx == "NOASSERTION":
        return False
    base = spdx.replace("-only", "").replace("-or-later", "")
    return base in OSI

def api(path):
    req = urllib.request.Request(
        "https://api.github.com/" + path,
        headers={"Accept": "application/vnd.github+json",
                 "User-Agent": "stage3-eval-001"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r)

def log(rec):
    with open(WALK, "a") as f:
        f.write(json.dumps(rec) + "\n")

def main():
    items = json.load(open(RAW))["items"]
    st = {"next_rank": 0, "qualifying": [], "pinned": []}
    if os.path.exists(STATE):
        st = json.load(open(STATE))
    t0 = time.time()
    while st["next_rank"] < len(items) and len(st["qualifying"]) < 2 * N - 1:
        if time.time() - t0 > BUDGET_S:
            json.dump(st, open(STATE, "w"))
            print("PAUSED at rank", st["next_rank"] + 1, "qualifying", len(st["qualifying"]))
            return
        i = st["next_rank"]
        r = items[i]
        st["next_rank"] = i + 1
        full = r["full_name"]
        spdx = (r.get("license") or {}).get("spdx_id")
        if not osi(spdx):
            log({"rank": i + 1, "repo": full, "verdict": "FAIL-LICENSE", "spdx": spdx})
            continue
        try:
            tree = api(f"repos/{full}/git/trees/{r['default_branch']}?recursive=1")
        except Exception as e:
            log({"rank": i + 1, "repo": full, "verdict": "FAIL-TREE-FETCH", "error": str(e)})
            json.dump(st, open(STATE, "w"))
            print("TREE FETCH FAILED at", full, e)
            return
        paths = [t["path"] for t in tree.get("tree", []) if t.get("type") == "blob"]
        ts = [p for p in paths if (p.endswith(".ts") or p.endswith(".tsx")) and not p.endswith(".d.ts")]
        truncated = bool(tree.get("truncated"))
        if len(ts) < 100:
            v = "FAIL-FILES-TRUNCATED" if truncated else "FAIL-FILES"
            log({"rank": i + 1, "repo": full, "verdict": v, "ts_files": len(ts), "truncated": truncated})
            continue
        st["qualifying"].append({"rank": i + 1, "repo": full, "url": r["clone_url"],
                                 "branch": r["default_branch"], "ts_files": len(ts),
                                 "truncated": truncated, "spdx": spdx,
                                 "stars": r["stargazers_count"]})
        log({"rank": i + 1, "repo": full, "verdict": "QUALIFY", "ts_files": len(ts),
             "truncated": truncated, "spdx": spdx})
        json.dump(st, open(STATE, "w"))
    selected = st["qualifying"][0::2][:N]
    if len(selected) < N and st["next_rank"] >= len(items):
        print("EXHAUSTED page 1 with", len(selected), "selected; a second recorded search page is needed")
        json.dump(st, open(STATE, "w"))
        return
    pinned = {p["repo"]: p for p in st["pinned"]}
    for s in selected:
        if s["repo"] in pinned:
            continue
        if time.time() - t0 > BUDGET_S:
            json.dump(st, open(STATE, "w"))
            print("PAUSED pinning after", len(st["pinned"]), "of", N)
            return
        import urllib.parse
        b = api(f"repos/{s['repo']}/branches/{urllib.parse.quote(s['branch'], safe='')}")
        s2 = dict(s)
        s2["sha"] = b["commit"]["sha"]
        st["pinned"].append(s2)
        log({"repo": s["repo"], "verdict": "PINNED", "sha": s2["sha"]})
        json.dump(st, open(STATE, "w"))
    with open(MANIFEST, "w") as f:
        f.write("# STAGE3-EVAL-001 pinned manifest, written by select.py from the one recorded search\n")
        for s in st["pinned"]:
            name = s["repo"].replace("/", "__")
            f.write(f"{name} {s['url']} {s['sha']}\n")
    json.dump(st, open(STATE, "w"))
    print("DONE", len(st["pinned"]), "repositories pinned ->", MANIFEST)

if __name__ == "__main__":
    main()
