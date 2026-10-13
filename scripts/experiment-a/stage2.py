# Stage 2: depth-1 clone of default branch; does source declare outputSchema statically?
import json, subprocess, os, concurrent.futures as cf, re, shutil
rows = [json.loads(l) for l in open("stage1.jsonl")]
cand = {}
for r in rows:
    if r.get("tags") and r["tags"] >= 3: cand[r["url"]] = r
done = set()
if os.path.exists("stage2.jsonl"):
    done = {json.loads(l)["url"] for l in open("stage2.jsonl")}
EXCL = re.compile(r"(^|/)(node_modules|dist|build|vendor|\.git|test|tests|__tests__|examples?)(/|$)")
SRC = (".ts",".tsx",".js",".mjs",".py",".go",".rs",".java",".cs",".kt",".json")
def check(url):
    d = "s2/" + url.split("github.com/")[1].replace("/","__")
    try:
        r = subprocess.run(["git","clone","-q","--depth","1",url,d],capture_output=True,text=True,timeout=120,
                           env={**os.environ,"GIT_TERMINAL_PROMPT":"0"})
        if r.returncode != 0: return {"url":url,"err":r.stderr.strip()[:120]}
        hits=[]
        for root,dirs,files in os.walk(d):
            rel = os.path.relpath(root,d)
            if EXCL.search(rel.replace(os.sep,"/")): dirs[:]=[]; continue
            for f in files:
                if not f.endswith(SRC): continue
                p=os.path.join(root,f)
                try:
                    if os.path.getsize(p) > 2_000_000: continue
                    t=open(p,encoding="utf-8",errors="ignore").read()
                except Exception: continue
                n = len(re.findall(r"outputSchema|output_schema", t))
                if n: hits.append((os.path.relpath(p,d), n))
        return {"url":url,"files":hits}
    except subprocess.TimeoutExpired:
        return {"url":url,"err":"timeout"}
    finally:
        shutil.rmtree(d, ignore_errors=True)
todo=[u for u in cand if u not in done]
os.makedirs("s2",exist_ok=True)
with open("stage2.jsonl","a") as out, cf.ThreadPoolExecutor(12) as ex:
    for i,res in enumerate(ex.map(check,todo)):
        out.write(json.dumps(res)+"\n"); out.flush()
        if i%100==0: print("stage2",i,"/",len(todo),flush=True)
print("STAGE2 DONE",flush=True)
