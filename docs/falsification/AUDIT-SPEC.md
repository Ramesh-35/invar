# AUDIT: every conviction re-verified against the bytes it cites (AGENT-1 P2)

Status: r1, FROZEN (bounded repair, founder's ruling of 2026-10-25,
18:42: one fixture added, one rule clarified for lock mode; no other rule
change). r0 was frozen with its fixtures before implementation and ruled
2026-10-25 (AU-D1, AU-D2 as recommended); it is preserved as
AUDIT-SPEC-r0.md (freeze record AUDIT-SPEC-r0.frozen). Section I records
the r1 changes. Parent: docs/architecture/AGENT-1.md as ruled 2026-10-25
(AG-D1 claims restated; AG-D2 a failed self-audit is exit 3; AG-D3 audit
first). Baseline: a4aa356 (G2 landed, native 501 of 501). Fixtures:
tests/audit_test.rs (r0) and tests/audit_r1_test.rs (r1); hashes in
AUDIT-SPEC.frozen.

## A. What the audit is, and what it is not

Before INVAR reports VIOLATED for a schema contract, it re-checks every
witness against the bytes it evaluated in that run. A witness that does
not survive its own check is an engine defect: the run is an ERROR
(exit 3, `ERROR (not a verdict): self-audit failed: ...`), never a
VIOLATED.

The audit checks INVAR against its own evidence. It does not decide that a
cited read is semantically a read of that field (r1-116 passes it, and
that is stated in AGENT-1). It turns "the witness points at real lines
that say what the witness says" from a reviewer's habit into a property
the engine enforces on every run.

## B. Rules

For a witness W of contract C (kind `schema`, strict, change-scoped and
lock mode), with consumer name N and consumer bytes T (the text evaluated
in this run), and the bound producer names P (producer, base, code
companion and its base, as artifact names). The FIELD is the residue,
except in lock mode, where the residue is shown as `Type.field` and the
field is the bound field the engine matched (the `field` part):

- AU-1, consumer sites. Every entry of W.sites has the form `N:line`
  (the name exactly N), the line is between 1 and the line count of T, and
  that line's tokens include the field or a form W declares under
  `read as`. Tokens of a line: every identifier (`[A-Za-z_][A-Za-z0-9_]*`)
  and the content of every string literal (`"..."`, `'...'`, `` `...` ``),
  tokenized left to right; a substring is never a token (`metadata` does
  not contain the token `data`).
- AU-2, producer. W.missing_from, when present, begins with a name in P;
  W.type_change, when present, names a place beginning with a name in P.
- AU-3, request side (RS-1). A witness whose missing_from carries the RS-1
  form (`... no inbound read and no literal of K remain (inbound read at
  B:L at base); written at N:L1[, L2...]`) has no sites; its consumer
  lines are the `written at` lines, and each must exist in T and hold K as
  a token. B must be in P.
- Outcome: all rules hold, nothing changes (output byte-identical). Any
  rule fails, the evaluation returns an error naming the contract, the
  field, the rule and the site; `invar check` prints `ERROR (not a
  verdict): self-audit failed: ...` and exits 3, as every evaluation
  error already does.

Scope: `schema` contracts. Other kinds (event, lock_order and the rest)
are not audited by r0, and the spec says so; extending the audit is its
own spec.

## C. Measured before code

Over every VIOLATED corpus case on the a4aa356 engine (65 cases), using
exactly AU-1 on the sites the text output shows (up to three per witness;
the implementation checks every site): 239 of 239 cited consumer sites
hold the field or a declared read form as a token, and RS-1's three write
lines on r1-118 hold `pdfFormat`. Zero audit failures are expected on the
corpus; any failure is a defect to fix, never a reason to weaken the rule.

## D. Pre-registered movement

None. The full-corpus text output stays byte-identical on all 122 cases,
and every existing fixture keeps its result. The audit adds no output on
success.

## E. Fixtures (frozen with r0)

tests/audit_test.rs drives `invar_falsify::check::audit_witness(witness,
consumer_name, consumer_text, producers)` directly, so each rule is
exercised by a constructed witness (an engine that is correct never
produces a bad one), plus one CLI fixture that a real conviction still
exits 1 with no audit text. On the baseline the file does not build (the
function does not exist); every fixture is new behavior.

