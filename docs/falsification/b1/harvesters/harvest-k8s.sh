#!/usr/bin/env bash
# B1 harvest, Kubernetes family: the declared contract is k8s.io/api's
# generated prerelease-lifecycle ledger (APILifecycleRemoved per Kind).
# Parse it at three producer checkouts (pre-removal eras plus HEAD), then
# sweep independent consumer repositories for (a) reads of removed
# group/versions and (b) the coping notations written to absorb them:
# Helm Capabilities branching, dual-version Go imports, runtime discovery.
# Usage: bash harvest-k8s.sh <work-dir> [clone|ledger|sweep|all]
set -euo pipefail
work="$1"; phase="${2:-all}"
mkdir -p "$work"

API_TAGS=(kubernetes-1.21.0 kubernetes-1.24.0 master)
CONSUMERS=(
  https://github.com/cert-manager/cert-manager
  https://github.com/prometheus-operator/prometheus-operator
  https://github.com/kubernetes-sigs/external-dns
  https://github.com/kubernetes/ingress-nginx
)
# Historical consumer checkouts: eras when dual-version handling was live.
HIST=(
  "cert-manager|v1.5.3"
  "external-dns|v0.10.2"
)

clone_phase() {
  for t in "${API_TAGS[@]}"; do
    d="$work/k8s-api-$t"
    if [ ! -d "$d/.git" ]; then
      git clone -q --depth 1 --branch "$t" https://github.com/kubernetes/api "$d" \
        || echo "WARN: could not clone api@$t" >&2
    fi
  done
  for c in "${CONSUMERS[@]}"; do
    name="$(basename "$c")"
    if [ ! -d "$work/$name/.git" ]; then
      git clone -q --depth 1 "$c" "$work/$name" || echo "WARN: could not clone $c" >&2
    fi
  done
  for h in "${HIST[@]}"; do
    name="${h%%|*}"; tag="${h##*|}"
    d="$work/$name-$tag"
    if [ ! -d "$d/.git" ]; then
      url=""
      for c in "${CONSUMERS[@]}"; do case "$c" in */$name) url="$c";; esac; done
      git clone -q --depth 1 --branch "$tag" "$url" "$d" \
        || echo "WARN: could not clone $name@$tag" >&2
    fi
  done
  echo "clone phase done" >&2
}

ledger_phase() {
python3 - "$work" <<'EOF'
import os, re, sys
work = sys.argv[1]
rows = {}
for d in sorted(os.listdir(work)):
    if not d.startswith("k8s-api-"):
        continue
    root = os.path.join(work, d)
    for dirpath, _, files in os.walk(root):
        if "zz_generated.prerelease-lifecycle.go" not in files:
            continue
        rel = os.path.relpath(dirpath, root)          # e.g. networking/v1beta1
        parts = rel.split(os.sep)
        if len(parts) != 2:
            continue
        gdir, version = parts
        group = gdir
        reg = os.path.join(dirpath, "register.go")
        if os.path.exists(reg):
            m = re.search(r'GroupName\s*=\s*"([^"]*)"', open(reg, errors="ignore").read())
            if m:
                group = m.group(1)
        text = open(os.path.join(dirpath, "zz_generated.prerelease-lifecycle.go"),
                    errors="ignore").read()
        dep = dict(re.findall(
            r"func \(in \*(\w+)\) APILifecycleDeprecated\(\)[^{]*\{\s*return (\d+, \d+)", text))
        for kind, rel_ver in re.findall(
                r"func \(in \*(\w+)\) APILifecycleRemoved\(\)[^{]*\{\s*return (\d+, \d+)", text):
            if kind.endswith("List"):
                continue
            removed = rel_ver.replace(", ", ".")
            deprecated = dep.get(kind, "").replace(", ", ".")
            key = (group, version, kind)
            if key not in rows:
                rows[key] = (deprecated, removed, d)
print("group\tversion\tkind\tdeprecated\tremoved\tseen_in")
for (group, version, kind), (dep, rem, src) in sorted(rows.items()):
    print(f"{group}\t{version}\t{kind}\t{dep}\t{rem}\t{src}")
print(f"# declared removals: {len(rows)} kinds", file=sys.stderr)
EOF
}

