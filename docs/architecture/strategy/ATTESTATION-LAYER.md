# The Attestation Layer: Integration Contracts as Supply-Chain Claims

Status: SHELVED 2026-10-12, hours after drafting, on the falsification
pass this draft itself commissioned (docs/falsification/
attestation-fpass-2026-10-12.md). Shelved, not deleted: the record keeps
its reasoning, per doctrine. What killed it as a first move: the
best-evidenced consumer (agents in PRs) needs a receipt and an exit code,
not an envelope; nothing measurable consumes custom predicates today;
no regulation pulls on it; and the registry's generic Test Result /
Simple Verification Result predicates could carry a verdict in an
afternoon if ever needed, so the slot is not scarce.

Re-entry condition (binding, from the pass's own verdict): a NAMED
consumer asks for the envelope: a team, platform, or tool that states it
will verify or gate on an INVAR attestation. On that ask, emission is a
small addition (wrap the existing receipt; the pass confirmed cosign and
actions/attest carry custom predicate URIs). No predicate PR is filed
upstream before that ask exists. The pass's three-week dual-arm
experiment (receipt-as-PR-check vs receipt-plus-attestation) is the
standing cheapest test if we ever want to probe demand ourselves.

The draft below is preserved as written, before the pass.
Every claim below carries its evidence rung. Nothing here is measured INVAR
performance; the measured record lives where it always lives.

## 1. The gap (measured 2026-10-12, one probe; verify before relying)

The software supply-chain trust stack answers two questions today:

- What is inside this artifact? (SBOM: SPDX, CycloneDX)
- Who built it, from what, how? (provenance: SLSA, sigstore, GitHub
  Artifact Attestations)

The in-toto attestation predicate registry (spec v1.0.2) defines ten
predicate types: SLSA Provenance, Link, SCAI Report, Runtime Traces, SLSA
Verification Summary, SPDX, CycloneDX, Vulnerability, Release, Test Result.

None of them answers the third question: does this artifact still honor
the contracts its consumers depend on? The closest are Runtime Traces
(runtime data, not producer-consumer) and Test Result (generic outcomes).
There is no predicate for cross-repository integration contracts.

Rung: measured, single web probe of the v1.0.2 registry on 2026-10-12.
The registry may have grown since. The supplementary research prompt
(Section F1) exists to re-verify this before we commit.

## 2. The thesis (hypothesis, stated before any build)

One line: SBOM says what is inside. Provenance says who built it. INVAR
says whether it still fits.

The INVAR receipt is already the right shape for this layer: a
deterministic, replayable record that a specific engine, at a specific
commit pair, evaluated specific declared field dependencies and reached a
specific outcome with witnesses. Wrapped in an in-toto Statement envelope,
that receipt becomes an attestation predicate: the integration-contract
layer of a stack that currently stops at build provenance.

This extends, and does not replace, the locked declaration-graph thesis.
Locks are the claims. The dependents index is the publication. The
attestation is the verification event, made portable into infrastructure
other people already run.

## 3. Why ride existing rails instead of building a log

The survival doctrine already locked: become the format, stay neutral,
keep the honest record. Certificate Transparency is the scoped precedent
for the record half: CT did not make certificates secure, it made
misissuance publicly visible and undeniable, and its force came from
browsers mandating it. We have no mandator. So we do not build a log.
sigstore (Rekor), GUAC, and GitHub Artifact Attestations are
already-running, neutrally-governed or platform-run claim infrastructure
that ingests in-toto Statements. A new predicate type on existing rails
costs us a JSON envelope. A new log would cost us a service, which
violates local-first and zero-server.

Rung: CT history and the named projects are training knowledge
(cutoff Jan 2026) except the predicate-registry probe above. F1-F3 of the
research prompt verify current state.

## 4. The compounding loop (design intent, not measured)

Three append-only artifacts, each one's growth raising the value of the
other two:

1. The corpus (exists, public, scored): every published miss improves the
   engine; test 42 guards the reproduction floor.
2. The lock index (spec drafted, awaiting ratification): every published
   lock adds a node to the public declaration graph.
3. The attestation stream (this draft): every verification event adds a
   time point to the public health record of a claim.

A consumer reading an attestation is led to the lock; a lock is only
legible through the format docs; the format docs cite the corpus and the
miss rate. Each artifact is citable, deterministic, and replayable. That
is the compounding: not features, records.

## 5. The second graph: MCP tool schemas (same physics, new substrate)

An MCP tool schema is a producer contract. Agent code calling the tool is
the consumer. Schemas drift; agents break silently. Anyone can diff
declared schema against declared schema; that is cheap and others will do
it. INVAR's differentiated physics is declared versus observed: the lock
records what the tool's responses actually contained, with witness counts
and staleness, exactly as it does for payloads today. The crate already
has an mcp module and a TOML/JSON observation pipeline; the schema graph
is an application of W2/W3, not a new engine.

Position: the attestation layer is the headline; the MCP graph is the
second territory it covers. The founder's running deep-research report
already has MCP drift as an adjacency; adjudicate that section before
building here.

## 6. Prior art, named squarely (training knowledge; F5-F6 verify)

- Pact and the Pact Broker: consumer-driven contract testing with a
  central verification matrix and a can-i-deploy gate. This is the
  closest living relative and it proves two things at once: the demand is
  real, and the both-sides adoption trap is lethal. Pact requires both
  producer and consumer to adopt a testing framework and run a broker.
  INVAR's standing answer is one-sided value: invar init pays off with
  zero counterpart adoption, no tests written, no server run.
- Buf (protobuf) and Confluent Schema Registry: breaking-change detection
  plus a registry became de facto standards inside their format silos.
  Precedent for become-the-format; limited to declared-vs-declared.
- GitHub Artifact Attestations: build provenance on sigstore rails, public
  beta 2024. The platform threat in this layer is GitHub extending
  attestations toward contracts. Our standing answers hold: neutrality
  (GitHub cannot credibly be the referee of claims about code GitHub
  hosts), local-first and air-gap, and observed-reality evidence rather
  than declared-schema diffing.

## 7. Doctrine bindings (these are constraints, not decoration)

The supply-chain world frames itself as security. Our locked trust rule
says a published lock is a claim by its publisher, for coordination,
never security. These collide unless the attestation is worded with
care, so the following language is binding on any predicate schema we
draft:

- The attestation attests to a verification EVENT: this engine version,
  this commit pair, these declared dependencies, this outcome, these
  witnesses. It attests nothing about safety, correctness, or security.
  This is the same honesty SLSA provenance practices: built-by-X-from-Y,
  not safe.
- The predicate schema embeds the stated honest floor and the miss-rate
  pointer. An INVAR attestation that cannot cite the public miss rate is
  not an INVAR attestation.
- Banned phrases remain banned: no zero-hallucination, no proves safety.
  "Only engine attesting integration contracts" is a labeled hypothesis
  until F1 confirms the registry gap and stays a hypothesis about the
  future regardless.
- Signing and identity come later or never. The first emitter is
  unsigned, offline, and publisher-claimed, exactly like the dependents
  index. We never handle keys in argv or chat.

## 8. What kills this (recorded before any build)

- F1 finds an existing or in-progress contract-compatibility predicate.
  Then we implement theirs instead of drafting ours. Riding rails means
  riding them when they already go where we are going.
- Publishers only attest passes. Mitigation is disclosure, not
  prevention: coverage statements in the attestation (checks evaluated,
  checks not evaluated, reason codes), same physics as invar coverage.
  If selective publication still dominates, the stream degrades into
  marketing and the layer fails. This is the biggest open risk.
- Replay requires access to both repos. Cross-org private pairs cannot
  replay, so scope v1 to open source, where SWHID anchors already give
  us durable replay targets.
- Nobody consumes the attestations. CT had browser mandates; we have
  none. Candidate consumers to test cheapest-first: agents citing
  receipts in PRs, GUAC ingestion, dependabot-style bots gating bumps.
  If none consumes within a measured window after launch, this layer is
  shelved like E2/E3, recorded, not deleted.
- sigstore/in-toto governance rejects or stalls the predicate. Fallback:
  the predicate lives in our repo as a documented format; the envelope
  is still standard in-toto Statement JSON, so third parties can verify
  shape without upstream blessing.

## 9. First buildable step (small, offline, zero new dependencies)

invar check --attestation: emit the existing receipt wrapped in an
in-toto Statement JSON envelope (_type, subject with repo digest,
predicateType URI under the public repo, predicate = receipt plus
coverage plus engine version plus MEASURED_RECORD). No network, no
signing, no new crates; the JSON renderer we already use. Prediction to
record before building: the envelope adds no new failure modes to the
frozen exit contract because it is a serialization of state the engine
already computed. A test proves byte-stable output for a pinned fixture.

Ordering: this does not jump the queue. DEPENDENTS-GRAPH ratification
and the MCP-extension-versus-index decision come first; the attestation
emitter slots naturally beside the index builder because index entries
and attestation subjects share the claim-statement shape.

## 10. Open for founder ratification

1. Accept or reject the attestation layer as the third strategy lock
   (after declaration-graph and substrate).
2. Approve running the supplementary research prompt (Sections F1-F6)
   before any predicate schema is drafted.
3. Confirm v1 scope: open source only, unsigned, emit-only.
