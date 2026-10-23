# CHECK-INCR-1: reuse a HOLDS verdict when every input is unchanged

Status: RATIFIED (I-D1 to I-D6 and corrections C-1, C-2, founder,
2026-10-23), implemented through Gates 0 to 5; sections J and K. Sections A
to I are the draft r1 text, unchanged. Baseline: 0abe91e, release binary
fb6428e71806. All figures are from the sandbox (Linux x86_64).

## A. Evidence

Corpus: the 162 contracts EVAL-008 proposed at the pinned commits of the 6
repositories that have any (freeCodeCamp, vue, playwright and NextChat
have none). For each, the last 100 first-parent commits: 16,200 checks.
Scripts: scripts/perf/skipmeasure.py (rates), skipcost.py (instructions),
skipwall.py (wall time); raw rows in docs/evidence/perf/skip-100.tsv.

A1. Share of checks whose inputs (producer, consumer, code companion) are
byte-identical:

| Repo | Contracts | vs previous commit | vs a lock 100 commits back |
|---|---|---|---|
| pi | 108 | 97.1% | 73.5% |
| excalidraw | 28 | 68.0% | 14.5% |
| Understand-Anything | 12 | 95.7% | 35.2% |
| OmniRoute | 6 | 99.0% | 88.8% |
| puppeteer | 5 | 99.2% | 56.4% |
| claude-mem | 3 | 97.0% | 66.7% |
| all 16,200 | 162 | 92.1% | 60.4% |

A2. Cost of `invar check` evaluating only the changed contracts, per commit
(previous-commit basis), against evaluating all:

| Repo | All | Changed only, median commit | Read and BLAKE3 every input | Estimated reuse run |
|---|---|---|---|---|
| pi | 79.6 ms, 337 M instr. | 7.5 ms, 21 M | 3.5 ms (2.13 MB, 112 files) | 11.0 ms |
| excalidraw | 53.3 ms, 234 M | 16.2 ms, 64 M | 1.0 ms (1.20 MB) | 17.3 ms |
| Understand-Anything | 13.5 ms, 40 M | 2.5 ms, 2.1 M | 0.5 ms (0.22 MB) | 3.0 ms |

The floor (no contracts) is 2.6 to 3.6 ms. Hashing was timed in Python
over single-threaded BLAKE3, so it is an upper bound.

A3. What this is worth, stated plainly. On the largest measured repository
a reuse run saves about 69 ms per run. That matters for a check run on
every save or commit (pre-commit hook, editor, agent loop). In CI, where
a job takes tens of seconds to start, it is small; the CI case grows with
contract count, and no measured repository has more than 108.

## B. What a verdict reads (code read at 0abe91e)

B1. Every file a contract's evaluation reads goes through one function,
`Resolver::resolve` (resolver.rs), local or remote. So the input set of a
contract is exactly the list of references resolved while evaluating it.
It is captured there, not listed by hand per kind, so a kind that reads a
`list`, `trace` or `shape` file is covered the same way as `consumer`.

B2. The verdict also depends on things that are not files:
- the contract's `[[invariant]]` block in invar.toml;
- its `invar.lock` entry (bindings, pointer, members);
- waivers naming it, through today's date (days left, expiry);
- a locked sample naming it, through today's date (staleness) and
  `--sample`;
- the engine that evaluates it.

B3. Two facts that shape the design:
- `invar lock` today records the producer hash but evaluates nothing, so
  the lock holds no verdict to reuse.
- The 0abe91e parser ignores unknown keys in `[[contract]]` (probed: an
  added `held = "probe"` line, check unchanged, exit 0). New keys therefore
  need no lock format bump, and an older engine simply evaluates
  everything.

## C. Design: one hold record, two stores

C1. A hold record says: under engine E, contract X, with block B and lock
entry L, read inputs I1..In with these hashes, and HOLDS. Only a HOLDS
outcome is ever recorded.

