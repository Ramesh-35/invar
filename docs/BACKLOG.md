# The gated backlog (opened 2026-10-28)

What we have deliberately NOT built yet. Everything here is real, written
down, and waiting on a gate. The discipline that produced the engine
governs the roadmap: B1's harvesters were built only after the kill gate
was ratified, judge only after B1 passed. Capability accumulating ahead
of evidence anyone wants it is the zombie pattern wearing a productivity
costume, so every item below names the gate that unlocks it and the
number that would kill it even after unlocking. The PROGRAM-LEDGER's
dates rule; this file holds the depth behind its rows.

## Unlock protocol

- Nothing below is built before its gate fires. A fired gate is recorded
  here with its date and the evidence.
- An item built without its gate firing is reverted, whatever its quality.
- New ideas enter this file with a gate attached, or they do not enter.
- Innovation attaches to a gated item; it never spawns ungated work.

## 1. JUDGE-2: consumers sign what they read from a data contract

What: bind consumer reads to declared data contracts through the existing
signed-lock machinery (SIB-1, unchanged). A consumer repository signs a
lock naming the schema fields it reads; the producer's CI holds every
contract change against those signatures.

Gate: the B2 window passes its kill rule (2 or more external independent
repositories running `invar judge` in an active PR pipeline within 14
days of the v0.2.0 release). Judge adoption proves anyone wants the
producer side; only then does the consumer side deserve code.

Innovation thesis: a cross-organization dependency graph for data where
every edge is a cryptographic signature, not a guess. Today no system on
earth lets Team B's CI refuse Team A's schema change because Team B
signed a statement of what it reads. The B1 corpus shows the coping that
replaces it: hand-written compat maps, both-name fallbacks, shim
packages. JUDGE-2 deletes that entire ritual class.

Kill: if within 30 days of JUDGE-2 shipping no pair of genuinely separate
repositories exchanges a signed data lock, the consumer side folds back
into single-player judge.

## 2. Contract-family readers: OTel semconv, Kubernetes lifecycle, Terraform provider schemas, then ODCS and Avro

What: one reader per declared-contract family, each judging the family's
native ledger format (semconv attribute_map blocks, prerelease-lifecycle
declarations, provider schema.Schema maps, ODCS documents, Avro schemas
in a registry).

Gate, per family: an external user asks with a concrete repository in
hand, or the B2 window passes and the family's own B1 specimens justify
the slice. Families unlock one at a time, never as a batch.

Innovation thesis: the spec-from-specimen method. Every reader's frozen
fixtures are the family's pinned B1 case bytes (as jf_f01/jf_f02 are
b1-001/b1-002), so no reader ever ships against synthetic tests, and the
kill gate that proved the family breaks is the same corpus that proves
the reader catches it. As far as we know, no verification tool has ever
been built this way end to end.

Kill, per family: a reader unused by any external repository 30 days
after shipping is marked dormant in this file and stops receiving work.

## 3. B3: the outreach instrument pointed at data teams

What: the same 40-team written-outreach instrument as A2, aimed at
platform and data-infrastructure teams, offering the judge plus
hold/approve/log on their declared contracts.

Gate: v0.2.0 shipped AND the live judge demo PR open on invar-demo-api
(outreach without a clickable witness is spam). Both land launch night.

Kill (already pre-registered in the ledger): fewer than 2 written
paid-pilot intents within 14 days of first contact kills the data tier.

## 4. A4: the 122-case corpus as a public benchmark

What: publish the corpus (real cross-repository breaks, pinned commits,
per-case manifests, held-out scoring protocol) as the public benchmark
for breaking-change detection, inviting every tool to report recall
against it.

Gate: none on substance (the ledger prices it as packaging only), but
sequenced after launch week so it gets its own attention. 60-day metric:
external citation or use.

Innovation thesis: we publish the benchmark on which our own score is 38
of 70, stated first, before anyone else reports a number. A benchmark
whose author leads with their own misses is unheard of, and it makes the
corpus credible in a way no vendor benchmark can be. The B1 restatement
counts (mean 3.2 coping notations per broken promise) publish with it as
the structural audit of the ritual.

Kill: none; the corpus compounds regardless.

## 5. Receipt transparency: attestations and a public log

What: GitHub artifact attestations and/or Sigstore for release binaries
and judge receipts, so a third party can prove a verdict existed at a
time without trusting us. Deferred while the repository was private; the
repository is public as of 2026-10-28, so the blocker is gone.

Gate: the first external person verifying a receipt or signature chain
(an issue, a question, an A2/B3 pilot that cares), because transparency
infrastructure with zero verifiers is ceremony.

Innovation thesis: verdicts as public, timestamped facts. Combined with
JUDGE-2 this makes "who promised what to whom, and when" a matter of
public record for data contracts, which is the end state of the whole
program: the judge nobody has to trust.

Kill: if a quarter passes with no external verification event, this
stays parked here without shame.

## 6. H2, invar compat: the coping layer compiled from the declared diff

What: a new output command on the existing engine. It reads a producer's
declared rename diff, intersects it with the fields the consumer's
signed invar.lock proves are read, and emits the coping artifacts people
today write by hand (alias tables as OTTL, JSON, TypeScript, Go), plus,
for every change it cannot translate one to one (merges, value changes,
unit changes), an executable falsification test harness instead of a
bare NOT COVERED, and a signed receipt listing both. Source: Brief 4
(docs/research/BRIEF4-REPORT-coping-layer.md) and the founder's
elevation memo (docs/research/H2-ELEVATION-three-breakthroughs.md).

Gate (founder ruling 2026-10-28): B2 clears its kill rule first. Timing
tension, recorded for a founder decision: the dated forcing event is the
OpenTelemetry Java agent 3.0 cutover, targeted October 2026, which may
not wait out B2's full 14 days; a possible amendment is opening H2 on a
strong day-7 B2 signal. Not amended until ruled.

Innovation thesis: the coping layer is the largest unowned, unverified
body of code at every boundary, now increasingly written by agents, and
its input is already data. H2 makes INVAR the deterministic ground truth
agents consult instead of hallucinating fallbacks, and turns every
untranslatable change into a test the hand-written bridge must pass.

Kill (pre-registered in Brief 4): fewer than 3 external events (merged
INVAR-generated artifact, CI run, or third-party receipt verification)
from at least 2 independent organizations by day 14 after release kills
H2 as a product direction; separately, fewer than 1 of 10 platform teams
on the Java agent 3.0 cutover agreeing to a paid or letter-of-intent
pilot by day 14 records H2 as having no willingness to pay.

## Where innovation enters this stream

The unprecedented work is not more features; it is the compounding of
four assets no one else holds together: a pinned corpus of real breaks
across four ecosystems, a falsification discipline the public can
replay, a signed trust chain from release binary to verdict receipt, and
a voice that states what it did not evaluate. Every backlog item above
is one of those assets reinforcing another. Anything that does not is
out of scope, however exciting it looks at midnight.
