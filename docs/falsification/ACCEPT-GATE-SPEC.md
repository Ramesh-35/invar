# A3-GATE: the one CI command (r0)

Drafted 2026-11-11, frozen with the seven `ag_` fixtures in
crates/invar-falsify/tests/accept_gate_test.rs BEFORE implementation;
freeze record: ACCEPT-GATE-SPEC.frozen. Third and last A3 piece
(A3-RECEIPT v0.2.28, A3-ACCEPT v0.2.30). The judge does not change: same
verdicts, same output, same receipts. The gate only decides the exit code.

## The command

    invar judge --contracts GLOB --base REF --acceptances DIR --signers FILE

Exit 0 only when the change HOLDS, or when a person listed in FILE signed
for exactly this change. Otherwise the judge's own exit code (1 VIOLATED,
4 NOT EVALUATED, 3 error), plus one line saying how to accept.

The older `invar gate` (config, proof, wrapped command) is a different
tool and keeps its name; this is two flags on `judge`, not a new command.

## What is in DIR

For each accepted change, three committed files with one stem:

    DIR/<name>.receipt.json          invar judge --contracts ... --receipt
    DIR/<name>.acceptance.json       invar accept ... --prepare, completed by a person
    DIR/<name>.acceptance.json.sig   ssh-keygen -Y sign -n invar-acceptance

## When an acceptance stands at the gate

1. Judged identity. The committed receipt and the judgment the gate just
   made agree on everything except the two commit fields: the same
   contracts, the same pre and post hashes, the same verdict and findings.
   Compared as the receipt body with `base_commit` and `head_commit`
   removed.
2. The acceptance names that committed receipt's footer hash.
3. A3-ACCEPT rules 2, 3 and 4, unchanged: signature by a listed
   principal, every finding listed, verdict named and not HOLDS.

Rule 1 is the one design decision here. A receipt's footer hash covers
the commit fields, and committing the acceptance moves HEAD, so a gate
that demanded the same footer hash could never be satisfied from inside
the pull request it gates. The gate therefore matches on the judged
bytes and result, and commits are not part of what a person accepts.
"Nobody inherits an approval for bytes they did not see" holds: change
any contract byte, or let the base contract move, and the identity
differs and the acceptance accepts nothing (ag_f03).

## Output

The judge's output first, unchanged. Then, when the verdict is not HOLDS:

    gate: accepted by <principal>: <reason> (receipt <hash>)

or

    gate: no standing acceptance
    gate: <name>: <the first rule that failed>        (one per candidate)
    gate: to accept: invar judge --contracts ... --receipt R, then
          invar accept R --prepare, complete, sign, commit under DIR

Without `--acceptances` nothing changes (ag_f07). `--acceptances` and
`--signers` are given together or it is a usage error.

## What the gate does not establish

- That the break is safe. It establishes that a listed person signed for
  these exact bytes with a stated reason.
- That the signers file is the right one. A pull request can edit files
  in its own tree, so CI must take FILE from the base, for example
  `git show origin/main:.invar/signers > "$RUNNER_TEMP/signers"`.
- Anything about HOLDS it did not already establish: HOLDS carries the
  judge's stated limits and needs no acceptance.

## Fixtures (frozen before implementation; 1 of 7 passes today, the guard)

- ag_f01 unaccepted break: exit 1, "no standing acceptance", how to accept.
- ag_f02 signed acceptance committed on the branch: exit 0, principal and
  reason named, verdict still stated. Proves commit independence.
- ag_f03 one more change after acceptance: exit 1.
- ag_f04 HOLDS: exit 0, no gate complaint.
- ag_f05 signature by an unlisted key: exit 1, signature named.
- ag_f06 NOT EVALUATED: exit 4 until accepted, then 0.
- ag_f07 guard: no flags, no change.

## Mutants

- AG-M1 judged identity not compared            -> ag_f03
- AG-M2 signature not checked at the gate       -> ag_f05
- AG-M3 gate passes with no acceptance          -> ag_f01

## Pre-registered

7 of 7; AG-M1..M3 killed with all 128 earlier mutants (131); no corpus
moves; WG gate 76 pairs 0 HOLDS unchanged; full suite; clippy silent;
zero new Rust dependencies.
