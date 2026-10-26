#!/usr/bin/env python3
"""SECURITY S0 probe, path 3: does the Authorization header INVAR hands curl
(on stdin, `-H @-`) follow a redirect to another host? Two local HTTP
servers: A (127.0.0.1) redirects to B (localhost, a different host name).
curl runs with INVAR's own flags (resolver::curl_args_max) except the
https-only `--proto`/`--proto-redir` pair, removed so plain local HTTP is
possible; in production those flags additionally forbid any non-https hop.
Reports whether B saw the token. Usage: python3 -I scripts/security/token-redirect.py"""
import http.server, subprocess, threading
seen = {}
class A(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        seen["A"] = self.headers.get("Authorization")
        self.send_response(302); self.send_header("Location", f"http://localhost:{pb}/x"); self.end_headers()
    def log_message(self, *a): pass
class Bh(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        seen["B"] = self.headers.get("Authorization")
        self.send_response(200); self.end_headers(); self.wfile.write(b"ok")
    def log_message(self, *a): pass
sa = http.server.HTTPServer(("127.0.0.1", 0), A); sb = http.server.HTTPServer(("127.0.0.1", 0), Bh)
pa, pb = sa.server_address[1], sb.server_address[1]
for s in (sa, sb): threading.Thread(target=s.serve_forever, daemon=True).start()
args = ["curl", "--silent", "--show-error", "--fail", "--location", "--max-time", "10", "--max-filesize", "1000000", "-H", "@-", f"http://127.0.0.1:{pa}/x"]
r = subprocess.run(args, input=b"Authorization: Bearer SECRET-TOKEN\n", capture_output=True)
v = subprocess.run(["curl", "--version"], capture_output=True, text=True).stdout.split("\n")[0]
print(f"{v}\ncurl exit {r.returncode}; host A saw token: {seen.get('A') is not None}; redirect host B saw token: {seen.get('B') is not None}")
