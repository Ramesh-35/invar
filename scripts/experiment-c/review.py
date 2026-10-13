"""Print, per shape-import candidate, the shape's declared fields and the
consumer's property accesses on those field names, for HUMAN review."""
import json, re, sys, os
sys.path.insert(0, "."); from locate import packages, shapes_in, read
STOP = {"id","name","type","description","data","value","payload","key","label","title","status","error","message","result","items","url","path","version","createdat","updatedat"}
nk = lambda s: s.lower().replace("_","").replace("-","")
def shape_fields(text, name):
    m = re.search(rf"(?:interface\s+{name}\b[^{{]*|type\s+{name}\b[^=]*=\s*(?:Readonly<\s*)?|const\s+{name}\s*=\s*z\.object\(\s*|pub\s+struct\s+{name}\b[^{{]*|type\s+{name}\s+struct\s*)\{{", text)
    if not m: return None
    i, depth = m.end()-1, 0
    for j in range(i, len(text)):
        if text[j] == "{": depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0: body = text[i+1:j]; break
    else: return None
    # top-level keys only
    out, d, cur = [], 0, ""
    for ch in body:
        if ch in "{([<": d += 1
        elif ch in "})]>": d -= 1
        if ch in ",;\n" and d == 0: out.append(cur); cur = ""
        else: cur += ch
    out.append(cur)
    keys = []
    for e in out:
        e = re.sub(r"//.*|/\*.*?\*/", "", e).strip()
        mm = re.match(r"(?:pub\s+)?(?:readonly\s+)?['\"]?([A-Za-z_]\w*)['\"]?\??\s*[:]", e) or re.match(r"([A-Z]\w*)\s+\S", e)
        if mm: keys.append(mm.group(1))
    return keys
repo = sys.argv[1]; limit = int(sys.argv[2]) if len(sys.argv) > 2 else 6
loc = json.load(open("locate.json"))[repo]
d = "repos/" + repo.split("github.com/")[1].replace("/", "__")
pk = packages(d); byname = {v: k for k, v in pk.items()}
seen = 0
for c in loc["cross_imports"]:
    for s in c["shapes"]:
        pdir = byname[c["from"]]; sh = shapes_in(pdir); src = read(os.path.join(pdir, sh[s]))
        f = shape_fields(src, s) or []
        distinct = [k for k in f if nk(k) not in STOP]
        ctext = read(os.path.join(d, c["consumer"]))
        reads = sorted({k for k in distinct if re.search(rf"\.{re.escape(k)}\b|\[['\"]{re.escape(k)}['\"]\]|\b{re.escape(k)}\s*[,}}]\s*(?:=|\}})", ctext)})
        test = bool(re.search(r"(test|spec)\.|/tests?/|__tests__", c["consumer"]))
        print(f"- {s} ({c['from']}:{sh[s]}) -> {c['consumer']}{' [test]' if test else ''}")
        print(f"    declared distinctive fields ({len(distinct)}): {distinct[:12]}")
        print(f"    consumer mentions (candidate reads, VERIFY BY HAND): {reads[:12]}")
        seen += 1
        if seen >= limit: sys.exit()
