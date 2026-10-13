# Stage 1: tag count via git ls-remote (no clone). Parallel, resumable.
import json, subprocess, sys, concurrent.futures as cf, os, re
repos = [l.strip() for l in open("repos.txt") if l.strip()]
done = {}
if os.path.exists("stage1.jsonl"):
    for l in open("stage1.jsonl"):
        r = json.loads(l); done[r["repo"]] = r
def norm(url):
    m = re.match(r"https?://github\.com/([^/]+)/([^/#?]+)", url)
    return f"https://github.com/{m.group(1)}/{m.group(2)}" if m else None
def tags(repo):
    u = norm(repo)
    if not u: return {"repo": repo, "tags": None, "err": "unparseable"}
    try:
        r = subprocess.run(["git","ls-remote","--tags","--refs",u], capture_output=True, text=True, timeout=60,
                           env={**os.environ, "GIT_TERMINAL_PROMPT":"0"})
        if r.returncode != 0: return {"repo": repo, "tags": None, "err": r.stderr.strip()[:120]}
        t = [l.split("refs/tags/",1)[1] for l in r.stdout.splitlines() if "refs/tags/" in l]
        return {"repo": repo, "url": u, "tags": len(t)}
    except subprocess.TimeoutExpired:
        return {"repo": repo, "tags": None, "err": "timeout"}
todo = [r for r in repos if r not in done]
with open("stage1.jsonl","a") as out, cf.ThreadPoolExecutor(16) as ex:
    for i, res in enumerate(ex.map(tags, todo)):
        out.write(json.dumps(res)+"\n"); out.flush()
        if i % 200 == 0: print("stage1", i, "/", len(todo), flush=True)
print("STAGE1 DONE", flush=True)
