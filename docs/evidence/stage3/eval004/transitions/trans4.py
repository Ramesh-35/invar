import re,sys
def load(p):
    t=open(p).read().splitlines()
    ab=[l for l in t if l.startswith("# ABSTAIN ")]
    cons=set(re.findall(r'(?m)^(?:consumer|producer) = "([^"]+)"',"\n".join(t)))
    e5=set(re.findall(r'\((\S+?):\d+\)',"\n".join(l for l in t if l.startswith("# E5 ") or l.startswith("# E4 "))))
    return ab,cons|e5
for r in sys.argv[1:]:
    oa,_=load(f"eval003/{r}.invar.toml"); na,ncov=load(f"eval004/{r}.invar.toml")
    removed=[l for l in oa if l not in set(na)]; added=[l for l in na if l not in set(oa)]
    def key(l):
        m=re.search(r'(\S+\.tsx?):(\d+)',l); return (m.group(1),m.group(2)) if m else None
    nkeys={key(l) for l in na}
    silent=[l for l in removed if key(l) and key(l) not in nkeys and key(l)[0] not in ncov]
    code=lambda l:l.split()[2]
    from collections import Counter
    print(r,"removed",len(removed),Counter(map(code,removed)),"added",len(added),Counter(map(code,added)),"to-silence",len(silent))
    for l in silent: print("   SILENT<-",l[:230])
