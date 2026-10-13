"""Stage 3: per-tag static extraction over the seeded sample (method addendum A1)."""
import json, os, random, re, shutil, subprocess, sys, collections
from extract import extract, classify

SEED = 20261013
TARGET = 30
TAGS_WINDOW = 10
SERVER_PAT = re.compile(r"inputSchema|input_schema|mcp-ts-core|\btool\(\s*['\"]|registerTool\(|\.tool\(|@mcp\.tool|@server\.tool|ListToolsRequestSchema|FastMCP\(|McpServer\(|mcp\.NewServer|server\.AddTool|setRequestHandler\(\s*CallToolRequestSchema")

def git(d, *a, timeout=300):
    r = subprocess.run(["git", "-C", d, *a], capture_output=True, text=True, timeout=timeout,
                       env={**os.environ, "GIT_TERMINAL_PROMPT": "0"})
    return r.returncode, r.stdout, r.stderr

def run_one(url, paths):
    name = url.split("github.com/")[1].replace("/", "__")
    d = os.path.join("s3", name)
    shutil.rmtree(d, ignore_errors=True)
    r = subprocess.run(["git", "clone", "-q", "--filter=blob:none", "--no-checkout", url, d],
                       capture_output=True, text=True, timeout=600, env={**os.environ, "GIT_TERMINAL_PROMPT": "0"})
    if r.returncode != 0:
        return {"url": url, "result": "clone-error", "err": r.stderr.strip()[:160]}, False
    _, out, _ = git(d, "for-each-ref", "--sort=creatordate", "--format=%(refname:short)", "refs/tags")
    tags = [t for t in out.split() if t][-TAGS_WINDOW:]
    if len(tags) < 3:
        return {"url": url, "result": "fewer-than-3-tags-reachable", "tags": len(tags)}, False
    src = [p for p in paths if not p.endswith((".d.ts",))]
    per_tag = []
    server = False
    unresolved = collections.Counter()
    for t in tags:
        tools = {}
        for p in src:
            rc, text, _ = git(d, "show", f"{t}:{p}")
            if rc != 0:
                continue
            if SERVER_PAT.search(text) or "/tools/" in "/" + p:
                server = True
            for tool, sch in extract(p, text).items():
                key = tool if tool not in tools else f"{p}::{tool}"
                tools[key] = sch
                if isinstance(sch, str):
                    unresolved[sch] += 1
        per_tag.append(tools)
    resolved_presence = collections.Counter()
    for tools in per_tag:
        for k, v in tools.items():
            if isinstance(v, dict):
                resolved_presence[k] += 1
    qualifying_tools = [k for k, n in resolved_presence.items() if n >= 2]
    rec = {"url": url, "tags": tags, "server": server,
           "tools_resolved_any": len(resolved_presence), "qualifying_tools": len(qualifying_tools),
           "unresolved": dict(unresolved)}
    if not server:
        rec["result"] = "not-a-server"
        return rec, False
    if not qualifying_tools:
        rec["result"] = "no-statically-extractable-schema-in-2-tags"
        return rec, False
    flags, pairs = [], 0
    for i in range(1, len(per_tag)):
        a, b = per_tag[i - 1], per_tag[i]
        for k in qualifying_tools:
            if isinstance(a.get(k), dict) and isinstance(b.get(k), dict):
                pairs += 1
                for br in classify(a[k], b[k]):
                    flags.append({"from": tags[i - 1], "to": tags[i], "tool": k, "break": br})
    rec.update({"result": "QUALIFYING", "pairs_compared": pairs, "flags": flags})
    return rec, bool(flags)

def main():
    hits = []
    for l in open("stage2.jsonl"):
        r = json.loads(l)
        if r.get("files"):
            hits.append((r["url"], [f for f, _ in r["files"]]))
    hits.sort()
    random.Random(SEED).shuffle(hits)
    os.makedirs("s3", exist_ok=True)
    done = {}
    if os.path.exists("stage3.jsonl"):
        for l in open("stage3.jsonl"):
            r = json.loads(l); done[r["url"]] = r
    q = sum(1 for r in done.values() if r.get("result") == "QUALIFYING")
    with open("stage3.jsonl", "a") as out:
        for url, paths in hits:
            if q >= TARGET:
                break
            if url in done:
                continue
            try:
                rec, keep = run_one(url, paths)
            except subprocess.TimeoutExpired:
                rec, keep = {"url": url, "result": "timeout"}, False
            if not keep:
                shutil.rmtree(os.path.join("s3", url.split("github.com/")[1].replace("/", "__")), ignore_errors=True)
            out.write(json.dumps(rec) + "\n"); out.flush()
            if rec.get("result") == "QUALIFYING":
                q += 1
            print(len(done) + 1, rec.get("result"), url, "qualifying:", q, "flags:", len(rec.get("flags", [])), flush=True)
            done[url] = rec
    print("STAGE3 DONE qualifying", q, "examined", len(done), flush=True)

if __name__ == "__main__":
    main()
