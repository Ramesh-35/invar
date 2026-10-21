#!/usr/bin/env bash
# STAGE3-EVAL-008 M-S-5 (round trip) and M-S-6 (backtest) over the new
# TRUE contracts. For each repository: a depth-200 clone of the pinned
# commit, an invar.toml holding exactly the accepted blocks, then
# lock, check twice (byte-identical), backtest twice (byte-identical).
# Usage: roundtrip.sh <invar binary> <clones dir> <eval007 dir> <accepted ids tsv> <out dir>
# accepted ids tsv: repo <TAB> id, one line per accepted contract.
set -u
BIN=$1; CLONES=$2; EV=$3; ACC=$4; OUT=$5
mkdir -p "$OUT"
for repo in $(cut -f1 "$ACC" | sort -u); do
  w=$(mktemp -d)/$repo
  git clone -q --depth 200 "file://$CLONES/$repo" "$w"
  python3 - "$EV/$repo.invar.toml" "$ACC" "$repo" > "$w/invar.toml" <<'EOF'
import sys
src, acc, repo = sys.argv[1:4]
ids = {l.split("\t")[1].strip() for l in open(acc) if l.split("\t")[0] == repo}
blocks, cur, keep = [], [], None
for l in open(src).read().splitlines():
    if l.startswith("# PROPOSED "):
        cur, keep = [l], l[len("# PROPOSED "):] in ids
    elif cur:
        if l == "":
            if keep:
                blocks.append("\n".join(cur))
            cur = []
        else:
            cur.append(l)
print("\n\n".join(blocks))
EOF
  cp "$w/invar.toml" "$OUT/$repo.accepted.invar.toml"
  (cd "$w" && "$BIN" lock > "$OUT/$repo.lock.out" 2>&1; echo "exit $?" >> "$OUT/$repo.lock.out")
  cp "$w/invar.lock" "$OUT/$repo.invar.lock"
  for i in 1 2; do
    (cd "$w" && "$BIN" check > "$OUT/$repo.check$i.out" 2>&1; echo "exit $?" >> "$OUT/$repo.check$i.out")
    (cd "$w" && "$BIN" backtest > "$OUT/$repo.backtest$i.out" 2>&1; echo "exit $?" >> "$OUT/$repo.backtest$i.out")
  done
  rm -rf "$(dirname "$w")"
done
