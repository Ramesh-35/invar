import re,os,subprocess
STOP={"id","name","type","description","data","value","payload","key","label","title","status","error","message","result","items","url","path","version","createdat","updatedat"}
nk=lambda s:s.lower().replace("_","").replace("-","")
cases=[("earendil-works__pi","packages/coding-agent/examples/extensions/with-deps/index.ts","ExtensionAPI","packages/coding-agent/src"),
("earendil-works__pi","packages/coding-agent/src/core/compaction/branch-summarization.ts","Model","packages/ai/src"),
("earendil-works__pi","packages/coding-agent/src/core/compaction/branch-summarization.ts","SimpleStreamOptions","packages/ai/src"),
("earendil-works__pi","packages/coding-agent/src/core/compaction/branch-summarization.ts","Usage","packages/ai/src"),
("earendil-works__pi","packages/coding-agent/test/branch-summary-extensions.test.ts","Usage","packages/ai/src"),
("earendil-works__pi","packages/coding-agent/test/rpc-prompt-response-semantics.test.ts","AssistantMessage","packages/ai/src"),
("earendil-works__pi","packages/coding-agent/test/rpc-prompt-response-semantics.test.ts","Model","packages/ai/src"),
("puppeteer__puppeteer","test/installation/assets/puppeteer/configuration/puppeteer.config.ts","Configuration","packages/puppeteer-core/src"),
("puppeteer__puppeteer","test/src/browsercontext-cookies.test.ts","DeleteCookiesRequest","packages/puppeteer-core/src")]
for repo,cons,name,pkg in cases:
    root=f"e2c/{repo}"
    out=subprocess.run(["grep","-rnE",rf"export (declare )?(interface|type) {name}\b",f"{root}/{pkg}"],capture_output=True,text=True).stdout.strip().splitlines()
    out=[o for o in out if not o.endswith(".d.ts")]
    decl=out[0] if out else "NONE"
    f,ln=decl.split(":")[0],int(decl.split(":")[1]) if out else 0
    t=open(f).read().splitlines() if out else []
    body=[];depth=0;started=False
    for l in t[ln-1:]:
        body.append(l); depth+=l.count("{")-l.count("}")
        if "{" in l: started=True
        if started and depth<=0: break
        if not started and (l.rstrip().endswith(";")): break
    mem=set(re.findall(r'(?m)^\s{1,4}(?:readonly\s+)?([A-Za-z_]\w*)\??\s*[:(]',"\n".join(body[1:])))
    src=open(f"{root}/{cons}").read()
    reads=sorted(m for m in mem if re.search(r'(?<!\.\.)\.'+re.escape(m)+r'\b',src))
    dist=[r for r in reads if nk(r) not in STOP]
    print(f"{repo.split('__')[1]} {os.path.basename(cons)} {name}: decl {decl.replace(root+'/','')[:90]!r} members={len(mem)} reads={reads} distinctive={len(dist)} {'EARNED' if len(dist)<2 else 'CHECK'}")
    if not started: print("   alias head:", body[0][:160])
