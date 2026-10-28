#!/usr/bin/env bash
# B1 harvest, Fivetran family: for one connector pair, find commits in the
# SOURCE package (fivetran/dbt_<x>_source) that REMOVED declared columns
# from a src_*.yml, and check which of those columns the TRANSFORM package
# (fivetran/dbt_<x>) reads, word-boundary, with snippets. Also records
# whether the removed column carried dbt tests (enforcement semantics,
# bears on admission rule 2).
# Usage: bash harvest-fivetran.sh <work-dir> <connector>   e.g. ... github
set -euo pipefail
work="$1"; x="$2"
mkdir -p "$work/fivetran"
for r in "dbt_${x}_source" "dbt_${x}"; do
  if [ ! -d "$work/fivetran/$r/.git" ]; then
    git clone -q "https://github.com/fivetran/$r" "$work/fivetran/$r"
  fi
done
python3 - "$work/fivetran" "$x" <<'EOF'
import os, re, subprocess, sys
base, x = sys.argv[1], sys.argv[2]
src = os.path.join(base, f"dbt_{x}_source")
tx  = os.path.join(base, f"dbt_{x}")

def git(repo, *args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True).stdout

def parse_columns(text):
    # table -> {column: has_tests}. Indentation state machine over dbt
    # source yml: list entries sit two spaces deeper than their key.
    tables, cur_table, col_indent, table_indent = {}, None, None, None
    pending_col = None
    for raw in text.splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip())
        line = raw.strip()
        if line.startswith("tables:"):
            table_indent = indent; col_indent = None; cur_table = None; pending_col = None; continue
        m = re.match(r"- name:\s*['\"]?([A-Za-z0-9_]+)", line)
        if m:
            if col_indent is not None and indent >= col_indent:
                pending_col = m.group(1); tables[cur_table][pending_col] = False
            elif table_indent is not None and indent > table_indent:
                cur_table = m.group(1); tables.setdefault(cur_table, {})
                col_indent = None; pending_col = None
            continue
        if line.startswith("columns:") and cur_table is not None:
            col_indent = indent; pending_col = None; continue
        if line.startswith("tests:") and pending_col is not None and cur_table is not None:
            tables[cur_table][pending_col] = True
    return tables

ymls = [l for l in git(src, "ls-files").splitlines() if re.search(r"models/.*src_.*\.ya?ml$", l)]
rows = []
for yml in ymls:
    commits = git(src, "log", "--format=%H %cs", "--", yml).splitlines()
    for entry in commits:
        sha, date = entry.split()
        parent = git(src, "rev-parse", f"{sha}^").strip()
        if not parent or "fatal" in parent:
            continue
        before = git(src, "show", f"{parent}:{yml}")
        after = git(src, "show", f"{sha}:{yml}")
        if not before or not after:
            continue
        tb, ta = parse_columns(before), parse_columns(after)
        for table, cols in tb.items():
            removed = set(cols) - set(ta.get(table, {}))
            for col in sorted(removed):
                rows.append((sha[:12], date, yml, table, col, cols[col]))
print(f"# {x}: src ymls {len(ymls)}; column removals found: {len(rows)}", file=sys.stderr)
if not rows:
    sys.exit(0)
# One pass over the transform package for all removed columns.
fields = sorted({r[4] for r in rows})
pat = re.compile(r"(?<![A-Za-z0-9_])(" + "|".join(re.escape(f) for f in fields) + r")(?![A-Za-z0-9_])")
hits = {}
for dirpath, _, files in os.walk(tx):
    if "/.git" in dirpath or "integration_tests" in dirpath:
        continue
    for fn in files:
        if not fn.endswith((".sql", ".yml", ".yaml")):
            continue
        path = os.path.join(dirpath, fn)
        try:
            text = open(path, errors="ignore").read()
        except Exception:
            continue
        for m in pat.finditer(text):
            if len(hits.setdefault(m.group(1), [])) < 3:
                line = text[:m.start()].count("\n") + 1
                snippet = text.splitlines()[line - 1].strip()[:70]
                hits[m.group(1)].append(f"{os.path.relpath(path, tx)}:{line}: {snippet}")
print("commit\tdate\tsrc_yml\ttable\tcolumn\thad_tests\tconsumer_hits")
for sha, date, yml, table, col, tested in rows:
    h = hits.get(col, [])
    print(f"{sha}\t{date}\t{yml}\t{table}\t{col}\t{tested}\t{' ;; '.join(h) or '-'}")
EOF