| Fixture | Asserts |
|---|---|
| au_f01 | a witness whose site holds the field passes |
| au_f02 | a site past the end of the consumer fails (AU-1) |
| au_f03 | a site whose line lacks the field fails (AU-1) |
| au_f04 | a declared `read as` form is accepted (AU-1) |
| au_f05 | a substring is not a token: `metadata` for field `data` fails (AU-1) |
| au_f06 | a site naming another file fails (AU-1) |
| au_f07 | a string-literal token counts: `data["legacy"]` holds `legacy` (AU-1) |
| au_f08 | missing_from naming no bound producer fails (AU-2) |
| au_f09 | a type change placed in no bound producer fails (AU-2) |
| au_f10 | an RS-1 witness whose write lines hold K passes (AU-3) |
| au_f11 | an RS-1 write line lacking K fails (AU-3) |
| au_f12 | CLI: a real change-scoped conviction exits 1 and prints no audit text |

## F. Pre-registered mutation operators

| Operator | Change | Declared killer |
|---|---|---|
| AU-M1 | a missing line passes instead of failing | au_f02 |
| AU-M2 | token match replaced by substring match | au_f05 |
| AU-M3 | `read as` forms ignored | au_f04 |
| AU-M4 | site file name not checked | au_f06 |
| AU-M5 | string-literal contents not tokenized | au_f07b (r1; r0 declared au_f07, see I) |
| AU-M6 | missing_from producer not checked | au_f08 |
| AU-M7 | type-change place not checked | au_f09 |
| AU-M8 | RS-1 write lines not checked | au_f11 |
| AU-M9 | the line-content check removed (any existing line passes) | au_f03 |

Defect injection (proof that the audit is wired into evaluation, not only
correct in isolation): two deliberate witness-builder defects, each run
over the full fixture suite. DI-1: every residue site line off by one.
DI-2: the site file name replaced by the producer's name. Expected: every
fixture that asserts a schema conviction fails with exit 3 and
`self-audit failed`, and no conviction is printed. A DI run where any
conviction is still printed is a failed gate.

## G. Decision points

- AU-D1 (scope). Schema contracts only in r0 (recommended: they carry
  every corpus conviction), or all kinds now.
- AU-D2 (token rule). Whole tokens, identifiers and string-literal
  contents (recommended: measured 239 of 239, and it rejects substrings),
  or name_key-normalized matching.

## H. Bounds

No verdict changes on a correct engine; exit codes unchanged in meaning
(3 already means "could not evaluate, never a verdict"); text output
byte-identical on success; zero new dependencies; frozen spec and
fixtures before code; mutation operators with declared killers; defect
injection; full-corpus byte comparison; native verification.

## I. r1 repair (founder, 2026-10-25, 18:42)

- Lock mode (found by the existing fixture suite on the first full run,
  13 tests). Lock mode shows its residue qualified, `Order.tax`, while the
  consumer line reads `tax`. r0's AU-1 named "the field name" and its
  measurement covered only the corpus, which runs change-scoped. The lock
  path now passes the audit the bound field it matched; the rule is NOT
  loosened (no "anything after a dot", which would widen change-scoped
  matching too). After the change: 513 of 513 tests, corpus unchanged.
- AU-M5 survived r0's run (docs/evidence/audit/mutate-r0.log), and it is
  not equivalent: au_f07's key `legacy` is also an identifier, which the
  identifier scan finds inside the quotes, so literal tokenization mattered
  only for keys that are not identifiers. A throwaway test
  (docs/evidence/audit/au-m5-survivor-probe.txt): for
  `row["template-id"]`, the engine accepts and the mutant wrongly rejects.
  Added fixture au_f07b (tests/audit_r1_test.rs) on exactly that shape; it
  is AU-M5's declared killer. au_f07 stays as a guard.
- The mutation harness runs both fixture files with --no-fail-fast. The
  defect injections DI-1 and DI-2 are rerun unchanged.
