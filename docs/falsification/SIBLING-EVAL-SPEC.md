# SIB-1: tell a producer which consumers its change breaks (P5)

Status: r1, FROZEN with its fixtures before any implementation. r0 was the
draft ruled on 2026-10-27; its text is kept in git history. Parent: the
2026-10-27 research evaluation and ruling ("upgrade --locks from
identifying bindings to evaluating them"). Depends on LOCK-1 (P4).
Fixtures: tests/sibling_eval_test.rs; hashes in SIBLING-EVAL-SPEC.frozen.

## A. The gap, from the code

`invar check --locks <dir>` (A4.1) states which sibling invar.lock files
bind this repository's producer files ("3 bindings"), not whether the
working tree still provides those fields. Three further gaps, measured:
1. Remote producer refs (`github:OWNER/REPO@REF:PATH`), the form a real
   consumer lock uses, are skipped, so a shared folder of consumer locks
   binds nothing today.
2. Sibling locks are read with no signature check.
3. `invar check` refuses a repository with no contracts of its own (exit
   3), which is exactly a pure producer repository.

## B. Rules

Evaluation mode is `invar dependents --locks <dir> --repo OWNER/NAME
[--lock-signers <file>]` with no field query (P5-D5), run in the producer
repository's root. The query mode is unchanged.

- SB-1, identity (P5-D1). A lock contract is this repository's when its
  producer is `github:OWNER/REPO@REF:PATH` and OWNER/REPO equals `--repo`,
  ASCII case-insensitive. REF is ignored (the question is the working
  tree); PATH resolves against the current directory. Other locks are not
  listed. No network, no git remote lookup.
- SB-2, evaluation. The producer file is extracted with the extractor
  `check` and `backtest` use. Each binding is `absent` (backtest's naming
  form presence rule), `retyped` (locked type and current type both known
  and different), or provided. A scoped binding absorbs the unscoped
  binding of the same field in the same contract, so a field counts once.
  A producer file that is missing or cannot be extracted is `not
  evaluated`, never `absent`.
- SB-3, provenance (P5-D2, P5-D3). One allowed-signers file names every
  trusted consumer repository; a lock's signer is found with `ssh-keygen
  -Y find-principals` and verified (namespace `invar-lock`) over the
  exact bytes then parsed. Without `--lock-signers`, or with no signature,
  or a signature that does not verify, the lock is `not evaluated` with
  that reason, and nothing read from it reaches the output.
- SB-4, advisory (P5-D4). Broken fields never fail the run: exit 0 when
  everything matched was evaluated, 4 when anything was not (stated blind
  spots, as `dependents` already exits 4 on unreadable locks), 3 on error.
  No query and no `--repo` is an error. Text only in this slice
  (`--format json` with `--repo` is an error); MCP and the PR report
  follow on the same code path.
- SB-5, honesty. Every evaluation ends with: `note: a lock records the
  fields a consumer depends on, not the lines that read them.`

Output shape:

    consumer-a/invar.lock [verified as consumer-a@acme] lk:
        Limits.maxLines absent (locked number)
        1 binding(s) provided
    consumer-b/invar.lock: not evaluated: unsigned (no invar.lock.sig)
    summary: 1 broken | 1 provided | 1 lockfile(s) not evaluated
    note: a lock records the fields a consumer depends on, not the lines that read them.

## C. Kill gate (ruled and measured 2026-10-27)

On the 37 non-held-out corpus cases (31 breaks, 6 controls; the 85 sealed
held-out cases untouched): lock each consumer against producer_base, then
against the post-break producer; a vanished binding is `absent`, a changed
type `retyped` (scripts/sibling/killgate.py, evidence
docs/evidence/sibling/killgate.txt; a second run is byte-identical).

| | Cases | Result |
|---|---|---|
| Breaks | 31 | 15 actionable, 9 silent, 7 not measured |
| Controls | 6 | 0 false alarms, 5 quiet, 1 not measured |

Not measured, stated: 6 consumers in languages INVAR does not read (C++,
Swift, Java) and r1-000 (no producer_base). Gate: PASS (15, threshold 3).
Tuning rung only: not recall; the sealed record stays 38 of 70. r1-025 and
r1-028 name more absent bindings than their recorded residue (the output
changed wholesale); r1-012's Rust type text is the extractor's raw form.
The gate assumes consumers publish signed locks; how many will is the open
adoption question.

## D. Premises, measured on the pre-P5 engine

scripts/sibling/premises.sh, evidence premises-baseline.log: a lock
rewritten to a remote producer parses and is found by today's query mode;
the TS extractor types the retyped field `string`; one signers file names
the signer (`find-principals`) and verify accepts it; an untrusted signer
is named by no principal (exit 255); today's CLI rejects `--repo` (exit 2).

## E. Fixtures (frozen with r1)

| Fixture | Asserts |
|---|---|
| p5_f01 | removed field named `absent (locked number)`, exit 0, note line |
| p5_f02 | retyped field named `retyped (locked number, now string)` |
| p5_f03 | all provided: each field counted once (`2 binding(s) provided`) |
| p5_f04 | unsigned lock: not evaluated, nothing of it printed, exit 4 |
| p5_f05 | signature by an untrusted key: refused, exit 4 |
| p5_f06 | lock edited after signing: refused, exit 4 |
| p5_f07 | a lock on another repository is not listed, exit 0 |
| p5_f08 | `Acme/API` matches `acme/api` |
| p5_f09 | no `--lock-signers`: not evaluated, exit 4 |
| p5_f10 | missing producer file: not evaluated, never absent, exit 4 |
| p5_f11 | no query and no `--repo`: exit 3 |
| p5_f12 | `--format json` with `--repo`: exit 3 |

On the current engine all 12 FAIL at their first exit-code assertion (the
parser rejects `--repo` or requires a query); setup (lock, rewrite, sign)
succeeds in every fixture.

## F. Pre-registered mutation operators

| Operator | Change | Declared killer |
|---|---|---|
| P5-M1 | every binding reported provided | p5_f01 |
| P5-M2 | types not compared | p5_f02 |
| P5-M3 | scoped binding does not absorb unscoped | p5_f03 |
| P5-M4 | unsigned lock evaluated | p5_f04 |
| P5-M5 | signature not verified | p5_f05 |
| P5-M6 | repository not compared (any owner/name) | p5_f07 |
| P5-M7 | repository compared case-sensitively | p5_f08 |
| P5-M8 | no signers: locks evaluated | p5_f09 |
| P5-M9 | missing producer reported absent | p5_f10 |
| P5-M10 | blind spots exit 0 | p5_f04 |
| P5-M11 | note line omitted | p5_f01 |

Not testable by a fixture, stated: that the verified bytes and the parsed
bytes are one read (held by review of one read path, as in LOCK-1).

## G. Bounds

No verdict, exit code meaning or `check` behavior changes; the query mode
of `dependents` unchanged; zero network; zero new dependencies; frozen
spec and fixtures before code; mutation operators with declared killers;
native verification.
