#!/usr/bin/env bash
# JUDGE-SUBTYPE r2 regression corpus: replay `invar judge` over the 819
# SchemaStore schema changes pinned in MANIFEST.tsv. Acceptance (spec,
# Amendment B): every real commit VIOLATED; every stated commit NOT
# EVALUATED; no false_alarm commit VIOLATED; no holds commit newly
# VIOLATED. Anything else fails, listed with its witnesses.
# Whatever false alarms remain are listed with their witnesses, never
# hidden. Needs network once (clones SchemaStore into a cache dir).
# Usage, from the repo root:
#   bash docs/falsification/schemastore-replay/replay.sh [path/to/invar]
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
bin="${1:-target/release/invar}"
bin="$(cd "$(dirname "$bin")" && pwd)/$(basename "$bin")"
[ -x "$bin" ] || { echo "no invar binary at $bin (cargo build --release first)"; exit 3; }
pin=1f9ebc8379f842319663f776f9767fdf6ca5e4a5
cache="${INVAR_SS_CACHE:-${TMPDIR:-/tmp}/invar-schemastore}"
if [ ! -d "$cache/.git" ]; then
  git clone -q --shallow-since=2026-04-20 https://github.com/SchemaStore/schemastore.git "$cache" || { echo "clone failed"; exit 3; }
fi
git -C "$cache" cat-file -e "$pin^{commit}" 2>/dev/null || { echo "pinned commit $pin not in $cache"; exit 3; }
"$bin" --version
python3 - "$here/MANIFEST.tsv" "$cache" "$bin" <<'EOF'
import sys, csv, json, subprocess, collections
manifest, ss, binary = sys.argv[1:4]
rows = [r for r in csv.DictReader((l for l in open(manifest) if not l.startswith('#')), delimiter='\t')]
label = {r['commit']: r['commit_label'] for r in rows}
verdicts = collections.defaultdict(list)
detail = collections.defaultdict(list)
for c in sorted({r['commit'] for r in rows}):
    subprocess.run(['git', '-C', ss, 'checkout', '-q', '--force', c], check=True)
    for r in (x for x in rows if x['commit'] == c):
        p = subprocess.run([binary, 'judge', '--contract', r['file'], '--base', c + '^', '--format', 'json'],
                           cwd=ss, capture_output=True, text=True)
        try:
            d = json.loads(p.stdout)
        except ValueError:
            print(f"ERROR {c[:9]} {r['file']}: {p.stderr.strip()[:200]}"); sys.exit(3)
        verdicts[c].append(d['verdict'])
        for w in d.get('witnesses', []):
            detail[c].append(f"{r['file'].split('/')[-1]}: {w['kind']} {w['path']}")
violated = {c for c, v in verdicts.items() if 'VIOLATED' in v}
count = collections.Counter(label.values())
by = lambda l: [c for c, x in label.items() if x == l]
real, stated, fa, holds, und = by('real'), by('stated'), by('false_alarm'), by('holds'), by('undecided')
missed = [c for c in real if c not in violated]
stated_bad = [c for c in stated if 'NOT EVALUATED' not in verdicts[c] or c in violated]
left = [c for c in fa if c in violated]
newly = [c for c in holds if c in violated]
print(f"commits {len(label)}: real {len(real)}, stated {len(stated)}, false_alarm {len(fa)}, holds {len(holds)}, undecided {len(und)}")
print(f"real VIOLATED: {len(real) - len(missed)} of {len(real)}")
print(f"stated NOT EVALUATED: {len(stated) - len(stated_bad)} of {len(stated)}")
print(f"false alarms VIOLATED: {len(left)} of {len(fa)}")
print(f"holds newly VIOLATED: {len(newly)} of {len(holds)}")
print(f"undecided (pattern/format) flagged: {sum(1 for c in und if c in violated)} of {len(und)}")
for c in missed: print(f"  MISSED real {c[:9]}: verdicts {sorted(set(verdicts[c]))}")
for c in stated_bad: print(f"  stated not NOT EVALUATED {c[:9]}: verdicts {sorted(set(verdicts[c]))}")
for c in left: print(f"  false alarm {c[:9]}: " + "; ".join(detail[c][:3]))
for c in newly: print(f"  newly VIOLATED {c[:9]}: " + "; ".join(detail[c][:3]))
ok = not missed and not stated_bad and not left and not newly
print("REPLAY: PASS" if ok else "REPLAY: FAIL")
sys.exit(0 if ok else 1)
EOF
