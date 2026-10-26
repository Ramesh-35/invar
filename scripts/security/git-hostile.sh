#!/usr/bin/env bash
# SECURITY-SPEC S3 measurement (SEC-D6, path 6): does any git invocation
# INVAR makes execute a program named by the repository's .git/config or
# .gitattributes? Each vector gets a fresh repository with one committed
# producer file; INVAR's exact argument vectors are then run (backtest.rs:
# log, show, rev-parse; proof.rs: rev-parse, hash-object), and the vector
# is EXECUTED when its marker file appears. Usage:
#   scripts/security/git-hostile.sh [scratch dir]
set -u
root="${1:-$(mktemp -d)}"; mkdir -p "$root"; root="$(cd "$root" && pwd)"
printf 'git %s\n' "$(git --version | awk '{print $3}')"
printf 'vector\tinvocation\tresult\n'
fresh() {
  local d="$root/$1"; rm -rf "$d"; mkdir -p "$d"
  git -C "$d" init -q
  git -C "$d" config user.email t@t; git -C "$d" config user.name t
  printf 'export interface A {\n  x: number;\n}\n' > "$d/p.ts"
  printf '*.ts filter=evil diff=evil\n' > "$d/.gitattributes"
  git -C "$d" add -A; git -C "$d" commit -q -m one
  printf 'export interface A {\n  x: number;\n  y: number;\n}\n' > "$d/p.ts"
  git -C "$d" commit -q -am two
  echo "$d"
}
mark() { printf '%s' "touch '$1/EXECUTED-$2'"; }
probe() {
  local name="$1" d="$2" h
  h="$(git -C "$d" rev-parse HEAD)"
  local -a inv=(
    "backtest log|-C|$d|log|-n|50|--format=%H %cs|--|p.ts"
    "backtest show|-C|$d|show|$h:p.ts"
    "backtest rev-parse prefix|-C|$d|rev-parse|--show-prefix"
    "backtest rev-parse shallow|-C|$d|rev-parse|--is-shallow-repository"
    "proof rev-parse HEAD|-C|$d|rev-parse|HEAD"
    "proof rev-parse blob|-C|$d|rev-parse|HEAD:p.ts"
    "proof hash-object|-C|$d|hash-object|p.ts"
  )
  for spec in "${inv[@]}"; do
    IFS='|' read -r -a parts <<< "$spec"
    local label="${parts[0]}"; rm -f "$d"/EXECUTED-*
    git "${parts[@]:1}" >/dev/null 2>&1 < /dev/null
    if ls "$d"/EXECUTED-* >/dev/null 2>&1; then r=EXECUTED; else r=no; fi
    printf '%s\t%s\t%s\n' "$name" "$label" "$r"
  done
}
d=$(fresh fsmonitor); git -C "$d" config core.fsmonitor "$(mark "$d" fsmonitor)"; probe core.fsmonitor "$d"
d=$(fresh filter); git -C "$d" config filter.evil.clean "$(mark "$d" clean); cat"; git -C "$d" config filter.evil.smudge "$(mark "$d" smudge); cat"; probe filter.clean+smudge "$d"
d=$(fresh process); git -C "$d" config filter.evil.process "$(mark "$d" process)"; git -C "$d" config filter.evil.required false; probe filter.process "$d"
d=$(fresh extdiff); git -C "$d" config diff.external "$(mark "$d" extdiff)"; probe diff.external "$d"
d=$(fresh textconv); git -C "$d" config diff.evil.textconv "$(mark "$d" textconv); cat"; probe diff.textconv "$d"
d=$(fresh pager); git -C "$d" config core.pager "$(mark "$d" pager); cat"; git -C "$d" config pager.log true; git -C "$d" config pager.show true; probe core.pager "$d"
d=$(fresh hooks); mkdir -p "$d/hk"; for hk in post-checkout post-index-change reference-transaction fsmonitor-watchman; do printf '#!/bin/sh\n%s\n' "$(mark "$d" hook)" > "$d/hk/$hk"; chmod +x "$d/hk/$hk"; done; git -C "$d" config core.hooksPath "$d/hk"; probe core.hooksPath "$d"
d=$(fresh gpg); printf '#!/bin/sh\n%s\nexit 1\n' "$(mark "$d" gpg)" > "$d/gpg.sh"; chmod +x "$d/gpg.sh"
tree=$(git -C "$d" rev-parse HEAD^{tree}); parent=$(git -C "$d" rev-parse HEAD)
c=$(printf 'tree %s\nparent %s\nauthor t <t@t> 1700000000 +0000\ncommitter t <t@t> 1700000000 +0000\ngpgsig -----BEGIN PGP SIGNATURE-----\n \n x\n -----END PGP SIGNATURE-----\n\nthree\n' "$tree" "$parent" | git -C "$d" hash-object -t commit -w --stdin)
git -C "$d" update-ref HEAD "$c"; git -C "$d" config log.showSignature true; git -C "$d" config gpg.program "$d/gpg.sh"; probe log.showSignature "$d"
# Positive controls: the same configuration, a git command known to honour
# it. A vector whose control does not fire proves nothing about INVAR.
control() {
  local name="$1" d="$root/$2"; shift 2
  rm -f "$d"/EXECUTED-*
  git -C "$d" "$@" >/dev/null 2>&1 < /dev/null
  if ls "$d"/EXECUTED-* >/dev/null 2>&1; then r=FIRES; else r=silent; fi
  printf '%s\tcontrol: git %s\t%s\n' "$name" "$*" "$r"
}
printf 'x\n' >> "$root/fsmonitor/p.ts"; control core.fsmonitor fsmonitor status
control filter.clean+smudge filter add p.ts
control diff.external extdiff diff HEAD~1 HEAD
control diff.textconv textconv log -p -n 1
python3 -c 'import pty,sys; pty.spawn(["git","-C",sys.argv[1],"log","-n","1"])' "$root/pager" >/dev/null 2>&1 < /dev/null; if ls "$root/pager"/EXECUTED-* >/dev/null 2>&1; then r=FIRES; else r=silent; fi; printf 'core.pager\tcontrol: git log on a terminal\t%s\n' "$r"
printf 'y\n' >> "$root/hooks/p.ts"; control core.hooksPath hooks commit -qam hook
control log.showSignature gpg log -n 1
