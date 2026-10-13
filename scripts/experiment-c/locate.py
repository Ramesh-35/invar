"""Audit locator: workspace packages and ALL cross-package imports. No field filtering."""
import json, os, re, glob, sys
SKIP = re.compile(r"(^|/)(node_modules|dist|build|out|\.next|target|vendor|\.git|coverage)(/|$)")
SRC = (".ts",".tsx",".js",".jsx",".mjs",".cjs",".mts",".cts",".rs",".go",".py")
def read(p):
    try: return open(p, encoding="utf-8", errors="ignore").read()
    except Exception: return ""
def ws_globs(root):
    g=[]
    p=os.path.join(root,"pnpm-workspace.yaml")
    if os.path.exists(p):
        m = re.search(r"^packages\s*:\s*\n((?:\s+-.*\n?)+)", read(p), re.M)
        if m: g += re.findall(r"^\s*-\s*['\"]?([^'\"\n#]+?)['\"]?\s*$", m.group(1), re.M)
    try:
        pj=json.load(open(os.path.join(root,"package.json")))
        w=pj.get("workspaces")
        if isinstance(w,dict): w=w.get("packages",[])
        g += [x for x in (w or []) if isinstance(x,str)]
    except Exception: pass
    for f in ("lerna.json",):
        try: g += json.load(open(os.path.join(root,f))).get("packages",[])
        except Exception: pass
    cargo=read(os.path.join(root,"Cargo.toml"))
    m=re.search(r"\[workspace\][^\[]*?members\s*=\s*\[(.*?)\]", cargo, re.S)
    if m: g += re.findall(r"['\"]([^'\"]+)['\"]", m.group(1))
    gw=read(os.path.join(root,"go.work"))
    g += re.findall(r"^\s*(?:use\s+)?\(?\s*(\./[^\s)]+)", gw, re.M)
    return [x for x in g if not x.startswith("!")]
def packages(root):
    dirs=set()
    for pat in ws_globs(root):
        for d in glob.glob(os.path.join(root, pat.strip().rstrip("/"))):
            if os.path.isdir(d) and any(os.path.exists(os.path.join(d,f)) for f in ("package.json","Cargo.toml","go.mod","pyproject.toml")):
                dirs.add(os.path.normpath(d))
    pk={}
    for d in sorted(dirs):
        name=None
        try: name=json.load(open(os.path.join(d,"package.json"))).get("name")
        except Exception: pass
        if not name:
            m=re.search(r'^\s*name\s*=\s*"([^"]+)"', read(os.path.join(d,"Cargo.toml")), re.M)
            if m: name=m.group(1).replace("-","_")
        if not name:
            m=re.search(r"^module\s+(\S+)", read(os.path.join(d,"go.mod")), re.M)
            if m: name=m.group(1)
        pk[d]=name or os.path.basename(d)
    return pk
def owner(path, pk):
    best=None
    for d in pk:
        if path==d or path.startswith(d+os.sep):
            if best is None or len(d)>len(best): best=d
    return best
IMP_TS = re.compile(r"""(?:import|export)\s+(type\s+)?([\w*{}\s,$]*?)\s*from\s*['"]([^'"]+)['"]|require\(\s*['"]([^'"]+)['"]\s*\)""")
def shapes_in(pkdir):
    names={}
    for root,dirs,files in os.walk(pkdir):
        if SKIP.search(root.replace(os.sep,"/")): dirs[:]=[]; continue
        for f in files:
            if not f.endswith(SRC): continue
            t=read(os.path.join(root,f))
            for m in re.finditer(r"export\s+(?:declare\s+)?interface\s+(\w+)|export\s+type\s+(\w+)\s*(?:<[^=]*>)?\s*=\s*(?:Readonly<\s*)?\{|export\s+const\s+(\w+)\s*=\s*z\.object\(|pub\s+struct\s+(\w+)|^type\s+([A-Z]\w*)\s+struct\b|^class\s+(\w+)\s*\((?:BaseModel|TypedDict)", t, re.M):
                n=next(g for g in m.groups() if g); names[n]=os.path.relpath(os.path.join(root,f), pkdir)
    return names
def run(root):
    pk=packages(root); res={"packages":len(pk),"package_names":sorted(pk.values()),"cross_imports":[]}
    if len(pk)<2: return res
    byname={v:k for k,v in pk.items()}
    shapes={d:shapes_in(d) for d in pk}
    for d in pk:
        for r,dirs,files in os.walk(d):
            if SKIP.search(r.replace(os.sep,"/")): dirs[:]=[]; continue
            if owner(r,pk)!=d: continue
            for f in files:
                if not f.endswith(SRC): continue
                fp=os.path.join(r,f); t=read(fp)
                for m in IMP_TS.finditer(t):
                    spec=m.group(3) or m.group(4); target=None
                    if spec.startswith("."):
                        target=owner(os.path.normpath(os.path.join(r,spec)),pk)
                    else:
                        for nm,dd in byname.items():
                            if spec==nm or spec.startswith(nm+"/"): target=dd
                    if fp.endswith(".rs"): pass
                    if target and target!=d:
                        names=[x.strip().split(" as ")[0].replace("type ","") for x in re.sub(r"[{}]"," ",m.group(2) or "").split(",") if x.strip()]
                        hit=[n for n in names if n in shapes[target]]
                        res["cross_imports"].append({"consumer":os.path.relpath(fp,root),"from":pk[target],"names":names,"shapes":hit})
                if fp.endswith(".rs"):
                    for m in re.finditer(r"\buse\s+([a-z_]\w*)::([\w:{}, ]+);", t):
                        dd=byname.get(m.group(1))
                        if dd and dd!=d:
                            names=re.findall(r"\b([A-Z]\w*)\b", m.group(2)); hit=[n for n in names if n in shapes[dd]]
                            res["cross_imports"].append({"consumer":os.path.relpath(fp,root),"from":pk[dd],"names":names,"shapes":hit})
    return res
if __name__ == "__main__":
  out={}
  for line in list(open("corpus.tsv"))[1:]:
      n,repo,head=line.strip().split("\t"); d="repos/"+repo.split("github.com/")[1].replace("/","__")
      r=run(d); out[repo]=r
      si=[c for c in r["cross_imports"] if c["shapes"]]
      print(f"{n:>2} {repo.split('github.com/')[1]:<48} pkgs={r['packages']:<3} cross_imports={len(r['cross_imports']):<4} shape_imports={len(si)}")
  json.dump(out,open("locate.json","w"),indent=1)
