# JUDGE-RECEIPT r1 (A3-RECEIPT): one receipt per judged change (r1, DRAFT)

Drafted 2026-11-08 for ratification. Frozen with the `jr1_` fixtures
appended to crates/invar-falsify/tests/judge_receipt_test.rs BEFORE
implementation; freeze record: JUDGE-RECEIPT-R1-SPEC.frozen. Extends
JUDGE-RECEIPT r0. No verdict logic changes; nothing in the walk changes.

## Why

r0 gives one receipt per contract (`judge --contract C --receipt`), and
`--receipt` with `--contracts` is refused. A pull request touches a set of
contracts and is accepted or blocked as one unit, so the record a person
signs (A3-ACCEPT, next) must be one file naming every contract the change
touched, each by the bytes judged. r0 also names no head commit: the
receipt says "the working tree", which nobody can re-read later. These
are the two gaps between a receipt that reproduces and a record that can
be accepted and audited across repositories, which is the Tier 2 product.

## What a change receipt states

These exact contract bytes, each at this exact base commit and each as
these exact head bytes, judged by this exact engine version, produced
these exact results. The head commit is named only when the working tree
bytes of every judged contract are the bytes of that commit; otherwise it
is null and the receipt says so. Nothing more: not that the change
merged, not who authored it, not that any validator confirmed a
counterexample (the Rust judge runs none, and writes none).

## Body (canonical, sorted keys, no wall clock, no absolute paths)

    {
      "engine": {"version": "<engine version>"},
      "inputs": {
        "base_commit": "<40-hex, or null>",
        "base_ref": "<the --base ref as given>",
        "contracts": [
          {"base_contract_blake3": "<hex or null>", "contract": "<path>", "post_contract_blake3": "<hex>"},
          ...                                   (sorted by contract path; only contracts the change touched)
        ],
        "head_commit": "<40-hex when every judged contract's working-tree bytes equal HEAD's, else null>"
      },
      "receipt_format": 1,
      "receipt_kind": "judge-change",
      "result": <exactly the `judge --contracts ... --format json` document>
    }

followed by the r0 footer line `{"receipt_blake3":"<hex>"}`. Single-contract
receipts (`receipt_kind: "judge"`) are unchanged byte for byte: r0's
fixtures stay green and jr_f10 (byte identity) still holds.

## Writing

`invar judge --contracts <glob> [--contracts ...] --base <ref> --receipt <path>`
writes one change receipt. The contracts listed are exactly those the
`--contracts` run judged (changed under the globs); an unchanged contract
is not an input. Exit code is the run's own, unchanged.

## Verifying (`invar verify-receipt` on a change receipt)

As r0, per contract, in order: footer; format and engine version;
optional signature (`-n invar-receipt`, unchanged namespace); for every
listed contract, the working-tree bytes must hash to its
`post_contract_blake3` and the bytes at `base_commit` to its
`base_contract_blake3` (drift: exit 1, naming the contract); then the
whole `--contracts` judgment is re-run on those bytes with the recorded
`base_ref` label and the re-derived body must equal the recorded body
byte for byte. `head_commit` is recorded, not re-derived: a later
checkout of another commit with identical contract bytes still
reproduces. A base commit missing from the repository is exit 3.

## What is deliberately absent

- No harness (tests.py) result inside the receipt: validator versions
  are not pinned by the engine version, so their answers are not
  reproducible by `verify-receipt`; the harness JSON can sit beside the
  receipt, labelled as what it is.
- No rules list: the engine version names the rulings in force (the
  specs in docs/falsification are the list).
- No wall clock, no author, no hostname, no path above the repository.

## Fixtures (frozen before implementation)

- jr1_f01 `--contracts` with two touched contracts writes a receipt with
  `receipt_kind` "judge-change", both contracts listed in path order with
  both hashes each, `result` equal to the `--format json` document.
- jr1_f02 a fresh change receipt verifies: exit 0, "reproduced".
- jr1_f03 editing one listed contract after writing: exit 1, drift naming
  that contract.
- jr1_f04 `head_commit` is HEAD's hash when the judged contracts are
  committed, and null when one is dirty in the working tree.
- jr1_f05 guard: a single-contract receipt is byte-identical to r0's
  (`receipt_kind` "judge", no `head_commit`, no `contracts`).
- jr1_f06 a forged verdict in a change receipt with a recomputed footer
  fails verification: exit 1.

## Mutation operators

- RM1-1 `head_commit` written as HEAD even when a contract is dirty -> jr1_f04
- RM1-2 one contract's post hash not checked on verify               -> jr1_f03
- RM1-3 re-derived body not compared for change receipts             -> jr1_f06

## Pre-registered prediction

No corpus moves: receipts record, they do not judge. Suite, clippy and
mutants as before, plus the three above.

## Measured outcome (2026-11-09, after implementation)

Freeze rebased byte-identically onto v0.2.27 (both .frozen hashes verify;
CC r0 touched neither file). Measured on the frozen tree: 1 of 6 (the
guard). After implementation: 6 of 6, and the ten r0 fixtures untouched
and green, so single-contract receipts are byte-identical. RM1-1, RM1-2,
RM1-3 killed. One detail the spec shape left open is now stated: a
contract DELETED by the change has no head bytes, so its
`post_contract_blake3` is null, and verification requires the file to be
absent; nothing is hashed as empty. The recorded `globs` and
`summary.unchanged` are the write-time selection, carried as recorded;
every verdict, witness and summary count is re-derived on verify. No
corpus moves: receipts record, they do not judge.
