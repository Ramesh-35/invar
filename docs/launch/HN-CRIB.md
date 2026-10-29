# HN reply crib (launch night)

Discipline: answer the sharpest version of the question, concede what is
true, link the file that proves the claim, never a number we have not
measured. No exclamation marks. If a critique is correct, say so and say
what it changes.

## 1. "It misses 46% of real breaks. Why would I use it?"

That number is ours, published on purpose: 38 of 70 held-out real
cross-repository breaks, and our own detection-reliability hypothesis is
recorded as FALSIFIED against the 20% miss gate we set ourselves. So we
do not sell detection. What you get even where it misses: anything not
evaluated is stated with a reason code instead of silently passing, the
lock records exactly what you accepted and when, and every verdict is
reproducible from a signed receipt. A separate command, invar judge,
covers declared data contracts (JSON Schema); on the ten pinned real
breaks in our public corpus it caught ten, because every admitted real
incident there was a field removal or a retype, and what it does not
judge (enum, pattern, bounds, required) is printed in every output.
Proof: README "What INVAR does not do", docs/epistemic/STATUS.md,
docs/falsification/b1/.

## 2. "How is this different from Pact / contract testing?"

Different layer. Pact is consumer-driven contract TESTING: both sides
run tests, usually with a broker, and it is strong where you can do
that. INVAR is one static binary that diffs declared surfaces and
signed consumer locks at the pull request: no runtime, no broker, no
mocks, and the base branch supplies the trust root so a PR cannot
choose its own judge. They can coexist; INVAR's job is the moment of
change, not the test suite. Proof: action.yml (the whole integration),
docs/falsification/SIBLING-BASE-SPEC.md.

## 3. "The lockfile is self-reported. Why trust it?"

Correct, and the receipt says so: a lock records what a consumer
DECLARES it reads, so a consumer can under- or over-declare. What the
signature buys is attribution and tamper-evidence, not truth: you know
who said it, that it has not changed since, and that the index a PR is
judged against comes from the base branch, never from the PR. Watch a
pull request delete the entire index and trust root in its own diff and
still get held against main's signed locks:
https://github.com/Ramesh-35/invar-demo-api/pull/2.

## 4. "One person plus AI agents built this. Why trust the tests?"

Do not trust them; run them. Specs and fixtures are frozen and hashed
BEFORE implementation (docs/falsification/JUDGE-SPEC.frozen), and every
feature ships with named mutation operators whose killers you can
replay: bash docs/falsification/judge-mutations.sh prints six mutants
dying on your machine. The release binaries verify against a committed
trust anchor (docs/reference/RELEASE-SIGNING.md). The corpus the recall
number comes from is in docs/falsification/ with per-case manifests.

## 5. "Who pays? What is the business model?"

Today, nobody, and that is written down rather than hidden: every
commercial hypothesis lives in docs/PROGRAM-LEDGER.md with a dated,
pre-registered kill rule, and a track that misses its number is
recorded dead in that file with its final count. This launch starts one
of those windows: if fewer than 2 external independent repositories run
invar judge in an active PR pipeline within 14 days of v0.2.0, the
standalone data slice dies as pre-committed. We measure instead of
promising.
