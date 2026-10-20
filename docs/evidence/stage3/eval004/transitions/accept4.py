import sys,csv,re
toml,merged,repo,out=sys.argv[1:5]
true={r["id"] for r in csv.DictReader(open(merged),delimiter="\t") if r["repo"]==repo and r["final"]=="TRUE" and not r["n"].startswith("G")}
lines=open(toml).read().split("\n")
res=[];i=0;kept=0
blocks=[];cur=[]
# split on "# PROPOSED " headers; header region kept
head=[];j=0
while j<len(lines) and not lines[j].startswith("# PROPOSED "): head.append(lines[j]); j+=1
body=lines[j:]
blk=[]
def flush():
    global kept
    if not blk: return
    m=re.match(r"# PROPOSED (\S+)",blk[0])
    if m and m.group(1) in true:
        res.extend(blk); kept+=1
    elif not m: res.extend(blk)
for l in body:
    if l.startswith("# PROPOSED ") or l.startswith("# ABSTAIN") or l.startswith("# INVENTORY"):
        flush(); blk=[l]
    else: blk.append(l)
flush()
open(out,"w").write("\n".join(head+res))
print(repo,"kept",kept,"of",len(true)); assert kept==len(true)
