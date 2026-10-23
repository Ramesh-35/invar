#!/usr/bin/env python3
"""CHECK-PERF-1 latency bench, self-contained: builds a synthetic workspace
(4 producer files shared by 18 contracts, 18 consumer files), locks it,
then times `invar check` as a fresh process and the `check` tool of a
running `invar mcp`. Prints medians in milliseconds. Needs only Python 3
and an invar binary.

Usage, from the repository root:
    python3 scripts/perf/bench.py target/release/invar
    python3 scripts/perf/bench.py target/release/invar --dir path/with/invar.toml
"""
import json, os, statistics, subprocess, sys, tempfile, time

def workspace(root):
    os.makedirs(f"{root}/lib", exist_ok=True)
    os.makedirs(f"{root}/src", exist_ok=True)
    blocks = []
    for p in range(4):
        shapes = []
        for s in range(30):
            fields = "".join(f"  f{p}_{s}_{k}: number;\n" for k in range(12))
            shapes.append(f"export interface S{p}_{s} {{\n{fields}}}\n")
        open(f"{root}/lib/p{p}.ts", "w").write("".join(shapes))
    for c in range(18):
        p, s = c % 4, c
        reads = "".join(f"  use(o.f{p}_{s}_{k});\n" for k in range(6))
        filler = "".join(f"const v{i} = m.get('x{i}').y{i};\n" for i in range(800))
        open(f"{root}/src/c{c}.ts", "w").write(f"export function f(o: any) {{\n{reads}}}\n{filler}")
        blocks.append(f'[[invariant]]\nkind = "schema"\nid = "code:c{c}"\nproducer = "lib/p{p}.ts"\nconsumer = "src/c{c}.ts"\nevidence = "S{p}_{s} ' + " ".join(f"f{p}_{s}_{k}" for k in range(6)) + '"\n')
    open(f"{root}/invar.toml", "w").write("\n".join(blocks))

def cli(binary, d, n=20):
    ts = []
    for _ in range(n):
        t = time.perf_counter()
        subprocess.run([binary, "check"], cwd=d, capture_output=True)
        ts.append((time.perf_counter() - t) * 1000)
    return statistics.median(ts)

def mcp(binary, d, n=30):
    p = subprocess.Popen([binary, "mcp"], cwd=d, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
    def call(i, method, params):
        p.stdin.write(json.dumps({"jsonrpc": "2.0", "id": i, "method": method, "params": params}) + "\n")
        p.stdin.flush()
        t = time.perf_counter()
        p.stdout.readline()
        return (time.perf_counter() - t) * 1000
    call(1, "initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "bench", "version": "0"}})
    first = call(2, "tools/call", {"name": "check", "arguments": {}})
    warm = [call(3 + i, "tools/call", {"name": "check", "arguments": {}}) for i in range(n)]
    p.stdin.close()
    p.wait()
    return first, statistics.median(warm)

def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    binary = os.path.abspath(sys.argv[1])
    if "--dir" in sys.argv:
        d = os.path.abspath(sys.argv[sys.argv.index("--dir") + 1])
    else:
        d = tempfile.mkdtemp(prefix="invar-bench-")
        workspace(d)
        subprocess.run([binary, "lock"], cwd=d, capture_output=True, check=True)
    code = subprocess.run([binary, "check"], cwd=d, capture_output=True).returncode
    first, warm = mcp(binary, d)
    print(f"workspace: {d} (check exit {code})")
    print(f"invar check, fresh process:      {cli(binary, d):7.2f} ms (median of 20)")
    print(f"invar mcp check, first call:     {first:7.2f} ms")
    print(f"invar mcp check, running server: {warm:7.2f} ms (median of 30)")

main()
