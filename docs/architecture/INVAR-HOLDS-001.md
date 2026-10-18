# INVAR-HOLDS-001: the durability line (trust is a duration, not a snapshot)

Status: DRAFT r1 for ruling. Implementation NOT authorized. Parent: Charter B
of 2026-10-18 (ratified in principle: hermetic computation from lock
metadata, zero git at check time, deep commit auditing stays in backtest).
Prepared to the house standard. Baseline: the clean tree at c0144d4.

## 0. What the line is, and the honest bounds

Every verifier on earth answers "is it green right now". This line answers
the question a human or an agent actually trusts: how long has this been
true? Each HOLDS outcome of `invar check` gains one indented sub-line:

```
[HOLDS] egress:OrderResponse ...
  durability: held since 2026-03-14 (218 days)
```

computed purely from the lock's own bitemporal metadata. Zero git, zero
network, zero new dependencies, zero new exit codes.

Bounds, stated before any code:

B1. Durability is ACCEPTANCE age, not correctness-at-every-commit. It states
how long the contract's current bindings have been accepted (each binding's
t_v `since` date, which `invar lock` already resets whenever a binding is
re-accepted after a change). Whether every intermediate commit also held is
`invar backtest`'s job, where git history is explicitly budgeted; this line
never claims it.
B2. The days figure reads the SAME `today` the check already reads for
waiver validation; no second clock source enters. Output therefore varies
across calendar days by design and is byte-stable within a day; the date
part is fully deterministic.
B3. v1 covers locked contracts (schema, egress, strings). Proof imports keep
their own freshness axis (accepted epochs, INVAR-PROOF-001) and are not
decorated; two different time semantics are never blended into one line.
B4. Rendering only, additive only. No existing output line changes by one
byte; machine artifacts (receipts, SARIF, report-md, MCP responses) are
byte-identical with and without the feature. This generalizes the Option B
lesson of INVAR-GATE-001 section K into a standing rule: new human-facing
text is added as new lines, never spliced into lines machines already read.

## A. Frozen scope

A1. The durability date of a contract is the MAXIMUM of its bindings' `since`
dates: the youngest acceptance. The honest sentence it encodes is "every
binding of this contract has been accepted since at least this date". A
minimum would let one old binding overstate the whole contract.
A2. The line appears under HOLDS outcomes only. VIOLATED and SKIPPED
outcomes carry no durability claim of any kind.
A3. A contract with no parseable `since` dates gets no line: omitted, never
guessed, exit code unchanged.
A4. check.rs stays Class 1 and untouched: the decoration lives where the
outcome lines are already printed (main.rs), reading the lock the check
already parsed. No new module is needed.

## B. Numbered requirements

R-H-1 Line format, frozen: two spaces, then
`durability: held since <YYYY-MM-DD> (<N> day(s))`, where `<N>` is
`today - since` in whole days, `0 days` for a same-day lock, and the unit
prints as `1 day` exactly at one. Printed immediately after the HOLDS
outcome's existing lines, one per held locked contract.

R-H-2 Source of truth: the parsed invar.lock's bindings for that contract
id; the date is max(since) per A1. recorded_at (t_r) is never used: it is
when INVAR wrote the entry, not when the binding was accepted.

R-H-3 Scope per A2 and B3: HOLDS on locked contracts only; nothing on
VIOLATED, SKIPPED, proof imports, or when no lock entry exists.

R-H-4 Machine surfaces frozen (B4): the receipt, SARIF, report-md and MCP
outputs contain no durability text in v1; a fixture asserts their silence.

R-H-5 Determinism: within one calendar day, byte-identical across runs; the
date part identical across days. No clock is read beyond the one `today`
the check already holds.

R-H-6 Independence: with the lock absent or every contract skipped, output
is byte-identical to today. Zero new dependencies, codes or exit changes.

## C. Countermodels (defined before fixtures and code)

CM-H-1 one ancient binding and one accepted yesterday: the line must say
yesterday (max, never min), or one stale survivor overstates the contract.
CM-H-2 a violated contract printing a durability line: a broken thing with a
proud age is worse than no line; declared: absent.
CM-H-3 a skipped contract printing one: a skip is never a pass and never has
an age; declared: absent.
CM-H-4 the date computed from recorded_at: a lock rewritten today for an
unchanged binding preserves since but refreshes nothing else; declared: the
line keeps the old since date.
CM-H-5 a receipt or SARIF file gaining durability bytes; declared: silent.
CM-H-6 a proof-import HOLDS line decorated; declared: absent.
CM-H-7 an unparsable since date producing an invented date or an error exit;
declared: line omitted, exit unchanged.

## D. Fixtures (declared outcomes, before any code)

H-F1 fresh lock, check: the held contract carries exactly
`  durability: held since <today> (0 days)`, with `<today>` computed by the
test from the library's own today_utc and format_date.
H-F2 the lock's since dates hand-edited to 2026-01-01 (the lock is plain
text and the fixture owns the repo): the line prints 2026-01-01 and the
exact day count the test computes with the same formula; recorded_at left
different from since, so CM-H-4 is bound in.
H-F3 producer change accepted via `lock --update`: the changed binding's
since resets, the line says held since today again.
H-F4 violated repo: no `durability:` text anywhere, exit 1 unchanged.
H-F5 unlocked repo (skipped, exit 4): no durability text.
H-F6 two bindings, one edited to 2026-01-01 and one left at today: the line
says today (CM-H-1, max).
H-F7 one repo with an egress, a schema and a strings contract, all held:
three durability lines, one per contract, each under its own HOLDS block.
H-N1 the same held repo with --receipt, --sarif and --report-md: none of
the three written files contains `durability` or `held since`.
H-N2 a consumer repo under check --proof with a holding import: the proof
HOLDS line carries no durability text.
H-M1 two runs the same day byte-identical; the no-lock output byte-identical
to the unchanged engine (asserted by the standing suites staying green).

