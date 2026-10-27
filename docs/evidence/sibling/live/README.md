# Live GitHub evidence for SIB-2 and SIB-3 (2026-10-27)

Captured with `gh` from the demo pair (public repositories), raw in this
directory: pr1.json, pr2-attack.json, runs.json, and the dependents-step
excerpt of each run log. Action: Ramesh-35/invar@main at e33e7f8 (SIB-3
r1). Producer: Ramesh-35/invar-demo-api (src/limits.ts, Limits with
maxBytes and maxLines). Consumer: Ramesh-35/invar-demo-web, whose signed
invar.lock binds both fields; its key is the only entry in the trust root.
Index and trust root live on the producer's main as .invar/index and
.invar/consumer-signers.

## Run 1: the honest break (PR 1, proves SIB-2 live)

PR 1 (head 029f631a) removes Limits.maxLines. The workflow at that point
did its own base-branch extraction and passed ABSOLUTE paths, so this run
proves SIB-2's evaluation and the B-3 pass-through, not SIB-3's own read.
Run 40477039872: completed, success. The github-actions comment is
byte-for-byte the fixtured payload: the row names invar-demo-web's lock,
web@demo, contract limits, `Limits.maxLines absent (locked number)`;
summary 1 broken, 1 provided, 0 not evaluated; the advisory line; the
field-level note.

## Run 2: the attack (PR 2, proves SIB-3's base-branch read live)

main was first switched to plain relative paths (.invar/index,
.invar/consumer-signers) with no extraction step of its own, so the base
read is SIB-3's. PR 2 (head a0a63387) then removes Limits.maxLines AND
deletes the whole .invar tree (index, lock, signature, trust root): it
deletes its judge in the change under review. Run 40477674907: completed,
success. The log shows the checkout at the merge commit of a0a63387 (a
tree with no .invar anywhere), the step receiving the relative paths and
INVAR_BASE_REF=main, and the evaluation reading
$RUNNER_TEMP/invar-base/.invar/index, the step's own extraction of main.
The comment still names `Limits.maxLines absent (locked number)`,
verified as web@demo, 1 broken, 0 lockfiles not evaluated: the deletion
changed nothing. Advisory held: the run is green with the break stated.

## What this does and does not prove

Proven live: SIB-2's evaluation, comment and advisory exit on GitHub's
runners; SIB-3's base-branch read ignoring a pull request that deleted
the index and trust root (B-1, B-2's never-fall-back path not reached,
B-3 pass-through in run 1). Not proven live: the unreadable-base job
failure (B-2 r1, exit 1), fixture-proven only; a forged (rather than
deleted) trust root, fixture b3_f01; behavior on fork pull requests,
where the comment token is read-only. PR 2 stays unmerged so the demo
keeps working.
