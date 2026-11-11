# A3-ACCEPT: a person's signed acceptance of one receipt (r0)

Drafted 2026-11-10, frozen with the seven `ac_` fixtures in
crates/invar-falsify/tests/accept_test.rs BEFORE implementation; freeze
record: ACCEPT-SPEC.frozen. Second of the three A3 pieces (A3-RECEIPT
shipped as JUDGE-RECEIPT r1 in v0.2.28; A3-GATE follows). Nothing in the
judge changes; an acceptance is written by a person, never by the judge.

## What an acceptance is, and is not

An acceptance is a small file beside a receipt saying: this named person
saw this exact receipt's verdict, listed everything it found, and accepted
it, with a reason. It is attributable (a detached OpenSSH signature by a
key the repository lists) and complete or nothing. It is not a claim the
break is safe, not transferable to other bytes, and never exists for
HOLDS, where there is nothing to accept. The three-state verdict model is
INVAR's own: HOLDS, VIOLATED, NOT EVALUATED, exactly as the receipt
records it; no other vocabulary enters this file.

## The file

    {
      "acceptance_format": 1,
      "accepts": ["VIOLATED"],            // or ["NOT EVALUATED"]; exactly the receipt's verdict
      "principal": "<the allowed-signers principal>",
      "reason": "<why this break is accepted; the person's words>",
      "receipt_blake3": "<the accepted receipt's footer hash>",
      "signed_on": "YYYY-MM-DD",
      "witness_paths": [ ... ]            // everything the receipt found, listed, none implied
    }

witness_paths entries, by receipt kind:
- judge: each witness path; stated findings as "not evaluated: PATH".
- judge-change: the same, prefixed "CONTRACT: " per contract, path order.

The signature is detached OpenSSH beside the file (`<file>.sig`),
namespace `invar-acceptance`, made by the person:
`ssh-keygen -Y sign -n invar-acceptance -f <key> <file>`.

## The four standing rules (an acceptance stands only when all four hold)

1. `receipt_blake3` is the footer hash of the receipt being checked, and
   that receipt itself verifies (`verify-receipt` logic: inputs match,
   body reproduced). New bytes mean a new receipt and a new acceptance;
   nobody inherits an approval for bytes they did not see.
2. The signature verifies for `principal` against the given
   allowed-signers file (the `verify_ssh` the release check already uses).
3. `witness_paths` lists every witness and every stated finding of the
   receipt; an acceptance that omits one accepts nothing.
4. `accepts` names the receipt's verdict, and the verdict is not HOLDS.

## Commands (r0)

- `invar accept RECEIPT --prepare [--out FILE]`: emits the skeleton with
  receipt_blake3, accepts and witness_paths filled from the receipt and
  reason/principal/signed_on empty, for a person to complete and sign.
  Refuses a HOLDS receipt: "nothing to accept", exit 1. An agent may run
  this; it cannot sign.
- `invar accept RECEIPT --acceptance FILE --signers SIGNERS`: checks the
  four rules in order, names the first that fails, exit 0 standing / 1
  not standing / 3 error. Exit 0 output names the principal, the reason
  and the receipt hash, which is the line a CI summary shows.

A3-GATE (next, its own spec) is `--contracts` + receipt + acceptance in
one CI command; this ruling deliberately ships the pieces first.

## Fixtures (frozen before implementation; 0 of 7 pass today)

- ac_f01 round trip: prepare, fill, sign, check: standing, principal named.
- ac_f02 accepts not the verdict: refused; HOLDS receipt: prepare refuses.
- ac_f03 a witness omitted: accepts nothing.
- ac_f04 new receipt for new bytes: the old acceptance accepts nothing.
- ac_f05 unlisted key, or edited after signing: refused on the signature.
- ac_f06 judge-change receipt: accepted as one unit, paths per contract.
- ac_f07 NOT EVALUATED: acceptable; stated paths listed or nothing.

## Mutation operators

- AC-M1 the verdict not compared to accepts   -> ac_f02
- AC-M2 witness completeness not checked      -> ac_f03
- AC-M3 the receipt hash not compared         -> ac_f04
- AC-M4 the signature result ignored          -> ac_f05

## Pre-registered prediction

No corpus moves, no verdict changes anywhere: acceptance reads receipts,
it never judges. Suite, clippy, WG gate and all 124 earlier mutants
unchanged, plus the four above.
