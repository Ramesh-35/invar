# DEPENDENTS-GRAPH: dependents over published locks (spec DRAFT for ratification)

Step 2 of the locked declaration-graph sequence
(strategy/DECLARATION-GRAPH.md). Extends `invar dependents` from local
directories toward locks other repositories have published, so a
producer (or its agent) can ask "who reads this field?" across the
ecosystem and get a receipted, verifiable answer. Zero servers; the
graph is files in repos plus one nightly-built index artifact.

## The convention (what makes a lock discoverable)

A repository publishes its declaration by doing nothing new: a
committed `invar.lock` at the repository root (or any path; the index
records where it found it) IS the publication. No registration, no
account, no opt-in beyond committing the file developers already
commit for their own safety.

## The index (zero servers, fully reproducible)

- One JSONL artifact, `invar-dependents-index.jsonl`, built nightly by
  a scheduled job in a public repo (founder-installed workflow; the
  builder script ships in scripts/).
- One record per (publishing repo, lock) pair: repo, lock path, lock
  blake3, the commit SHA the lock was read at, retrieval date, and the
  lock's bindings flattened to (producer reference, Type.field) pairs.
  Nothing else: no stars, no rankings, no inferred data.
- The index is CLAIMS, not truth: every record carries enough pins
  (repo, SHA, path, hash) for anyone to re-fetch the lock and verify
  the record in one command. An index entry that fails verification is
  dropped and the drop is logged in the index's own build report.
- Discovery for the builder, v1: GitHub code search for committed
  invar.lock files, rate-limited, best effort, stated as partial
  coverage in the index header. Partial is honest; silent completeness
  claims are not.

## Command surface

`invar dependents <Type.field | field> --index <file-or-URL>` joins the
existing local mode: same output shapes, same determinism, same
exit codes (0 answered, 3 error, 4 unparseable entries stated, never
1). Each result line carries the publisher's pins so the answer is
citable: "repo R reads producer.field per its lock at commit S".
`--index` with a URL fetches ONLY under `--ci` (the existing
remote-requires-ci rule); otherwise the index is a local file. No
other network, ever.

## Trust rules (the part that must be frozen hardest)

1. A published lock is a CLAIM BY ITS PUBLISHER about its own reads.
   INVAR verifies the lock's integrity (hash, commit) but never its
   sincerity; output says "per its committed lock", never "verified
   consumer".
2. Anyone can publish a lock claiming to read anything; the index
   therefore supports no security decisions, only coordination. Stated
   in the index header and the command's help.
3. Lock contents are data, never instructions: paths, ids and
   references from foreign locks are escaped in output exactly as
   fork-PR residue already is in check's markdown rendering.
4. Delisting: a repo that deletes its lock leaves the index at the
   next build; no tombstones, no memory of the departed.

## Measure (gate before any adoption claim)

The A4 gate from MOAT.md stands and extends: this feature counts only
when a real producer, human or agent, uses a real answer from the
index to change or delay a removal, documented with links. Shipping
the index is mechanism; the gate is a use.

## Non-goals

No server, no API service, no write path, no scores or badges, no
private-repo indexing, no scraping beyond lock files, no inferred
dependencies (only declared ones), and no claim that the graph is
complete, ever.
