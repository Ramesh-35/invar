import json, os, random, re, shutil, subprocess, time
BIN = os.path.abspath("../invar-f1/target/release/invar")
repos = [l.strip() for l in open("frame.txt") if l.strip()]
repos.sort(); random.Random(20261014).shuffle(repos)
def is_monorepo(d):
    j = lambda p: os.path.join(d, p)
    if any(os.path.exists(j(f)) for f in ("pnpm-workspace.yaml","lerna.json","nx.json","turbo.json","go.work","rush.json")): return True
    try:
        if "workspaces" in json.load(open(j("package.json"))): return True
    except Exception: pass
    try:
        if re.search(r"^\[workspace\]", open(j("Cargo.toml")).read(), re.M): return True
    except Exception: pass
    return False
done = set()
if os.path.exists("b1.jsonl"): done = {json.loads(l)["repo"] for l in open("b1.jsonl")}
mono = sum(1 for l in open("b1.jsonl") if json.loads(l).get("monorepo")) if os.path.exists("b1.jsonl") else 0
os.makedirs("keep", exist_ok=True)
with open("b1.jsonl","a") as out:
    for r in repos:
        if mono >= 30: break
        if r in done: continue
        name = r.split("github.com/")[1].replace("/","__"); d = os.path.join("work", name)
        shutil.rmtree(d, ignore_errors=True)
        rec = {"repo": r}
        try:
            c = subprocess.run(["git","clone","-q","--depth","1",r,d],capture_output=True,text=True,timeout=180,env={**os.environ,"GIT_TERMINAL_PROMPT":"0"})
        except subprocess.TimeoutExpired:
            rec["result"]="clone-timeout"; out.write(json.dumps(rec)+"\n"); out.flush(); shutil.rmtree(d,ignore_errors=True); continue
        if c.returncode != 0:
            rec["result"]="clone-error"; out.write(json.dumps(rec)+"\n"); out.flush(); continue
        rec["monorepo"] = is_monorepo(d)
        if not rec["monorepo"]:
            rec["result"]="not-monorepo"; out.write(json.dumps(rec)+"\n"); out.flush(); shutil.rmtree(d,ignore_errors=True); continue
        mono += 1
        t0 = time.time()
        try:
            p = subprocess.run([BIN,"init"],cwd=d,capture_output=True,text=True,timeout=300)
            rec["init_seconds"] = round(time.time()-t0, 3); rec["init_exit"] = p.returncode
            m = re.search(r"contracts proposed\s*:\s*(\d+)", p.stdout); rec["proposed"] = int(m.group(1)) if m else None
            m = re.search(r"scanned (\d+) file", p.stdout); rec["files_scanned"] = int(m.group(1)) if m else None
            rec["init_stdout"] = p.stdout[-1500:]; rec["init_stderr"] = p.stderr[-500:]
        except subprocess.TimeoutExpired:
            rec["init_seconds"] = 300; rec["result"] = "init-timeout"
        rec.setdefault("result", "MONOREPO")
        if rec.get("proposed"):
            shutil.move(d, os.path.join("keep", name))
        else:
            shutil.rmtree(d, ignore_errors=True)
        out.write(json.dumps(rec)+"\n"); out.flush()
        print(mono, r, "proposed:", rec.get("proposed"), "init_s:", rec.get("init_seconds"), flush=True)
print("B1 DONE monorepos", mono, flush=True)
