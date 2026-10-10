# A2: evidence receipts (draft for ratification, 2026-10-10)

Status: DRAFT. Nothing here is built. Ratification points are marked RATIFY.
The moat claim this serves (MOAT.md, A2): review is moving to AI, which
cannot promise the same answer twice; INVAR's counterweight is a check whose
inputs, engine and verdict are content-addressed, so anyone can re-run it and
must get the same bytes.

## What a receipt is

One file, `invar-receipt.json`, written by `invar check --receipt <path>`.
It states: these exact input bytes, evaluated by this exact engine version on
this stated date, produced this exact verdict. It is evidence, not a log
line: a third party, or an AI agent, verifies it locally without trusting
the machine that produced it.

## Determinism rules (the whole point; each is testable)

1. Canonical bytes: UTF-8, LF, two-space indent, object keys sorted,
   arrays in deterministic order (contracts by id, artifacts by ref, fields
   by name), no floats, no locale, no map iteration order anywhere.
2. No wall clock inside the hashed body. The evaluation date (which waiver
   arithmetic depends on) is an INPUT, recorded in the body as `as_of`;
   `invar check` fills it from UTC today, `verify-receipt` re-uses the
   recorded value. A receipt verified next year reproduces byte-identically.
3. The body carries no absolute paths, hostnames, usernames or environment.
   Local artifacts are identified by config-relative path and BLAKE3 only.
4. `receipt_blake3` (the file's own hash) is computed over the canonical
   body and stored in a one-line footer object, not inside the body.

## Body fields

- `receipt_format`: 1. Frozen on first release, compatibility promise as
  for lock format 1.
- `engine`: crate version string, plus `rules`: the list of active rule
  generations (A1, D1, E1, F1) so a verifier knows what the verdict means.
  The engine BINARY hash is deliberately absent: it differs per platform,
  and the cross-platform gate requires identical receipts from Linux and
  macOS.
- `as_of`: the evaluation date (rule 2).
- `config`: {path's basename, blake3 of invar.toml bytes}.
- `lock`: {blake3 of invar.lock bytes} or null.
- `artifacts`: every fetched or read input: {ref as written in config,
  bytes, blake3}. This is the existing provenance list, canonicalized.
- `contracts`: per contract: {id, kind, status (holds | holds_waived |
  violated | not_fully_evaluated | skipped | error), residue with read
  sites, unevaluated notes, waivers applied with days_left computed from
  as_of}. Same content as the MCP outcome JSON, canonicalized.
- `result`: the overall verdict and the exit code the run returned.

## Verification

`invar verify-receipt <path>` re-runs the evaluation and byte-compares:

1. Read the receipt; check footer hash over the body.
2. Obtain every artifact by its recorded ref; an artifact whose BLAKE3 does
   not match the receipt is reported as `input drift` and stops verification
   (that is a finding about the world, not about the receipt).
   `--artifacts <dir>` verifies fully offline from a directory of files
   named by BLAKE3; otherwise refs are fetched as `invar check` would.
3. Re-evaluate with the receipt's `as_of`, re-render the canonical body,
   byte-compare.
4. Exit codes, RATIFY: 0 reproduced; 1 not reproduced (verdict or bytes
   differ: the receipt's claim does not hold); 3 error, including an engine
   whose version cannot evaluate this receipt_format; nothing else. 1 here
   does not mean the CONTRACTS are violated; it means the receipt's claim
   failed. A receipt made by a different engine version is exit 3 with a
   plain message, never a silent re-interpretation.

## Signing (reuse, not invention)

The receipt file may carry a detached SSH signature, exactly the release
machinery: `ssh-keygen -Y sign -n invar-receipt`, verified with
`ssh-keygen -Y verify` against an allowed-signers file.
`invar verify-receipt --signers <file>` additionally checks the signature by
shelling out to ssh-keygen (same standing as the curl subprocess; RATIFY).
No Sigstore, no SLSA provenance, no transparency log, no new PKI, no new
crate: the "do not build" list stands, and signing stays optional so the
unsigned path has zero infrastructure. If a design partner later needs
Sigstore, that is a separate ratification with its own cost.

## Gates (from MOAT.md, unchanged)

- A receipt produced on Linux verifies byte-identically on macOS and vice
  versa, for EVERY corpus case (105 + batch 8 = 123 configs). Run in CI on
  both platforms; any diff is a determinism bug and blocks the feature.
- `verify-receipt` on an unmodified receipt: exit 0. On a receipt with one
  flipped verdict bit: exit 1. On a receipt from a tampered artifact:
  input drift reported.

## Non-goals

No server, no upload, no receipt registry, no chain between receipts
(hash-chaining across runs is a ledger feature and the ledger already
exists; RATIFY if wanted later), no timestamping authority, no attestation
of the machine. A receipt proves reproducibility, not honesty of the
producer; that is exactly as much as a local-first tool can prove, and the
docs will say so.

## Build order (when ratified)

1. Canonical JSON writer + receipt emission behind `--receipt` (no new
   deps; serde_json with sorted maps and a custom writer).
2. `verify-receipt` offline path + tests, including the flipped-bit test.
3. Cross-platform CI gate over the corpus.
4. Optional signature verification via ssh-keygen.
