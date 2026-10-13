import json, subprocess, urllib.parse, time, sys
base = "https://registry.modelcontextprotocol.io/v0/servers?limit=100&version=latest"
cursor=None; pages=0; fails=0
out=open("registry.jsonl","w")
while True:
    url = base + (f"&cursor={urllib.parse.quote(cursor)}" if cursor else "")
    for attempt in range(5):
        r = subprocess.run(["curl","-s","--max-time","60","-w","\n%{http_code}",url],capture_output=True,text=True)
        body, _, code = r.stdout.rpartition("\n")
        if code=="200": break
        fails+=1; time.sleep(3*(attempt+1))
    else:
        print("GAVE UP page",pages,"cursor",cursor,flush=True); break
    d=json.loads(body); pages+=1
    for e in d.get("servers",[]):
        s=e.get("server",{}); m=(e.get("_meta") or {}).get("io.modelcontextprotocol.registry/official") or {}
        out.write(json.dumps({"name":s.get("name"),"version":s.get("version"),"repo":(s.get("repository") or {}).get("url") or "","latest":m.get("isLatest"),"status":m.get("status")})+"\n")
    out.flush()
    if pages%20==0: print("page",pages,"retries",fails,flush=True)
    cursor=(d.get("metadata") or {}).get("nextCursor")
    if not cursor: print("DONE pages",pages,"retries",fails,flush=True); break
