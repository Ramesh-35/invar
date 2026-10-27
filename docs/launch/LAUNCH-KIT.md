# Launch kit (v0.1.0)

## Release checklist (day 2)

Run from the repository root, each line on its own:

```
git status -sb
cargo test --workspace 2>&1 | grep "test result" | awk '{p+=$4; f+=$6} END {print "passed", p, "failed", f}'
git tag -a v0.1.0 -m "INVAR v0.1.0"
git push origin v0.1.0
```

Then watch the release workflow, and verify one asset as a stranger would:
download the Linux binary and its .sig from the release page, and check it
against docs/reference/RELEASE-SIGNING.md's instructions. The
reproducible-build job must be green on the tag. If anything fails, delete
the tag (`git tag -d v0.1.0`, `git push origin :refs/tags/v0.1.0`), fix,
re-tag.

## Show HN draft (day 3; post as yourself, edit freely)

Title: Show HN: INVAR, a breaking-change firewall that states what it
could not check

Body:

I build INVAR solo (with AI agents doing the implementation under a
falsification discipline: specs and test fixtures are frozen and hashed
before any code, and every change must kill a set of pre-registered
mutants). It checks the promises between repositories: the fields your
code reads from another repository's API or payloads. One static Rust
binary, local-first, no server, no account.

The part I care most about: a skip is never a pass. Anything INVAR could
not evaluate is stated, with a reason code, and the exit code says so. The
honest numbers are in the README: on a corpus of 122 real cross-repository
breaks it catches 38 of 70 held-out cases, our own detection-reliability
hypothesis is recorded as FALSIFIED against our 20% miss gate, and every
release note says so. What it adds even where it misses: the lock records
exactly what you accepted and when, every verdict is reproducible from a
signed receipt, and silence never claims safety.

What might interest HN most is the trust chain, and it is live: consumer
repositories sign a lockfile of the fields they depend on; a producer's CI
evaluates those signed locks against every pull request and comments with
the exact broken field; and on pull requests the index and the trust root
are read from the base branch, never from the PR. Here is a real PR that
deleted the entire index and trust root in its own diff and was still
held against main's signed locks:
https://github.com/Ramesh-35/invar-demo-api/pull/2

There is also a Claude Code hook (one settings entry) that feeds the
witness of a break back into the agent's context right after the edit
that caused it, and an MCP server whose verdicts state whether the lock
they relied on was signature-verified.

Repository: https://github.com/Ramesh-35/invar
The unvarnished evidence record: docs/epistemic/STATUS.md

I would rather you attack the claims than star the repository; every
claim links to the evidence that should let you do it.

## Claims discipline for every public word

Banned, as in the docs: "zero false positives", "mathematically
airtight/proven/closed", "sub-4 ms", "sub-millisecond", "tamper-proof,
trust unconditionally". The recall is 38 of 70, stated, never softened.
