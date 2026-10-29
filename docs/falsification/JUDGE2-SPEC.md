# JUDGE-2: consumers sign what they read from a data contract (r0 DRAFT, NOT FROZEN)

Gated in BACKLOG.md: no code before B2 clears its kill rule. SIB-1
machinery is reused unchanged; this is a new input to an existing judge,
not a new engine.

## Job, one sentence

Hold a producer's declared data-contract change against the signed locks
of the consumers who read it, so the verdict names not only the broken
field but the signed parties it breaks.

## Interface (draft)

    invar judge --contract <path> [--base <ref>]
                [--consumers <locks-dir> --lock-signers <file>]

- Without `--consumers`: JUDGE-1 exactly as frozen, untouched.
- With `--consumers`: each signed lock naming this contract is
  intersected with the witnesses. Every witness gains the list of
  signed consumers bound to that field. Unsigned or unverified locks
  are stated, never read (SB-3 unchanged).
- Exit codes unchanged. A witness with zero bound consumers is still
  VIOLATED (the promise broke); consumers add weight, never gate.

## What the lock binds for data contracts (J2-D1, the one new design question)

Code locks bind Type.field surfaces. A data-contract lock binds dotted
schema paths (payload.user.zipcode) plus the declared type at lock time.
Draft answer: the existing lock payload-shape machinery already records
dotted paths; a data contract is locked with `invar lock` pointed at the
schema file, no new format. To ratify or amend.

## Decision points (J2-D1 to J2-D3)

- J2-D1 lock format: reuse payload shapes as above, or a dedicated
  schema-lock section?
- J2-D2 voice: witness line gains "read by 2 signed consumers
  (consumer-a, consumer-b)" when consumers are bound. Exact phrasing to
  freeze with fixtures.
- J2-D3 fixtures: jf2 fixtures derive from b1-001 bytes plus two
  synthetic consumer locks signed with the test key, mirroring
  sibling_eval_test.rs arrangements. One mutation operator per new code
  path, each with a named killer, as always.

## Kill (BACKLOG.md, carried verbatim)

If within 30 days of JUDGE-2 shipping no pair of genuinely separate
repositories exchanges a signed data lock, the consumer side folds back
into single-player judge.
