# Contributing

The most valuable contribution is a real corpus case. The corpus
(`corpus-real/`) is a set of real cross-repository breaks with pinned
commits and honest scores; it is how INVAR's own claims get attacked, and
every detection number we publish comes from scoring held-out batches of it.

## Adding a corpus case

A case is a directory `corpus-real/r1-NNN-short-name/` with two files:

- `case.toml`: id, title, a public source URL (issue, PR, or commit that
  evidences the break), `real = "true"`, break kind, the expected residue
  (the fields the consumer actually read), and batch number.
- `invar.toml`: one `[[invariant]]` with `kind = "schema"`, pinned
  `producer` and `producer_base` refs (`github:OWNER/REPO@FULL_SHA:PATH`)
  and a pinned `consumer` ref.

Admission rules (from docs/falsification/R1-CORPUS.md):

1. The change is visible in the pinned producer diff AND the consumer read
   is visible at the pinned consumer ref; or
2. there is public evidence of consumer impact (an issue, a fix PR, a
   postmortem), linked in `case.toml`.

Controls (cases where nothing should fire) are welcome and are named
`r1-NNN-control-...` with `real = "false"` and `residue = "none"`.

## Honesty rules (non-negotiable)

- Synthetic or constructed cases are never labeled real. If you built the
  break yourself, it is a fixture, not corpus material.
- Expected residue is what the public evidence says broke, not what INVAR
  happens to detect.
- Scores are reported as measured, including the misses. A skip or an
  unevaluated site is never counted as a catch.

Run a case: `scripts/r1-run.sh target/release/invar` (set a `GITHUB_TOKEN`
for private rate limits; the runner compares INVAR's verdict against
`case.toml` and prints a TSV row per case).

## Code

`cargo build --locked && cargo test --locked`. No new dependencies (the
crate list is frozen; `curl` and `ssh-keygen` subprocesses are the two
ratified exceptions). Exit codes, lock format 1 and receipt format 1 are
frozen. PRs that add detection rules need a frozen spec and a held-out
batch plan first; read docs/falsification/ to see what that means.
