#!/usr/bin/env python3
# G2-SPEC r0 measurement counter (section B): removed exported-or-not Go struct
# fields per case, whether the consumer is Go, and Go-identifier selector reads.
# Usage, from the repository root (corpus-real must be present):
#   python3 -I scripts/g2/measure.py <producer texts dir> <consumer texts dir>
import os, re, sys, glob
def comment(l): t=l.lstrip(); return t.startswith(("//","#","*","/*"))
def go_fields(text):
    out={}; depth=0; scope=""
    for n,line in enumerate(text.split("\n"),1):
        t=line.strip()
        if "struct {" in t or t.endswith("struct{"):
            if depth==0:
                h=t[5:] if t.startswith("type ") else t
                m=re.match(r"[A-Za-z_]\w*",h); scope=m.group() if m else ""
            depth+=1; continue
        if depth>0 and t.startswith("}"): depth-=1; continue
        if depth==0 or not t or t.startswith("//"): continue
        p=t.split()
        if len(p)<2 or not p[0][0].isupper() or p[1].startswith("`"): continue
        m=re.search(r'json:"([^"]*)"',t); tag=(m.group(1).split(",")[0] if m else "")
        if tag=="-": continue
        out[(scope,p[0])]=(tag or p[0],p[1],n)
    return out
def go_reads(text):
    r={}
    for n,l in enumerate(text.split("\n"),1):
        if comment(l): continue
        for m in re.finditer(r"\b[A-Za-z_]\w*(?:\([^()]*\))?\.([A-Z]\w*)\b(?!\s*\()",l):
            r.setdefault(m.group(1),[]).append(n)
    return r
pt,ct=sys.argv[1],sys.argv[2]
for c in sorted({os.path.basename(f).split(".")[0] for f in glob.glob(ct+"/*.consumer")}):
    cons=open(f"{ct}/{c}.consumer",errors="replace").read()
    inv=open(glob.glob(f"corpus-real/{c}/invar.toml")[0]).read()
    cm=re.search(r'consumer = "([^"]+)"',inv).group(1)
    for b,h,key in (("producer_base","producer","producer"),("producer_code_base","producer_code","producer_code")):
        bp,hp=f"{pt}/{c}.{b}",f"{pt}/{c}.{h}"
        if not(os.path.exists(bp) and os.path.exists(hp)): continue
        pm=re.search(key+r' = "([^"]+)"',inv).group(1)
        if not pm.endswith(".go"): continue
        bf,hf=go_fields(open(bp,errors="replace").read()),go_fields(open(hp,errors="replace").read())
        hnames={k[1] for k in hf}; hjson={v[0] for v in hf.values()}
        removed=[(k,v) for k,v in bf.items() if k[1] not in hnames]
        go_cons=cm.endswith(".go")
        reads=go_reads(cons) if go_cons else {}
        for (sc,name),(tag,ty,ln) in removed:
            hit=reads.get(name)
            print(f"{c}\t{'go-consumer' if go_cons else 'other-consumer'}\tremoved {sc}.{name} json={tag} type={ty} base:{ln}\tjson-survives={tag in hjson}\tgo-read={hit[:3] if hit else '-'}")