C2. Store 1, invar.lock (stateless, committed). `invar lock` evaluates each
contract after writing its entry and appends a record when it HOLDS:

```text
[[contract]]
id = "orders"
producer = "openapi.yaml"
pointer = "/components/schemas/Order"
blake3 = "<producer bytes when locked>"
recorded_at = "2026-10-23T06:00:00Z"
bind = "spec 2026-10-23 Order.amount_cents integer"
held = "2026-10-23T06:00:00Z <engine> <blake3 of block> <blake3 of entry>"
held_input = "<blake3> openapi.yaml"
held_input = "<blake3> web/src/orders.ts"
```

The `held_input` lines are I-D2's consumer and companion hashes,
generalized to every file the evaluation read. The entry hash covers the
entry's other lines, so a hand-edited binding voids the record. Re-running
`invar lock` with nothing changed rewrites nothing.

C3. Store 2, `.invar/cache/verdicts.json` (local, never committed). It
holds the same records, keyed by contract id. Rules:
- `invar check` rewrites the file after each run, atomically (temp file,
  then rename).
- On first write it also creates `.invar/cache/.gitignore` containing `*`.
- A missing, unreadable or malformed cache is ignored without error, and
  the run evaluates everything.

C4. Reuse rule. Before evaluating a contract, `evaluate_with` reuses a
HOLDS only if all of these hold:
1. No waiver and no locked sample names the contract, and the run has no
   `--sample`. This replaces the date clause; see I-D2a.
2. A record exists, from the lock first, then the cache.
3. The record's engine, block hash and entry hash equal the current ones.
4. Every recorded input resolves again by the same reference, and its
   BLAKE3 equals the recorded one.

A resolve failure means no reuse; the normal evaluation then reports the
error exactly as today. Any one mismatch means the contract is evaluated.

C5. Engine identity: the crate version plus a BLAKE3 of the engine's own
source files, computed by a build script at compile time. A dev build with
the same version number but changed logic has a different engine. The
version string alone is not enough (I-D5).

C6. `--no-reuse` evaluates everything and is the reference for every
equivalence test.

## D. Output and exit codes

D1. A reused contract prints, per I-D3 with the corrections in H:

```text
  [HOLDS]    orders (schema) unchanged since lock 2026-10-23T06:00:00Z
```

or `unchanged since last run` when the record came from the cache. Its
fetched lines print as today. Notes produced only by evaluation (for
example "2 field(s) added since lock, tolerated") are not printed. A note
is never a verdict.

D2. Exit codes are unchanged: a reused contract counts as held (exit 0
contribution). JSON status tag is `HOLDS`. SARIF emits nothing for HOLDS,
as today. The summary line counts reused contracts among held and, when N > 0,
adds "(N reused)".

## E. Mutant harness parameters

- Harness: scripts/check-incr-1/mutate.sh, one patch per mutant in
  scripts/check-incr-1/mutants/, applied to a clean copy of the Gate 2
  tree. It runs `cargo test --release --test check_incr1_test`, one
  mutant at a time, with a 600 s timeout per mutant.
- A mutant is caught when the test target fails, and survives when it
  passes.
- Gate 3 passes at 100% of non-equivalent mutants caught. Any mutant
  claimed equivalent is argued in section K, in writing.

| Mutant | Change | Caught by |
|---|---|---|
| E01 | consumer hash not compared | F2 |
| E02 | producer hash not compared | F3 |
| E03 | companion hash not compared | F4 |
| E04 | block hash not compared | F5 |
| E05 | entry hash not compared | F6 |
| E06 | engine not compared | F7 |
| E07 | waiver guard removed | F8 |
| E08 | sample guard removed | F9 |
| E09 | record written for any verdict | F10 |
| E10 | cache read under `--ci` | F11 |
| E11 | lock record trusted under `--ci` without a verified signature | F12 |
| E12 | malformed cache is an error | F13 |
| E13 | cache `.gitignore` not written | F14 |
| E14 | missing input treated as unchanged | F15 |
| E15 | only the first recorded input compared | F4 |
| E16 | reference compared instead of bytes | F2 |
| E17 | reused contract contributes exit 4 | F1 |
| E18 | `held` time refreshed on an unchanged re-lock | F18 |

