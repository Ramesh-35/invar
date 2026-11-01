# JUDGE-RECEIPT r0: a judge receipt anyone can check (r0, FROZEN)

Frozen with crates/invar-falsify/tests/judge_receipt_test.rs BEFORE
implementation; freeze record: JUDGE-RECEIPT-SPEC.frozen. Founder go
2026-11-01, after the capability audit at 38be6bf found:

- A judge receipt's BLAKE3 footer catches accidental edits only. Anyone
  can change the verdict and recompute the footer (demonstrated).
- Judge receipts are not signable or verifiable: `verify-receipt` rejects
  them ("receipt_format None is not 1").
- The receipt names the base as a ref (`main`), which moves, and holds no
  hash of either contract version, so it cannot say what was judged.

No new verdict logic. The judgment is unchanged; only the evidence of it
is made checkable.

## What a judge receipt states

These exact contract bytes at this exact base commit, and these exact
contract bytes in the working tree, judged by this exact engine version,
produced this exact result. Nothing more: not that the change merged,
not who authored it, not that the base was the right base to judge
against (that is the repository's branch protection, below).

## Body (canonical, sorted keys, no wall clock)

    {
      "engine": {"version": "<engine version>"},
      "inputs": {
        "base_commit": "<40-hex commit the base ref resolved to, or null>",
        "base_contract_blake3": "<blake3 of the base contract bytes, or null>",
        "base_ref": "<the --base ref as given>",
        "contract": "<contract path>",
        "post_contract_blake3": "<blake3 of the working-tree contract bytes>"
      },
      "receipt_format": 1,
      "receipt_kind": "judge",
      "result": <exactly the --format json document>
    }

followed by the existing footer line `{"receipt_blake3":"<hex>"}`.

- `result` is byte-for-byte the JSON surface frozen in JUDGE-SPEC r1, so
  that surface does not change. Text, JSON, SARIF and GitHub outputs are
  unchanged.
- `base_commit` is null only when the base ref does not resolve
  (unreadable_base). `base_contract_blake3` is null when there is no base
  contract (new_contract, unreadable_base).
- Check receipts carry no `receipt_kind` and keep their verification path
  unchanged.

## Signing (no keys in INVAR)

INVAR never holds a key. The signer signs the receipt file with the
existing namespace:

    ssh-keygen -Y sign -n invar-receipt -f <key> receipt.json

This produces `receipt.json.sig`. `invar verify-receipt receipt.json
--signers <allowed_signers> --principal <id>` checks it, as it already
does for check receipts.

## Verification (`invar verify-receipt` on a judge receipt)

Run in the repository, with the base commit fetched. In order:

1. The footer must match the body. A mismatch is a corrupted file: exit 3,
   never a verdict (unchanged).
2. `receipt_format` must be 1 and the engine version must be this engine's
   (unchanged: reproduction is promised by the same version only).
3. With `--signers`, the detached signature must verify. A missing or
   failing signature: exit 1, "receipt NOT verified".
4. The working-tree file at `inputs.contract` must hash to
   `post_contract_blake3`. Otherwise: exit 1, input drift.
5. The base contract is read from `inputs.base_commit` (the commit, never
   `base_ref`, which can move) and must hash to `base_contract_blake3`.
   Otherwise: exit 1, input drift. A commit that is not in the repository
   is exit 3 ("fetch it"), never a verdict.
6. The judgment is re-run on those bytes, labelled with the recorded
   `base_ref`, and the re-derived body must equal the recorded body byte
   for byte. Otherwise: exit 1, "receipt NOT reproduced".
7. Exit 0: "receipt reproduced", with "signature ok (<principal>)" when
   step 3 ran.

A receipt with no `receipt_format` (judge receipts from v0.2.0 to v0.2.2)
remains exit 3, stated.

## What closes the commit-time gap (outside INVAR, documented)

A receipt says what was judged. Whether the judged base is still the base
when the PR merges is GitHub's to enforce, not INVAR's: make the judge
check required under branch protection and turn on "Require branches to
be up to date before merging", or use a merge queue. Both block a merge
until the judge has passed against the current base. The README states
this.

## Fixtures (judge_receipt_test.rs)

- jr_f01 shape: kind judge, format 1, `base_commit` equals `git rev-parse`
  of the base, both hashes equal BLAKE3 of the real bytes, `result` equals
  the `--format json` output.
- jr_f02 a fresh receipt verifies: exit 0, "reproduced".
- jr_f03 FM-FORGERY-1, the audit's forgery: verdict edited, footer
  recomputed. Exit 1 (kills RM-1).
- jr_f04 signed: verifies with `--signers` (exit 0). The forged body under
  the original signature fails (exit 1).
- jr_f05 base pinned: the base branch moves after the receipt. Still exit
  0, because the base is read from the recorded commit (kills RM-2).
- jr_f06 post substitution: the working-tree contract changes. Exit 1,
  input drift (kills RM-3).
- jr_f07 base hash forged in the receipt, footer recomputed. Exit 1 (kills
  RM-4).
- jr_f08 `base_commit` not in the repository. Exit 3.
- jr_f09 a v0.2.x receipt without `receipt_format`. Exit 3.
- jr_f10 two receipts on the same inputs are byte-identical.

## Mutation operators

- RM-1 re-derived body not compared (always "reproduced"): killed by
  jr_f03.
- RM-2 base read from `base_ref` instead of `base_commit`: killed by
  jr_f05.
- RM-3 post contract hash not checked: killed by jr_f06.
- RM-4 base contract hash not checked: killed by jr_f07.
