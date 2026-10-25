# AGENT-1: what an AI coding agent can rely on, made true and tested

Status: r0, RULED 2026-10-25 (AG-D1 to AG-D4 as recommended). P2 (self-audit) landed 2026-10-25: docs/falsification/AUDIT-SPEC.md, docs/evidence/audit. P1, P4, P3 follow in that order. Parent: the founder's
directive of 2026-10-25 (build the three agent incentives with first-class
quality) and the standing ruling that agent integration is a proposal, not
a product claim, until it is built and measured. Baseline: 9e89f33.

The three incentives, as first written, each contain one sentence that
cannot be made true. Each is restated below as a property INVAR can
guarantee, prove with fixtures, and measure on real repositories. The
restated forms are what any external text may say.

## A. The three properties

### P1. A localized witness an agent can act on without guessing

As written: "an explicit, localized structural witness". True today in
substance, not yet in form. Measured over the corpus (64 VIOLATED cases,
126 witness lines, 238 cited consumer sites): every conviction names its
consumer read sites except RS-1 (1 line), whose sites live inside its
message text. The MCP and JSON output carry each witness as strings
(`"read_at": ["file:line", ...]`, `"missing_from": "<text>"`), and every
NOT EVALUATED item as one string with its reason code inside brackets.

Build:
- Each residue item gains structured fields: `rule` (which rule convicted:
  removed, retyped, relocated, request-side, ...), `consumer_sites` as
  `{file, line}` objects, `producer` (`{file}`, plus a line where the rule
  has one), `read_as`, and `candidate_rename` (G2-R) as its own field.
- Each NOT EVALUATED item becomes `{reason, text}`, `reason` the stable
  lowercase code (`default-switch`, `opaque-emission`, ...), so an agent
  branches on the code, never on prose.
- The text output is unchanged byte for byte; only the JSON gains fields
  (additive, existing keys kept), so no existing consumer of the JSON
  breaks.

### P2. Convictions any agent can re-verify (replaces "zero false positives")

As written: "The Zero-False-Positive Guarantee ... When INVAR says
VIOLATED, it is mathematically airtight." Not true and not claimable: the
grammars are text heuristics (Class 4 in the evidence register), and the
evidence is measured precision on a corpus with 15 held-out controls, not
a proof. The record itself contains a counterexample in kind: r1-116
convicts `temperature` on `SensorDeviceClass.TEMPERATURE`, a constant, not
a field read (the case is a real break, convicted on the wrong field).

What can be made true, and is stronger in practice: every conviction is
RE-VERIFIABLE from the exact bytes it cites.
- Audit, per witness: each cited consumer line, re-read from the
  blake3-pinned bytes INVAR fetched, contains the field name or a form the
  witness itself declares under `read as`; and the producer bytes at head,
  re-read the same way, still lack the field under the rule that
  convicted. A conviction that fails its own audit is an engine defect and
  is reported as an error (exit 3), never shown as VIOLATED.
- Measured today, by a throwaway audit over the corpus: 238 of 238 cited
  sites contain the field or a declared `read as` form. The audit checks
  INVAR against its own evidence; it does not claim the read is
  semantically a read of that field (r1-116 passes it, and that is stated).
- Precision stays a measured, published number with its denominator (the
  held-out record), never a guarantee.
- r1-116's constant-as-read is a separate precision finding, to be
  measured on its own (how many convictions rest only on ALL_CAPS member
  reads) before any rule is proposed.

External form: "Every VIOLATED names the lines it read and the producer
bytes it checked, pinned by hash, so you can verify it yourself. When the
evidence is not enough, INVAR says NOT EVALUATED and why."

### P3. Fast enough to run on every agent step, measured and held

As written: "an instant, sub-4-millisecond verification". Measured
2026-10-25, sandbox, process start included, 15 runs each:

| Repository | Contracts | Full evaluation p50 | Reuse from lock p50 |
|---|---|---|---|
| Understand-Anything | 12 | 11.4 ms | 3.7 ms |
| excalidraw | 28 | 45.2 ms | 5.0 ms |
| pi | 108 | 56.1 ms | 8.4 ms |

Sub-4 ms is true only for a reused check on a small repository. Build:
- A latency record per release on these three repositories (full and
  reuse, p50 and max), committed as evidence, and a regression gate that
  fails the build when a p50 moves past a declared budget (CHECK-PERF-1
  already proves memoization equivalence; this adds the time bound).
- External form: "single-digit milliseconds when nothing changed; tens of
  milliseconds for a full evaluation of a hundred contracts", with the
  record linked.

### P4. invar.lock as the agent's durable memory of contracts

As written: "a cryptographic long-term memory for AI sessions ... a
deterministic anchor". True in mechanism today: each locked contract pins
its producer by blake3, records what was bound and when, and HOLDS
records pin every input byte; the lock can be signed and verified
(`--lock-signers`); the MCP `contract` tool already reads it with no
network. "Cryptographic" holds only for a signed lock; an unsigned lock is
hash-pinned, not authenticated.

Build: nothing new is required for the mechanism. The improvement is
disclosure: the MCP `contract` and `check` results state whether the lock
in use was signature-verified in that run (`lock_verified: true|false`),
so an agent never treats an unsigned lock as authenticated memory.

## B. Order

1. P2 audit first: it is the property the other three rest on, and it can
   fail closed today without changing any verdict (measured 238/238).
2. P1 structured JSON (additive).
3. P4 lock disclosure (one field).
4. P3 latency record and gate.
Each with frozen fixtures before code, mutation operators with declared
killers, full-corpus byte comparison of the text output, native
verification. G2 (pending ruling) lands first; AGENT-1 does not touch the
detection rules.

## C. Decision points

- AG-D1 (claims). Adopt the restated external forms in A, and retire
  "zero false positives", "mathematically airtight" and "sub-4 ms" from
  all external text (recommended: each is falsified by the record).
- AG-D2 (audit failure). A conviction that fails its own audit is exit 3,
  an engine error (recommended: fail closed, never a silent VIOLATED), or
  it is demoted to NOT EVALUATED with the audit failure stated.
- AG-D3 (scope and order). P2, P1, P4, P3 as one arc after G2
  (recommended), or a subset.
- AG-D4 (r1-116). Measure constant-as-read convictions as a separate
  precision finding before any rule (recommended).