## F. Fixtures (frozen at Gate 1, before code)

Each is a temp workspace in tests/check_incr1_test.rs.

- F1: Lock, check, check. The second run prints "unchanged since lock",
  exits 0, and its held count equals the first.
- F2 (consumer): The cache records HOLDS at (P1, C1), where C1 no longer
  reads a field that P1 dropped and the lock still binds. C2 reads it
  again, with P1 unchanged, so the run must evaluate: VIOLATED, exit 1.
- F3 (producer): The cache records HOLDS at (P0, C0). P1 drops a field C0
  reads, so the run must give VIOLATED.
- F4 (companion): F3 through `producer_code`, with the contract reading
  three inputs.
- F5 (block): The pointer in invar.toml changes. The run must evaluate and
  hit the existing "different producer than invar.lock" error, exit 3.
- F6 (entry): One `bind` type is hand-edited in the lock. The run must give
  VIOLATED.
- F7 (engine): The engine token in `held` is edited. The run must evaluate,
  with no "unchanged since" text.
- F8 (waiver): A contract with an active waiver is never reused. Its WAIVED
  line and days left print.
- F9 (sample): A contract with a locked sample is never reused.
- F10: VIOLATED, NOT EVALUATED and SKIPPED contracts get no `held` lines
  and no cache record.
- F11: Under `--ci` the cache is neither read nor written.
- F12: Under `--ci`, a lock record is reused only when `--lock-signers`
  verified the lock in that run. Otherwise the contract is evaluated.
- F13: A malformed cache (bytes, empty, wrong shape) gives a normal run,
  normal exit, and a rewritten cache.
- F14: `.invar/cache/.gitignore` exists and contains `*`.
- F15: When a recorded input is deleted, the run gives today's error path,
  never a reuse.
- F16 (golden): On a 6-contract workspace, run with reuse against
  `--no-reuse`. Tags and exit code must be equal, and the output must be
  equal once the reuse suffix, the reuse count and evaluation-only notes
  are removed.
- F17: With no records (no `held` lines, no cache), output is
  byte-identical to 0abe91e.
- F18: `invar lock` twice with no change gives identical lock bytes.

## G. Gates

- Gate 0: baseline hashes of 0abe91e and fb6428e71806; full suite 404 of
  404.
- Gate 1: F1 to F18 committed and failing on the baseline (F17 passes by
  construction).
- Gate 2: implementation. Files: check.rs (reuse in `evaluate_with`), lock.rs
  (record, render, parse), resolver.rs (input capture), main.rs (`--no-reuse`,
  cache I/O, summary count), a build script for the engine id. demo and mcp
  call `evaluate_with` with reuse off, so they are unchanged.
- Gate 3: E01 to E18, 100% of non-equivalent caught.
- Gate 4: full suite, 0 warnings.
- Gate 5: replay over the 6 repositories, 100 commits each, two runs per
  commit, one with `--no-reuse` and one with reuse (cache from the previous
  commit, lock taken at the window start). Pass criteria:
  - all 16,200 tags and every exit code equal;
  - reuse rates reported beside A1;
  - the A2 wall table re-measured with the release binary.
  The same script is then run on the Mac.

## H. Rulings and decision points

Recorded (founder, 2026-10-23):
- I-D1: hybrid; the lock is primary (stateless), the local cache secondary
  and non-authoritative, with silent fallback.
- I-D2: lock gains input hashes; skip only when the producer, consumer and
  companion hashes, and the INVAR version, match, and no waiver has
  expired.
- I-D3: a reused contract prints an explicit HOLDS line; exit 0.

