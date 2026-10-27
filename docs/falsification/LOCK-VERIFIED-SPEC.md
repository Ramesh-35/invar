# LOCK-1: does the agent's memory carry a signature? (AGENT-1 P4)

Status: r0, FROZEN with its fixtures before any implementation, for
founder ruling. Parent: docs/architecture/AGENT-1.md, P4 as ruled
2026-10-25 (AG-D3), and the P4 authorization of 2026-10-26. Baseline:
main with S3 and JSON-1 (native 562 of 562). Fixtures:
tests/lock_verified_test.rs; hashes in LOCK-VERIFIED-SPEC.frozen.

## A. The gap

invar.lock is an agent's durable record of what a repository depends on:
bound fields, producer hashes, hold records. It is authenticated only
when its signature verifies against a signers file the operator chose.
`invar check --lock-signers` already enforces that (fail closed, exit 3).
The MCP server has no such option, and its `check` and `contract` results
say nothing about the lock's provenance, so an agent cannot tell an
authenticated lock from one anyone with write access could have edited.
AGENT-1 P4: never let an unsigned lock pass as authenticated memory.

## B. Rules

- L-1, state. The `check` and `contract` results carry `lock_state`, one
  of `absent` (no invar.lock beside invar.toml), `unsigned` (a lock, and
  no signers configured for this server), `verified` (its signature
  verified in this call), and `lock_verified`, true exactly when
  `lock_state` is `verified`. Additive: every existing key keeps its value.
- L-2, configuration. `invar mcp` takes `--lock-signers <file>` and
  `--lock-principal <name>`, the same pair and the same namespace
  (`invar-lock`) as `invar check`. Signers without a principal: the server
  does not start (exit 3, `--lock-signers needs --lock-principal` on
  stderr). A signature file on a server started without signers proves
  nothing: the state stays `unsigned`.
- L-3, every call. With signers configured, each `check` and `contract`
  call reads invar.lock once, verifies invar.lock.sig over exactly those
  bytes, and parses those same bytes. Nothing is cached between calls: a
  lock edited while the server runs is seen by the next call.
- L-4, fail closed. With signers configured, a missing lock, a missing
  signature or a signature that does not verify makes the call an error
  (`ERROR (not a verdict): lock not confirmed: ...`, `isError` true, no
  structuredContent), as `invar check --lock-signers` exits 3. A refused
  lock never yields a verdict and never degrades to `unsigned`.
- L-5, stdout. The server's stdout carries JSON-RPC only; verification
  writes nothing there (the CLI's `lock signature ok` line stays CLI-only).
- Unchanged: the CLI (`check --lock-signers` keeps its text and exit
  codes), the `dependents` and `coverage` tools, every verdict.

## C. Measured before code

- On the current engine, neither result carries any lock provenance (the
  JSON-1 key set, docs/evidence/json/corpus-json.txt).
- Premises of the signed fixtures, through the CLI's existing path on the
  current engine (scripts/lock/premises.sh, evidence premises-baseline.log):
  a trusted signature verifies (exit 0); a missing signature, a signature
  by an untrusted key, a lock edited after signing, and a lock with one
  appended newline are each refused (exit 3, `lock not confirmed`).

## D. Fixtures (frozen with r0)

tests/lock_verified_test.rs, 10 tests. Every MCP call also asserts that
the stdout line is JSON-RPC with the request's id (L-5).

| Fixture | Asserts |
|---|---|
| p4_f01 | no lock: `absent`, false; existing keys unchanged |
| p4_f02 | an unsigned lock: `unsigned`, false, on `check` and `contract`; existing keys unchanged |
| p4_f03 | a valid signature but no signers configured: `unsigned`, false |
| p4_f04 | a lock signed by the trusted key: `verified`, true, on both tools |
| p4_f05 | signers configured, no signature: both tools refuse |
| p4_f06 | a signature by an untrusted key: refused |
| p4_f07 | a lock edited after signing: refused |
| p4_f08 | verified at the first call, edited while running, refused at the next (L-3) |
| p4_f09 | `--lock-signers` without `--lock-principal`: exit 3 at start, stdout empty |
| p4_f10 | guard: `invar check --lock-signers` unchanged |

On the current engine: 9 FAIL (f01 to f03 at the new keys; f04 to f08
because the server rejects the new flags and exits; f09 exits 2 from the
argument parser, not 3), and the guard f10 passes.

## E. Pre-registered mutation operators

| Operator | Change | Declared killer |
|---|---|---|
| P4-M1 | any present lock reported verified | p4_f02 |
| P4-M2 | a missing lock reported unsigned | p4_f01 |
| P4-M3 | signers configured, signature not checked | p4_f06 |
| P4-M4 | verification done once, at the first call | p4_f08 |
| P4-M5 | a refused lock degrades to an `unsigned` result | p4_f05 |
| P4-M6 | principal not required | p4_f09 |
| P4-M7 | a status line written to stdout | p4_f04 |
| P4-M8 | `contract` without the lock fields | p4_f02 |
| P4-M9 | a signature counted without configured signers | p4_f03 |

Not testable by a fixture, stated: that the verified bytes and the parsed
bytes are the same read (L-3); held by code review of one read path.

## F. Decision points

- P4-D1 (carrying the configuration). (a) The MCP `Server` struct gains a
  signers field; one existing struct literal (tests/harness_test.rs, the
  MCP test, line 1293) needs a mechanical `lock_signers: None`, no
  assertion changed (recommended: explicit, no global state). (b) A
  process-global set by main before serving: no fixture touched, but the
  server's behavior then depends on hidden state.
- P4-D2 (refusal). A lock that does not verify is an error with no result
  (recommended: the same fail-closed rule as `invar check`), or a result
  with `lock_state` `rejected` and the contracts still evaluated.
- P4-D3 (scope). `check` and `contract` (recommended), or also
  `coverage`. `dependents` reads other repositories' locks; their
  provenance is a separate question and not in P4.

## F2. Rulings (2026-10-27)

P4-D1 (a), P4-D2 refusal is an error, P4-D3 `check` and `contract` only:
all ratified as recommended.

## F3. Amendment r1 (fixture defect, found at implementation)

p4_f01 asserted `result` `all_hold` as an unchanged existing key. The
engine before P4 reports a contract with no lock entry and no literal
reads as `not_fully_evaluated`, exit 4 (measured on the pre-P4 binary and
the P4 binary: same value, P4 adds only `lock_state` and `lock_verified`).
The r0 baseline failed one line earlier, at the missing `lock_verified`
key, which hid the wrong expectation. r1 asserts `not_fully_evaluated`
and `exit_code` 4. No other fixture changed; no operator's killer changed.

## G. Bounds

No verdict changes; the CLI and its exit codes untouched; additive JSON
only; zero new dependencies; frozen spec and fixtures before code;
mutation operators with declared killers; native verification.