sweep_phase() {
python3 - "$work" <<'EOF'
import os, re, sys
work = sys.argv[1]
# Rebuild the group/version set from the ledger output if present, else
# from the api checkouts directly (same parse, versions only).
gvs = set()
led = os.path.join(work, "k8s-ledger.tsv")
if os.path.exists(led):
    for line in open(led):
        if line.startswith(("group\t", "#")):
            continue
        g, v = line.split("\t")[:2]
        gvs.add((g, v))
if not gvs:
    sys.exit("run ledger phase first and save to k8s-ledger.tsv")
# Two match forms per group/version: manifest apiVersion and Go import path.
forms = {}
for g, v in gvs:
    gdir = g.split(".")[0] if g else g
    forms[f"{g}/{v}" if g else v] = (g, v, "apiVersion")
    forms[f"k8s.io/api/{gdir}/{v}"] = (g, v, "go-import")
pat = re.compile("|".join(re.escape(k) for k in sorted(forms, key=len, reverse=True)))
coping = {
    "helm-capabilities": re.compile(r"\.Capabilities\.APIVersions\.Has|semverCompare.*[Kk]ubeVersion|\.Capabilities\.KubeVersion"),
    "runtime-discovery": re.compile(r"ServerPreferredResources|ServerResourcesForGroupVersion|discovery\.NewDiscoveryClient"),
}
exts = (".go", ".yaml", ".yml", ".tpl", ".json")
skip = ("/.git", "/vendor/", "node_modules", "/testdata/", "/test/", "_test.go",
        "/docs/", "/examples/", "/CHANGELOG")
consumers = [d for d in sorted(os.listdir(work))
             if not d.startswith(("k8s-api-", "k8s-ledger"))
             and os.path.isdir(os.path.join(work, d, ".git"))
             and d not in ("semantic-conventions", "iglu-central", "fivetran",
                           "jaeger", "signoz")
             and not d.startswith("dbt-snowplow")]
print(f"# consumers swept: {', '.join(consumers)}", file=sys.stderr)
print("consumer\tgroup_version\tmatch_form\tfile\tline\tsnippet")
cope_rows = []
for d in consumers:
    root = os.path.join(work, d)
    for dirpath, _, files in os.walk(root):
        p = dirpath + "/"
        if any(s in p for s in skip):
            continue
        for fn in files:
            if not fn.endswith(exts) or fn.endswith("_test.go"):
                continue
            path = os.path.join(dirpath, fn)
            try:
                text = open(path, errors="ignore").read()
            except Exception:
                continue
            rel = os.path.relpath(path, root)
            seen = set()
            for m in pat.finditer(text):
                g, v, form = forms[m.group(0)]
                key = (g, v, rel)
                if key in seen:
                    continue
                seen.add(key)
                line = text[:m.start()].count("\n") + 1
                snippet = text.splitlines()[line - 1].strip()[:90]
                print(f"{d}\t{g}/{v}\t{form}\t{rel}\t{line}\t{snippet}")
            for name, cpat in coping.items():
                cm = cpat.search(text)
                if cm:
                    line = text[:cm.start()].count("\n") + 1
                    snippet = text.splitlines()[line - 1].strip()[:90]
                    cope_rows.append(f"{d}\t{name}\t{rel}\t{line}\t{snippet}")
print("\n# coping notations (restatement evidence):")
print("consumer\tcoping_kind\tfile\tline\tsnippet")
for r in cope_rows:
    print(r)
EOF
}

case "$phase" in
  clone)  clone_phase ;;
  ledger) ledger_phase ;;
  sweep)  sweep_phase ;;
  all)    clone_phase; ledger_phase > "$work/k8s-ledger.tsv"; sweep_phase ;;
  *) echo "unknown phase $phase" >&2; exit 2 ;;
esac