Two factual corrections to the ruled text:
- C-1: The line names the contract id (`orders`), not a route like
  `r.p.f`. Routes exist only in init's discovery.
- C-2: "since lock 0abe91e" is an INVAR commit, not a lock. The lock
  records times (`recorded_at`), so the line reads "unchanged since lock
  2026-10-23T06:00:00Z".

Also: reuse is not a proof that the contract holds. It replays a verdict
on identical inputs, and it is sound only if the record covers every input
(B1, B2). F2 to F9 and Gate 5 are the evidence for that.

For ruling:

- I-D2a (recommended): a contract named by a waiver or a locked sample is
  never reused. It replaces the date clause and the "any expiry voids all
  skips" rule.
  - Why: those outcomes change with the date (days left, staleness) even
    when no byte changes. Excluding them removes the date from the rule
    entirely, and costs only the waived contracts.
- I-D4 (recommended: signed-lock only in the gate): which records the gate
  (`--ci`) may trust.
  - The problem: a `held_input` line is a hash anyone can compute. A pull
    request could hand-edit invar.lock so that a violating consumer's hash
    matches, and a reused HOLDS would hide the violation. In a diff the
    edit looks like an ordinary re-lock. Today every lock edit that
    changes a verdict is a readable `bind` change.
  - Recommendation: under `--ci`, reuse a lock record only when
    `--lock-signers` verified the lock in that run. Never read the cache
    under `--ci`. Local runs reuse both.
  - The cost: unsigned CI evaluates everything, at most 80 ms on any
    measured repository (A3).
- I-D5 (recommended: version plus source hash): engine identity.
  - Why: the version string alone (0.1.0-alpha.29 spans several landed
    arcs) would replay verdicts made by different logic.
- I-D6 (recommended: yes): `invar lock` evaluates contracts so it can
  write `held` lines.
  - Effects: lock becomes as slow as one check. The lock diff changes
    when a consumer changes and someone re-locks, and `lock::diff` reports
    `held` changes on a separate line from binding changes.

## I. Bounds

- I-B1: No exit code changes. Only HOLDS is ever reused; VIOLATED, NOT
  EVALUATED and SKIPPED are always evaluated.
- I-B2: With no records, output is byte-identical to 0abe91e (F17).
- I-B3: Zero new dependencies. blake3 and serde_json are already used; the
  build script uses blake3 as a build dependency of the same crate.
- I-B4: Lock format number unchanged (B3); older engines ignore the new
  keys and evaluate.
- I-B5: init, its PROPOSED output and the PRESENT, ABSENT and UNRESOLVED
  interface are untouched. `invar mcp` and `invar demo` are unchanged in
  this arc; the warm MCP path is already 2.1 ms on the Mac.
- I-B6: Published figures are the measured ones, per repository and
  machine, never a general claim.

## J. Rulings of 2026-10-23 (recorded)

- C-1, C-2: RATIFIED. The reused line names the contract id and the time the
  record was made: `[HOLDS]    orders (schema) unchanged since lock
  2026-10-23T06:00:00Z`, or `unchanged since last run`.
- I-D4: RATIFIED. Under `--ci` the cache is never read or written; a lock
  record is reused only when `--lock-signers` verified invar.lock in that
  run. Unsigned CI evaluates everything.
- I-D2a: RATIFIED. A contract named by a waiver or a locked sample, and every
  contract of a `--sample` run, is evaluated, never reused or recorded.
- I-D5: RATIFIED. Engine id = crate version, "-", BLAKE3 computed by build.rs.
- I-D6: RATIFIED. `invar lock` evaluates and writes `held` and `held_input`
  only for HOLDS; `lock::diff` reports `held` changes on separate lines.

Precision on the ruled wording (no change to what was ruled):
- Reuse is sound if the record covers every input of the verdict. That rests
  on a code-reading argument (B1: every file read goes through
  `check::fetch`) checked by fixtures F2 to F9 and the Gate 5 replay. It is
  not a mechanical proof.
