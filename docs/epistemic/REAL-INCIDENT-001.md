# REAL-INCIDENT-001 - First REAL-tier evaluation

INVAR's first evaluation of a genuine cross-repository incident. Recorded in
full so the verdict can be re-derived and audited. The verdict was produced by
`evaluator::classify`, not asserted by a human.

## Externality (how we know the artifacts are real)

- Producer: `github.com/Ramesh-35/invar-demo-producer` @ `7c0855709970092b4f913093f0b57a70923e0b1a`
- Consumer: `github.com/Ramesh-35/invar-demo-consumer` @ `fe73889bf250275711339d065ecc37f21879fc72`
- Both files served HTTP 200 from `raw.githubusercontent.com` at those exact
  commit SHAs. Negative control: a bogus all-zero SHA returned HTTP 404, so a
  200 means the commit genuinely resolves.
- Artifact byte identity (sha256):
  - producer `schema.json`: `cb245016cff507a3cdfc9ca49d53d9799be2585e919ad5cfb7b20a8c3f727e5c`
  - consumer `client.js`:  `934d062e043b0fdd89c65713b1ad8b13171d6200045f8cff1be345f003db244d`

LIMIT: externality was established by an audited out-of-band fetch (curl via the
HTTPS proxy) with a negative control, NOT yet by in-process verification inside
the Rust binary. The GitHub commit API was gated for this session, so commit
author/timestamps were not retrievable.

## Evidence

- Producer `schema.json` DEFINES properties `{order_id, amount_micro_units}`.
  Its `required` array still names `amount_cents`, a field it no longer defines
  (an internal inconsistency, evidence of a rename/removal).
- Consumer `client.js` reads `payload.amount_cents` and throws without it.

## Differential probe (INVAR)

`reality::differential_residue` = consumer-required minus producer-defined
= `["amount_cents"]`. `reality::derive_probe_hash` (canonical BLAKE3) =
`829547e08158a232667735aafbc41ae2b1919fb822f8f55b83905ec88a045ab5`.

## Composite baseline (the tools INVAR is measured against)

semgrep 1.179.0 executed on each repo in isolation with a local ruleset
(never `--config auto`):
- consumer scan: 2 advisory findings (`payload.amount_cents` read at lines 3, 6).
- producer scan: 1 advisory finding (token `amount_cents` in the schema).
None isolates the cross-repository contract break: a single-repo tool sees a
token in one repo but cannot adjudicate that the consumer requires a field the
producer no longer defines. So `caught_designated = false`. OPA, OpenTelemetry,
and property-testing legs were SKIPPED (not executable offline).

## Verdict (produced by classify)

`PROVEN_RESIDUE` at REAL tier. Every required leg (semgrep) executed and none
isolated the designated failure; the differential probe isolated the residue;
evidence tier REAL carried no FIXTURE "candidate only" qualifier. Persisted to a
bi-temporal BLAKE3 ledger; the node re-verified. Reproduce with:
`cargo run --example real_incident -- <producer schema.json> <consumer client.js>`.
NOTE: valid-time was recorded as the evaluation time because commit timestamps
were not retrievable.

## Ruling: what this does and does not establish

EARNS the first bare `[PROVEN]` for ONE specific claim: on REAL-INCIDENT-001,
INVAR isolates a real cross-repository field-contract residue that an executed
single-repo composite baseline does not.

Does NOT establish:
- Overall mathematical adequacy. Primitive A stays PARTIAL; this incident did
  not mislabel anything, so A's real-case dependency is unresolved. Overall
  remains NOT ESTABLISHED.
- A wild or consequential defect. The two repos are a constructed demonstration
  (`invar-demo-*`), so this proves the MECHANISM on real external artifacts, not
  that INVAR found a production defect that mattered to someone.
- Self-contained externality. The fetch and its control were an audited shell
  step, not in-process Rust verification.
- A multi-tool baseline. Only semgrep (the one required leg for this failure
  type) executed; a broader baseline would harden the "composite missed it"
  claim.

## Addendum 2026-10-06: in-process re-resolution

`invar check --ci` (commit `8830086`) fetched the consumer artifact itself,
inside the binary, through the curl transport:
`github:Ramesh-35/invar-demo-consumer@fe73889bf250275711339d065ecc37f21879fc72:client.js`,
229 bytes, BLAKE3 `ecd0a60aa8fd380e574fae70e75810da57f3b43c744ec03d492bc3b21c576f69`.
An independent BLAKE3 over the out-of-band copy fetched 2026-10-04 is
identical, so both paths saw the same bytes. Contract VIOLATED on
`amount_cents`; a bogus-SHA control returned 404 and exited 3.

This removes the "externality only out-of-band" limit for the `--ci` path.
Unchanged: a constructed demo, a single incident, and `check` does not run the
reality gate or the composite baseline, so this run is a contract violation,
not a new PROVEN_RESIDUE verdict.