## E. Mutation set (CAUGHT / SURVIVED / BUILD-FAILED; the last two are not kills)

1 min taken instead of max: H-F6. 2 durability printed on VIOLATED: H-F4.
3 printed on SKIPPED: H-F5. 4 day count off by one: H-F2. 5 the line removed:
H-F1. 6 date read from recorded_at: H-F2. 7 durability written into the
receipt: H-N1. 8 the frozen format changed (unit or wording): H-F1. 9 proof
imports decorated: H-N2. 10 a missing since invented as today: fixture H-F2
variant with one binding's since blanked, asserting omission. Each must die
by a named assertion failure.

## F. Gates 0 to 5

Gate 0 hash ledger at the clean tree c0144d4. Gate 1 fixtures first, failing
against the unchanged engine for the declared reasons (no durability line
exists; the N and M fixtures pass as invariance controls and stay). Gate 2
implementation to allowed paths. Gate 3 mutation harness all CAUGHT with
control green. Gate 4 full suite green, zero warnings, zero frozen-suite
edits, machine artifacts byte-identical. Gate 5 evidence ledger here,
STATUS.md entry.

## G. Path classes

Class 1 (untouched, hash-verified): the standing protected set, explicitly
including check.rs, lock.rs, config.rs, proof.rs, explain.rs, gate.rs,
init.rs, valuepairs.rs, strings.rs, codepairs.rs and both Cargo manifests.
Class 2: tests/holds_test.rs, scripts/holds/. Class 3 (REQUESTED, not
assumed): main.rs only (the decoration prints inside the existing outcome
loop, from the lock the check already parsed). Class 4: none; no new module
is needed, which is the point.

## H. Decision points for ruling

H-D1 Placement: the indented sub-line (recommended; no existing line changes
bytes, the Option B principle generalized) versus splicing into the [HOLDS]
line itself (rejected draft alternative: it rewrites every HOLDS line
machines may already read).
H-D2 Content: date plus day count (recommended; the count is the human
yield) versus date only (fully clock-free; rejected as less useful, with B2
noting the only cost of keeping the count).
H-D3 max(since) as the contract's date (A1).
H-D4 Machine surfaces frozen in v1 (R-H-4); a receipt field, if ever wanted,
is its own ruling with its own byte-stability analysis.
H-D5 Omission on unparsable or missing since dates (A3), never a guess and
never an error.
H-D6 Proof imports excluded in v1 (B3).

## I. Stop conditions

Stop and report on: any need to touch check.rs or any Class 1 file; any
byte change to an existing output line, a receipt, SARIF, report-md or an
MCP response; any frozen fixture failing; any second clock read; any
pressure to claim per-commit history that belongs to backtest; any new
dependency, code or exit change.

## J. Rulings of 2026-10-18 (recorded)

H-D1 to H-D6 APPROVED as drafted (21:59): sub-line placement as standing
doctrine, date plus day count from the one `today` the check already reads,
max(since), machine surfaces frozen, strict omission, proof imports excluded.
Class 3 granted for main.rs only; check.rs explicitly untouched.
Implementation GO under baseline c0144d4.

## K. Implementation Evidence Ledger (appended at Gate 5; sections 0 to J are unchanged)

Status: IMPLEMENTED. The whole engine change is one guarded block of
fourteen lines inside main.rs's existing outcome loop; Class 4 is empty, as
chartered.

### K1. Gate results

| Gate | Result |
|---|---|
| 0 baseline | the clean tree at c0144d4; every protected hash byte-identical after implementation |
| 1 fixtures first | 11 fixtures written before the line existed; 6 failed for the declared reason and 5 invariance controls passed |
| 2 implementation | main.rs only (the durability block in the outcome loop); check.rs, lock.rs and every other engine file byte-identical |
| 3 mutations | PASS: 10 of 10 CAUGHT by named assertion, control green, none SURVIVED, BUILD-FAILED or APPLY-FAILED (scripts/holds/mutate.sh), measured on a full re-run from scratch after the fixture strengthening of K2 |
| 4 regression | 255 of 255 tests green, zero warnings, zero frozen-suite edits; receipts, SARIF, report-md and proof outputs asserted silent |
| 5 documentation | sections J and K, STATUS.md entry |

### K2. What the gates taught (stated, not hidden)

- H-INT-1: spec A3's unparsable-since arm is unreachable by construction;
  lock::parse_bind bails on any bind line whose t_v does not parse, so the
  lock grammar is the stronger guard. The planned mutation became the
  singular/plural rule mutant instead.
- Gate 1 falsified the first reset fixture twice, and the engine was right
  both times: a blanket date replace poisoned recorded_at (whose RFC3339
  value begins with the same date), and renaming an UNBOUND field does not
  reset durability because nothing was re-accepted. The final fixture models
  the full honest loop: rename on both sides, re-declare the evidence in the
  manifest (the lock is evidence-bounded, slice 1 L01), then
  `lock --update`; only that re-acceptance resets the clock.
- Gate 3's first run produced two SURVIVING mutants, both convicting
  fixtures, not the engine: the aged-binding fixture had aged recorded_at
  (the first date occurrence in the lock) instead of a bind line, so
  min-versus-max was indistinguishable; and the no-lock skip scenario made
  the age-on-skipped mutant unreachable, since there was nothing to date.
  Both fixtures were strengthened (one bind line aged surgically with the
  other asserted to remain at today; a locked strings contract turned
  dynamic, the k3_u2 pattern, now asserts no age under a real lock entry)
  and the full harness was re-run from scratch.
- H-INT-2: a future-dated since (clock skew) is omitted, never printed as a
  negative age.