- invar.lock is cryptographically bound only when signed and verified
  (`--lock-signers`). Unsigned, a `held` line is a hash anyone can compute,
  which is why I-D4 exists.
- The engine id hashes sources, Cargo.lock and the compiler version at build
  time. A binary patched after the build keeps its id; detecting that is a
  release-integrity question, outside this arc.

## K. Results (Gates 0 to 5, sandbox, Linux x86_64)

Gate 0. The tree is the byte-exact `git archive` of 0abe91e from the
founder's machine (tar sha256 ddd7d991c132...); the rebuilt baseline binary
hashes fb6428e71806; 404 of 404 tests, 0 warnings.

Gate 1. tests/check_incr1_test.rs froze F1 to F18 plus two more: F19 (a
`--receipt` run reuses nothing; its evidence comes from a full evaluation)
and F20 (a `held` record out of date while bindings are unchanged is a
stated note, never LOCK_DRIFT; `invar lock --update` refreshes it). On the
baseline, 19 of 20 fail and F17 passes by construction. One amendment after
freezing (K-1): the frozen reuse-detector matched the pre-existing note
"producer unchanged since lock (...)", so it rejected correct evaluated
outcomes; it now matches only the HOLDS line itself. The amended file was
re-run on the baseline: same 19 failures, F17 passes.

Gate 2. src/reuse.rs (records, capture, cache, engine id via build.rs),
reuse in `evaluate_reusing` (check.rs), `held`/`held_input` render, parse
and `diff_held` (lock.rs), wiring and `--no-reuse` (main.rs). K-2: `invar
lock` now records a HOLDS, so existing tests that run lock-then-check would
have silently stopped exercising evaluation; their `check` calls now pass
`--no-reuse` (11 such tests found by their failures, then all helpers
patched), and one lock byte-equality test compares the entry without its
`held` lines. Full suite 424 of 424, 0 warnings; clippy count unchanged
from the baseline (28).

Gate 3. 22 mutants (E01 to E18 of section E, renumbered and extended with
E19 receipt-run reuses, E20 stale held is LOCK_DRIFT, E21 sample-run
reuses, E22 --no-reuse ignored). Kill log:
docs/evidence/incr/mutate.log.

Gate 5. Replay of the last 100 first-parent commits of every EVAL-008
repository: at each commit, `invar check --no-reuse` against `invar check`
(cache carried from the previous commit; lock taken at the window's first
commit where every input exists, which for pi is commit 89 and for
excalidraw commit 57, since inputs named at the pinned commit do not exist
earlier). Required and observed: equal exit codes at all 600 commits, equal
tags on all 5,128 evaluable contract-commit pairs, and equal output after
removing the reuse suffix, the reuse count and notes; at commits where an
input file does not exist, both modes error identically (exit 3). Zero
disagreements.

| Repo | Evaluable pairs | Reused | Mean ms, full, evaluable commits | Mean ms, reuse |
|---|---|---|---|---|
| pi | 1296 | 98.8% | 89.7 | 17.5 |
| excalidraw | 1232 | 69.4% | 68.2 | 35.0 |
| Understand-Anything | 1200 | 95.7% | 15.8 (median) | 7.6 (median) |
| OmniRoute | 600 | 99.0% | 10.9 (median) | 4.3 (median) |
| puppeteer | 500 | 99.2% | 12.9 (median) | 7.0 (median) |
| claude-mem | 300 | 97.3% | 13.6 (median) | 7.0 (median) |

The 16,200 pairs of section A counted file-diff rows per commit; `invar
check` errors (exit 3) at a commit where an input file does not exist, so
the replay's evaluable pairs are fewer. Raw rows:
docs/evidence/incr/*.tsv; harness: scripts/check-incr-1/replay.py.

Not claimed: any latency on other machines; mechanical proof of the
input-completeness argument (B1); tamper-evidence of unsigned locks
(I-D4's reason); detection of a binary patched after build (see J).
