#!/usr/bin/env bash
# B1 harvest, Terraform family: the declared contract is the AWS provider's
# resource schema (schema.Schema maps in Go). Diff aws_s3_bucket's top-level
# attributes across the v3 -> v4 major boundary (writable arguments turned
# deprecated/read-only), and check the v4 -> v5 removal of the
# aws_subnet_ids data source. Consumers: terraform-aws-modules repositories,
# cloned FULL so git log -S can pin the exact commits where each module
# read, then absorbed, the change.
# Usage: bash harvest-terraform.sh <work-dir> [clone|schema|sweep|all]
set -euo pipefail
work="$1"; phase="${2:-all}"
mkdir -p "$work"

PROVIDER=https://github.com/hashicorp/terraform-provider-aws
PROVIDER_TAGS=(v3.76.1 v4.0.0 v5.0.0)
MODULES=(
  https://github.com/terraform-aws-modules/terraform-aws-s3-bucket
  https://github.com/terraform-aws-modules/terraform-aws-vpc
  https://github.com/terraform-aws-modules/terraform-aws-atlantis
  https://github.com/terraform-aws-modules/terraform-aws-eks
)

clone_phase() {
  for t in "${PROVIDER_TAGS[@]}"; do
    d="$work/provider-$t"
    if [ ! -d "$d/.git" ]; then
      git clone -q --depth 1 --branch "$t" "$PROVIDER" "$d" \
        || echo "WARN: could not clone provider@$t" >&2
    fi
  done
  for m in "${MODULES[@]}"; do
    name="$(basename "$m")"
    if [ ! -d "$work/$name/.git" ]; then
      git clone -q "$m" "$work/$name" || echo "WARN: could not clone $m" >&2
    fi
  done
  echo "clone phase done" >&2
}

schema_phase() {
python3 - "$work" <<'EOF'
import os, re, sys
work = sys.argv[1]

def parse_schema(path):
    """Top-level attributes of the FIRST `Schema: map[string]*schema.Schema{`
    block in the file, with their flag set. Brace-tracked so nested Elem
    schemas do not leak into the top level."""
    text = open(path, errors="ignore").read()
    m = re.search(r"Schema:\s*map\[string\]\*schema\.Schema\{", text)
    if not m:
        return {}
    i = m.end()
    depth = 1
    attrs = {}
    cur, start = None, None
    while i < len(text) and depth > 0:
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 1 and cur:
                body = text[start:i]
                flags = set(re.findall(r"(Optional|Required|Computed):\s*true", body))
                dep = re.search(r'Deprecated:\s*"([^"]*)"', body)
                if dep:
                    flags.add("Deprecated")
                attrs[cur] = (sorted(flags), dep.group(1)[:80] if dep else "")
                cur = None
        elif c == '"' and depth == 1:
            m2 = re.match(r'"([a-z0-9_]+)":\s*\{', text[i:])
            if m2:
                cur = m2.group(1)
                start = i + m2.end()
                depth += 1
                i += m2.end()
                continue
        i += 1
    return attrs

paths = {
    "v3.76.1": os.path.join(work, "provider-v3.76.1", "internal", "service", "s3", "bucket.go"),
    "v4.0.0":  os.path.join(work, "provider-v4.0.0", "internal", "service", "s3", "bucket.go"),
}
schemas = {}
for tag, p in paths.items():
    if not os.path.exists(p):
        sys.exit(f"missing {p}")
    schemas[tag] = parse_schema(p)
    print(f"# {tag}: {len(schemas[tag])} top-level attributes", file=sys.stderr)

a, b = schemas["v3.76.1"], schemas["v4.0.0"]
print("attribute\tv3_flags\tv4_flags\tchange\tv4_deprecation_message")
for k in sorted(set(a) | set(b)):
    fa = ",".join(a[k][0]) if k in a else "-"
    fb = ",".join(b[k][0]) if k in b else "-"
    if k not in b:
        change = "REMOVED"
    elif k not in a:
        change = "added"
    elif a[k][0] != b[k][0]:
        change = "RETYPED"
    else:
        change = "same"
    if change in ("REMOVED", "RETYPED"):
        print(f"{k}\t{fa}\t{fb}\t{change}\t{b[k][1] if k in b else ''}")

# v4 -> v5: aws_subnet_ids data source removal (file-level, both layouts).
for tag, cand in (("v4.0.0", ["internal/service/ec2/subnet_ids_data_source.go"]),
                  ("v5.0.0", ["internal/service/ec2/vpc_subnet_ids_data_source.go",
                              "internal/service/ec2/subnet_ids_data_source.go"])):
    found = [c for c in cand if os.path.exists(os.path.join(work, f"provider-{tag}", c))]
    print(f"# aws_subnet_ids data source at {tag}: {found or 'ABSENT (checked ' + str(len(cand)) + ' paths)'}", file=sys.stderr)
import subprocess
for tag in ("v4.0.0", "v5.0.0"):
    r = subprocess.run(["grep", "-rl", "aws_subnet_ids", os.path.join(work, f"provider-{tag}", "internal", "service", "ec2")],
                       capture_output=True, text=True)
    files = [f for f in r.stdout.splitlines() if f.endswith(".go") and "_test" not in f]
    print(f"# grep aws_subnet_ids in {tag} ec2 service (non-test): {len(files)} files", file=sys.stderr)
    for f in files[:4]:
        print(f"#   {os.path.relpath(f, work)}", file=sys.stderr)
EOF
}

sweep_phase() {
  for repo in terraform-aws-s3-bucket terraform-aws-vpc terraform-aws-atlantis terraform-aws-eks; do
    d="$work/$repo"
    [ -d "$d/.git" ] || continue
    echo "=== $repo"
    echo "--- commits touching aws_subnet_ids (git log -S):"
    git -C "$d" log -S "aws_subnet_ids" --format="%h %cs %s" -- '*.tf' | head -6
    if [ "$repo" = terraform-aws-s3-bucket ]; then
      echo "--- commits touching monolithic s3 args (acl/versioning/server_side_encryption_configuration inside main.tf):"
      git -C "$d" log -S "server_side_encryption_configuration" --format="%h %cs %s" -- main.tf | head -8
      last_v2=$(git -C "$d" tag -l 'v2.*' --sort=v:refname | tail -1)
      echo "--- last v2 tag: $last_v2; monolithic args at that tag:"
      git -C "$d" show "$last_v2:main.tf" | grep -nE "^\s+(acl|versioning|cors_rule|server_side_encryption_configuration|logging|website|lifecycle_rule)\b" | head -12
      first_v3=$(git -C "$d" tag -l 'v3.*' --sort=v:refname | head -1)
      echo "--- first v3 tag: $first_v3; separate-resource rewrite evidence:"
      git -C "$d" show "$first_v3:main.tf" | grep -nE "resource \"aws_s3_bucket_" | head -12
    fi
  done
}

case "$phase" in
  clone)  clone_phase ;;
  schema) schema_phase ;;
  sweep)  sweep_phase ;;
  all)    clone_phase; schema_phase > "$work/tf-schema-diff.tsv"; sweep_phase ;;
  *) echo "unknown phase $phase" >&2; exit 2 ;;
esac
