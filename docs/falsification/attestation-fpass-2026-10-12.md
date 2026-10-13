# Attestation Layer Falsification Pass (F1-F6), 2026-10-12

Provenance: founder-run deep research (LLM-assisted), delivered 2026-10-12
and adjudicated by the engineer the same day. The report's own evidence
labels ([PRIMARY]/[SECONDARY]/[INFERENCE]) and its zen-pass downgrades are
part of the record; readers must not quote an [INFERENCE] line as a
finding. Adjudication outcome: the attestation layer is SHELVED (see
strategy/ATTESTATION-LAYER.md for the re-entry condition), the MCP agent
surface is confirmed as the next build, and three upgrades the
accepting dispatch made beyond the report's text are corrected in
strategy/RUNWAY.md. Report verbatim below.

---

# INVAR Attestation Layer, Falsification Pass F1 to F6

The evidence says the attestation move is wrong as a first move, and the maintainers should not build it now. The in-toto envelope costs almost nothing to emit, because standard tooling accepts custom predicate types. But nothing that would read it is in place. GUAC has no parser for an unknown predicate. Attestation consumption is unmeasured everywhere I looked. No regulation asks for integration-contract claims. The consumer with the most evidence of demand, AI and agent review in pull requests, does not need an in-toto envelope at all. Ship the receipt as a PR check and an agent-readable artifact. Add the envelope only if a named consumer asks for it.

## TL;DR

- **The gap is real but cheap to fill and cheap to copy.** No vetted or open in-toto predicate covers producer-consumer contract compatibility. However, the registry on `main` now has at least twelve vetted entries, including the generic "Simple Verification Result" and "Test Result", which could carry an INVAR verdict today. cosign and `actions/attest` already carry custom predicate URIs, so being "the format" gives no moat.
- **Consumption is the binding constraint, and it is absent or unmeasured.** GUAC parses only known predicate types. No GUAC production adopter is named in its repo. npm provenance, the most visible attestation, was enabled by only 26 of 205 top GitHub-published packages in one sample. The closest real compatibility gate, Pact's can-i-deploy, has been configured not to block merging by at least one named production user (GOV.UK).
- **No regulatory tailwind.** The EU CRA's vulnerability-handling requirements are about security updates and SBOMs, and contain no compatibility-claim obligation. US EO 14306 (June 2025) and OMB M-26-05 weakened the federal attestation regime. Rank the first consumers as: agents in PRs, then bump-gating bots, then GUAC. Run the three-week dual-arm experiment below before building anything.

## Scope notes

- Lookalike check: none of the listed lookalikes (hardpointlabs/invar, Tefx/Invar, nrlab-CRUK/INVAR2, onyxfish/invar, jloasxhq/invar, sysuTobo/invar, the alloy) came up in any result. Nothing was discarded.
- INVAR docs: I did not re-read docs/epistemic/STATUS.md, docs/architecture/, docs/falsification/ or docs/reference/REASON-CODES.md. My fetch tool is blocked from GitHub tree pages (the in-toto `spec/predicates` tree fetch returned "Site disallows automated access"). All items listed as unconfirmed in the brief stay unconfirmed.
- Evidence labels: [PRIMARY] is a first-party page (spec, vendor docs, project repo, official changelog). [SECONDARY] is a third-party description. [INFERENCE] is my reasoning.

## F1. The predicate registry gap: is it real and is it still open?

**Finding F1.1: The maintainers' ten-predicate count is out of date. The gap they describe still exists, but a generic predicate may already fill it.**
- At tag v1.0.2 the directory lists ten vetted predicates: SLSA Provenance, Link, SCAI Report, Runtime Traces, SLSA Verification Summary, SPDX, CycloneDX, Vulnerability, Release and Test Result. [PRIMARY: https://github.com/in-toto/attestation/blob/v1.0.2/spec/predicates, accessed 2026-10-12] This matches the maintainers' probe.
- The `main` branch README lists more: CycloneDX, Link, **Reference** ("References documents that are relevant to some resource"), Release, Runtime Traces, SCAI Report, SLSA Provenance, SLSA Verification Summary, "SPDX2 and SPDX3", **Simple Verification Result** ("Evidence that an artifact has been evaluated against one or more policies"), Test Result ("A generic schema to express results of any type of tests") and VULNS. [PRIMARY: https://github.com/in-toto/attestation/tree/main/spec/predicates, search snippet, accessed 2026-10-12]
- None of these names producer-consumer contract compatibility. [PRIMARY: same]
- [INFERENCE] The falsifying point: an INVAR verdict can be expressed today as a Test Result or Simple Verification Result predicate, with the receipt digest as a subject or reference. A dedicated "integration-contract" predicate adds field-level semantics, but the registry slot itself is not scarce. If a downstream consumer only needs "pass/fail against a policy", the generic predicates already serve it, and INVAR's distinguishing fields (field paths, producer ref, miss rate) would be ignored.

**Finding F1.2: No open PR proposes an API-compatibility, interface-contract, schema-compatibility or consumer-driven-contract predicate. The recent proposal queue is dominated by AI-agent predicates.**
- The open PR list fetched on 2026-10-12 shows 9 open and 405 closed. The open predicate PRs are: eval-result #575 (opened 23 Jul 2026), Adversarial Execution Evidence #570 (15 Jul 2026, draft), runtime-trace v0.2 #568 (12 Jul 2026), OCG Execution Receipt #567 (10 Jul 2026), agent-threat-scan #552 (9 May 2026), promotion record #532 (4 Mar 2026, opened by a Member), initial baseline #502 (20 Nov 2025, 35 comments) and Tool Assessment #496 (8 Oct 2025, 51 comments). [PRIMARY: https://github.com/in-toto/attestation/pulls, 2026-10-12]
- A search snapshot of the same repo also lists later PRs: AI Change Review/Authorship/Provenance #601 to #603 (29 Sep 2026), Computation Receipt #592, AI Agent Decision #591, EIP-712 Payload #590, AI Agent Action #588, PRML #587 and source-review-coverage #581. [PRIMARY: https://github.com/in-toto/attestation/pulls, search snippet, 2026-10-12]
- **Contradiction:** the direct fetch shows only 9 open PRs, with the newest at #575. The search snapshot lists PRs up to #603. Either those later PRs were closed, or the two views were taken at different times. I cannot reconcile this.
- Issue #604 (AI code-generation provenance) states that "No existing or proposed predicate provides per-range human/AI attribution". This shows proposers checking the queue for overlap. [PRIMARY: https://github.com/in-toto/attestation/issues/604, 2026]
- Absence finding: I looked at the open and recent PR lists, two search passes on the repo, and the predicates README. I found no compatibility or contract predicate. I did not search closed issues by keyword ("compatibility", "contract", "schema"), the CNCF Slack, or the mailing list, so a past rejected proposal could exist without my finding it.

**Finding F1.3: The documented process is light. The observed process is slow and open-ended.**
- The guidelines say: "New attestation predicates usually undergo a short vetting process"; open a PR per the ITE-9 template; "The in-toto Attestation Framework maintainers will review the PR at the next maintainers meeting"; "If accepted, the new predicate type will be included in our directory." Proposers must answer "What's your use case?", "Why don't existing predicates cover this use case?" and "What policy questions do you want to be able to answer with the predicate?" [PRIMARY: https://github.com/in-toto/attestation/blob/main/docs/new_predicate_guidelines.md, accessed 2026-10-12]
- Observed: Tool Assessment #496 has been open about 12 months with 51 comments. Initial baseline #502 has been open about 11 months. Promotion record #532, by a project Member, has been open about 7 months. [PRIMARY: https://github.com/in-toto/attestation/pulls, 2026-10-12] [INFERENCE] "Next maintainers meeting" does not mean "merged soon". On the current queue, plan for six to twelve months or more, and expect review to ask why Test Result or Simple Verification Result does not already cover the use case.
- Calibration points: the Test Result predicate came from PR #152 and shipped in v1.0.1. Release came from PR #319 (author steiza) and shipped in v1.0.2, with a v0.2 update in PR #495. [SECONDARY: subagent read of the in-toto/attestation releases page, 2026-10-12] Test Result already appears in a SLSA blog post dated 2 May 2023. [PRIMARY: https://slsa.dev/blog/2023/05/in-toto-and-slsa, 2023-05-02] sigstore/fulcio issue #1548 describes the Release predicate as still in progress around January 2024. [SECONDARY: subagent, 2026-10-12] I could not get exact opened and merged dates for #152 or #319, so I cannot give a proposal-to-merge time.
- Accepted vs rejected: I found no explicitly rejected predicate with stated reasons. The pattern in the queue is long-open rather than rejected.

**Finding F1.4: A third party can ship a custom predicateType URI without upstream acceptance, and standard tooling carries it.**
- The original spec says: "Users are expected to choose a predicate type that fits their needs, or invent a new one if no existing one satisfies. Predicate types are not registered." [PRIMARY: https://github.com/in-toto/attestation/tree/v0.1.0/spec, v0.1.0] The v1 predicates README says "the framework easily supports new predicate types for new use cases." [PRIMARY: https://github.com/in-toto/attestation/tree/main/spec/predicates]
- **cosign:** `GenerateOpts.Type` is documented as "the pre-defined enums (provenance|link|spdx). default: custom". The constant for `--type custom` is `CosignCustomProvenanceV01 = "https://cosign.sigstore.dev/attestation/v1"`. [PRIMARY: https://pkg.go.dev/github.com/sigstore/cosign/v2/pkg/cosign/attestation] Kyverno's docs show cosign signing with an arbitrary URI: `cosign attest ... --predicate predicate.json --type https://example.com/CodeReview/v1`. [PRIMARY: https://main.kyverno.io/docs/policy-types/cluster-policy/verify-images/sigstore/, accessed 2026-10-12] [INFERENCE] So `--type` takes either a short enum name or a full URI, and a URI is passed through as the predicateType. I did not read `options/predicate.go` directly to confirm the pass-through branch.
- **GitHub `actions/attest`:** there is an explicit "Custom" mode ("predicate-type/predicate/predicate-path provided | User-supplied predicate"), and `predicate-type` is "Required when using 'predicate' or 'predicate-path' for custom attestations." The README example uses `predicate-type: 'https://example.com/predicate/v1'`. [PRIMARY: https://github.com/actions/attest, accessed 2026-10-12] The predicate cannot exceed 16MB. [PRIMARY: https://github.com/scalaaiinc/attest, a fork mirroring the README]
- Verifying a non-default type with `gh attestation verify` requires `--predicate-type`, because "its default looks for SLSA build provenance". This was measured in a real PR that attests a Test Result predicate. [PRIMARY: https://github.com/lfn-cnti/testsuite-action/pull/15, 10 Oct 2026]
- **Rekor:** cosign exposes `--tlog-upload=false` and `--rekor-entry-type dsse`, so attestations go to the transparency log by default. [PRIMARY: https://github.com/sigstore/cosign/pull/5108] [INFERENCE] DSSE entries in Rekor are typed by envelope, not by predicate, so a custom predicate would be stored. I did not read Rekor's type docs to confirm this.
- **GUAC:** see F2. Carriage works; parsing does not.

What I could not verify (F1): exact opened and merged dates for PRs #152 and #319; any closed or rejected compatibility-type proposal (closed issues not keyword-searched); CNCF Slack and mailing-list archives (not searched); the cosign source branch that passes a URI through `--type`; Rekor's documented behaviour for arbitrary predicate types; the cause of the open-PR count contradiction.

## F2. GUAC and consumers: would anyone ingest this?

**Finding F2.1: GUAC handles a fixed set of predicate types. A custom INVAR predicate would not become graph data without an upstream GUAC parser.**
- in-toto's integrations list says "GUAC has the ability to ingest and parse SLSA and other in-toto ITE6 attestations (either wrapped in DSSE or standalone)." [PRIMARY: https://github.com/in-toto/friends, accessed 2026-10-12]
- GUAC's processor defines specific ITE6 document types: SLSA, generic "ITE6", ITE6VUL, ITE6EOL, ITE6REF, ITE6CD (ClearlyDefined), plus DSSE and OPAQUE. Parser packages exist for known predicates such as `https://in-toto.io/attestation/vulns/v0.1`. [PRIMARY: https://pkg.go.dev/github.com/guacsec/guac/pkg/ingestor; SECONDARY: subagent read of pkg.go.dev for pkg/handler/processor v1.1.0, 2026-10-12]
- The ITE6 processor's own comment says the unpack step "does not return anything as it will be the same document". [SECONDARY: subagent, quoting pkg/handler/processor/ite6/ite6.go doc comment]
- [INFERENCE, unconfirmed against source] An unrecognised predicateType most likely falls through to the generic "ITE6" type, which has no registered parser. The likely result is an error at parse time, not opaque storage. The subagent could not fetch `parser.go` to confirm the exact branch.
- A reviewer on open GUAC PR #3017 warned that a v1 statement in one code path "will unmarshal with both fields empty and the rest of the pipeline runs on the empty result with no error". So edge cases can fail silently. [SECONDARY: subagent, quoting guacsec/guac PR #3017 review by mlieberman85]
- [INFERENCE] GUAC's graph is built around packages, artifacts, SBOMs, vulnerabilities and scorecards. There is no node or edge for "consumer repo X reads field F of producer Y". Even with a parser, INVAR would need a new ontology upstream. That is a second two-sided adoption problem on top of the first.

**Finding F2.2: Documented production consumption of GUAC is thin.**
- An independent profile (June 2026) says: "from 1,100+ stars at incubation (March 2024) to 1,502 in June 2026 is slow community growth for a four-year-old Google-originated project, and no production adopters are named in the repo itself." [SECONDARY: https://rywalker.com/research/guac, 2026]
- The Kusari (GUAC's commercial backer) case study says Guidewire "is in the development phase with GUAC" and "look[s] forward to maturing into the production phase". [PRIMARY vendor: https://www.kusari.dev/case-studies/innovative-supply-chain-security-for-enterprise-cloud-platform-service]
- The named endorsements are a 2023 Yahoo quote ("immense value and significant efficiency") and an OpenSSF listing of supporters (Yahoo, Microsoft, Red Hat, Guidewire, ClearAlpha). These are support statements, not deployment metrics. [PRIMARY: https://openssf.org/blog/2024/03/07/guac-joins-openssf-as-incubating-project/, 7 Mar 2024; SECONDARY: https://www.infoq.com/news/2023/07/google-announces-guac-0-1/, Jul 2023]
- GUAC v1.1.0 shipped 13 March 2026. Its maintainer meeting is now "every-other-week instead of weekly". [PRIMARY: https://guac.sh/blog/2026-03-13-guac-v1.1.0/; https://guac.sh/blog/]
- in-toto's adopters list names OpenVEX and Palantir ("signed attestations ... layered verification"). [PRIMARY: https://github.com/in-toto/friends]

**Finding F2.3: Compared with SBOM consumption, attestation consumption is unmeasured, and the emission-side evidence is worse.**
- For SBOMs there are consumption surveys: ENISA's 23% significant gap and 7% closed, and Cloudsmith's 25% (settled findings). For attestations I found **no consumption survey at all**.
- The only adoption numbers are on the emission side, and they are low. Of the 2,000 most-downloaded jsDelivr packages, 205 publish from GitHub Workflows, and "only 26 (12.6%) of these packages have enabled provenance". [SECONDARY: https://www.exaforce.com/blogs/npm-provenance-bridging-the-missing-security-layer-in-javascript-libraries, date not stated, c. late 2024] The Sigstore blog's GA post reports that "During the public beta period (which lasted from April 2023 until September 26), over 3,800 projects have adopted build provenance (including 134 high-impact projects)", with over 500 million total downloads of provenance-enabled package versions. [PRIMARY: https://blog.sigstore.dev/npm-provenance-ga/, 2023]
- A maintainer explains publicly why he declined provenance: "it does not seem to me that introducing npm provenance offers compelling enough benefits (for package consumers or for producers) to offset the cost". [PRIMARY: https://dlaa.me/blog/post/npmprovenance, 2024]
- [INFERENCE] If SBOMs are generated far more than they are read, attestations look like they are not even widely generated, apart from where a platform makes them default. The honest comparison is "unmeasured, and probably worse". This should change sequencing: building for an attestation consumer means building for a population nobody has counted.

**Finding F2.4: Gating on attestations exists as a capability at deploy time. I found no dependency-bump gate on any verification record. The one real compatibility gate in production was configured not to block.**
- Kyverno `verifyImages.attestations` can enforce on any predicate type, including custom ones (`predicateType: https://example.com/CodeReview/v1`, `failureAction: Enforce`). [PRIMARY: https://main.kyverno.io/docs/policy-types/cluster-policy/verify-images/sigstore/] An earlier Kyverno release labelled image verification "a beta feature. It is not ready for production usage". [PRIMARY: https://release-1-9-0.kyverno.io/docs/writing-policies/verify-images/] I found tutorials and vendor blogs, but **no named production deployment** gating on attestations.
- The in-toto spec names its intended consumers as "automated policy engines, such as in-toto-verify and Binary Authorization". [PRIMARY: https://github.com/in-toto/attestation/blob/main/spec/README.md] I found no published Binary Authorization deployment metrics.
- **GOV.UK** publishing documents its Pact setup with a broker, then states: "We've configured failing Pact tests to not block merging, so you should still be able to merge." [PRIMARY: https://docs.publishing.service.gov.uk/manual/pact-testing.html, accessed 2026-10-12] This is the most important consumption datapoint in this pass. A named production organisation that already had a compatibility gate chose to make it advisory.
- Renovate/Dependabot with provenance checks, OPA/Gatekeeper, Sigstore policy-controller, and anything gating on the npm provenance badge: **not found**. My search budget ran out before I could query these specifically, so this absence is weakly established.

What I could not verify (F2): the exact GUAC code path for unknown predicates; any GUAC, Rekor-search or GitHub-attestation-verification usage metric; any production Kyverno, policy-controller or Binary Authorization case study naming an organisation; any bot gating a dependency bump on provenance. Renovate, Dependabot and OPA were not specifically searched.

## F3. Platform threat: GitHub's trajectory in this layer

**Finding F3.1: GitHub has generalised its attestation tooling to arbitrary predicates. This makes INVAR's envelope trivial to emit and trivial to copy.**
- `actions/attest` v4 has three modes: provenance (default), SBOM, and custom predicate. [PRIMARY: https://github.com/actions/attest]
- `actions/attest-sbom` is now a deprecated wrapper: "its v4.1.0 only prints a warning and calls actions/attest". In a CI test, `actions/attest` accepted CycloneDX and SPDX 2.3 but rejected SPDX 3 ("Unsupported SBOM format"). [PRIMARY: https://github.com/kusari-oss/waybill-action/pull/6, 2026]
- GitHub's changelog of 28 Oct 2025 ("Immutable releases are now generally available") says release "assets and tags are protected from tampering after publication", and GitHub Community post #178351 (29 Oct 2025) describes the feature as "generating signed attestations for every asset" (settled finding).
- [INFERENCE] GitHub is building a general attestation pipe, not a semantic layer. I found no GitHub announcement of compatibility or contract claims. I did not search the github/roadmap repo, Dependency Review, the dependency graph, Dependabot or Copilot Autofix changelogs because my search budget ran out. This is an unexamined area, not a negative finding.
- [INFERENCE] The more credible platform threat is Copilot code review, which already runs on more than one in five GitHub reviews (settled finding). If GitHub adds "this change breaks a dependent" reasoning, it will appear as review commentary, not as an attestation. That competes with INVAR's consumer #1, not with the envelope.

**Finding F3.2: SmartBear has moved contract testing toward agents and into its Swagger brand, not toward attestations.**
- "PactFlow is now Swagger Contract Testing. Swagger Contract Testing, previously known as PactFlow, is now part of Swagger" (18 Jan 2026). [PRIMARY: https://support.smartbear.com/swagger/contract-testing/docs/en/release-notes.html]
- Later entries: "We added most of PactFlow's API to our MCP server, enabling a wide range of use cases in agentic coding" (23 Apr 2026); AsyncAPI support in Bi-Directional Contract Testing (16 Jul 2026); "Swagger Contract Testing AI using the MCP" for generating and reviewing contract tests in the IDE; automatic branch cleanup for all SaaS tenants (19 Jun and 28 Aug 2026). [PRIMARY: same]
- The SmartBear MCP server exposes `contract-testing_matrix`, `contract-testing_generate_pact_tests` and `contract-testing_review_pact_tests`. [PRIMARY: https://docs.pact.io/ai_tools/smartbear-mcp]
- PactFlow AI-augmented test generation (HaloAI) entered beta in 2024, as PactFlow was being folded into API Hub. [PRIMARY: https://smartbear.com/news/news-releases/expanding-haloai-footprint-smartbear-boosts-code/, 2024]
- [INFERENCE] The incumbent's revealed bet is that agents consume contract data over MCP. I found no SmartBear in-toto or Sigstore emission. This is evidence for consumer #1 and against the envelope.

What I could not verify (F3): the GitHub public roadmap; the Dependency Review, Dependabot and Copilot Autofix trajectory; acquisitions or experiments by GitLab, CircleCI, Buildkite, Harness, JFrog or Chainguard in contract testing since 2024. None of these were searched.

## F4. Regulatory tailwind: does anyone have to care?

**Finding F4.1: The EU CRA creates no obligation for machine-readable claims about maintained compatibility. Its update language is about security, separation and delivery, not non-breakage.**
- Entry into force: 10 December 2024. [PRIMARY: https://eur-lex.europa.eu/eli/reg/2024/2847/oj/eng, metadata `first_date_entry_in_force: 2024-12-10`] The European Commission's CRA page states: "The main obligations introduced by the Act will apply from 11 December 2027, with reporting obligations to apply as of 11 September 2026", and its summary specifies that "reporting obligations set out in Article 14 apply from 11 September 2026."
- Annex I Part II(2): "address and remediate vulnerabilities without delay, including by providing security updates; where technically feasible, new security updates shall be provided separately from functionality updates". [SECONDARY mirror of the Regulation text: https://cyber-resilience-act.com/annexes/annex-i/]
- Part II(3): "apply effective and regular tests and reviews of the security of the product with digital elements". [SECONDARY: same]
- Part II(7): "provide for mechanisms to securely distribute updates ... to ensure that vulnerabilities are fixed or mitigated in a timely manner and, where applicable for security updates, in an automatic manner". [SECONDARY: same]
- Part II(8): security updates "disseminated without delay and, unless otherwise agreed ... free of charge, accompanied by advisory messages providing users with the relevant information, including on potential action to be taken." [SECONDARY: same]
- Annex I Part I: automatic security updates "enabled as a default setting, with a clear and easy-to-use opt-out mechanism ... and the option to temporarily postpone them". [SECONDARY: https://streamlex.eu/annexes/cra-en-annex-i/]
- Recital 57 explains the separation rule: "to ensure that users are not required to install new functionality updates for the sole purpose of receiving the latest security updates". [SECONDARY: https://www.springlex.eu/en/packages/cra/cra-regulation/annex-1/]
- Documentation must be kept 10 years or for the support period, whichever is longer (Art 13). [SECONDARY: https://en.wikipedia.org/wiki/Cyber_Resilience_Act]
- **No "security updates must not break" language appears in the quoted provisions.** [INFERENCE] The closest hook is indirect. The security/functionality separation and default-on automatic updates both assume a security update is safe to auto-apply. A manufacturer could use compatibility evidence internally to justify that. But the CRA does not ask for that evidence, in any format, from anyone. I put the probability that CRA harmonised standards will require machine-readable integration-compatibility claims by 2027 below 10%, because the text is consistently vulnerability-oriented and SBOM-oriented.
- I did not re-retrieve the Annex I Part II(1) SBOM wording or the Annex VII technical-documentation items this pass.

**Finding F4.2: The US federal attestation regime has shrunk, and nothing in it touches integration contracts.**
- CISA released the Secure Software Development Attestation Form on 11 March 2024. [SECONDARY: https://www.jdsupra.com/topics/software/nist/supply-chain]
- EO 14144 (January 2025) would have required "attestations, supporting artifacts" submitted to CISA's RSAA with CISA validation. EO 14306 (6 June 2025) "removed the Biden-era requirement for federal contractors and subcontractors providing computer software to submit validated attestations and artifacts" through CISA's RSAA. It instead directs NIST to update the SSDF via an NCCoE consortium. [SECONDARY: https://www.wilmerhale.com/en/insights/client-alerts/20250613-new-executive-order-modifies-cybersecurity-requirements-to-be-imposed-on-federal-contractors-and-subcontractors, 13 Jun 2025; https://www.pilieromazza.com/cybersecurity-ai-and-eo-14306-what-government-contractors-must-do-to-stay-compliant/]
- OMB then rescinded the M-22-18 mandate. Under M-26-05, agencies are "permitted" but "not directed" to obtain secure development attestations, and may use the CISA common form. [SECONDARY: https://www.dwt.com/blogs/privacy--security-law-blog/2026/02/omb-changes-course-on-software-security, Feb 2026; https://www.wiley.law/alert-OMB-Rescinds-Secure-Software-Development-Mandate-in-Favor-of-a-Risk-Based-Approach]
- [INFERENCE] The opening for "third-party machine-generated evidence as supporting artifacts" was exactly what EO 14144 created and EO 14306 removed. I did not retrieve the form's attestation items or the SSDF practice mapping text this pass.

**The finding, stated plainly: no regulation I examined touches integration contracts.** The EU is security and SBOM oriented. The US is narrowing to optional self-attestation.

What I could not verify (F4): primary EUR-Lex article text (the fetch returned only the header); the Art 14 and Art 71 application dates; Annex VII items; the CISA form's exact attestation clauses; the M-26-05 primary text.

## F5. The Pact autopsy: why is the closest relative not the standard?

**Finding F5.1: The commercial product has lost its own brand and been absorbed into the platform's design suite.** PactFlow is now "Swagger Contract Testing ... part of Swagger" (18 Jan 2026). It is sold as "API Hub for Contract Testing, formerly known as PactFlow". SmartBear's integration docs describe API Hub for Design "coordinat[ing] with API Hub for Contract Testing when compatibility is checked to discover all known consumers". [PRIMARY: https://support.smartbear.com/swagger/contract-testing/docs/en/release-notes.html; https://support.smartbear.com/api-hub/design/docs/en/manage-apis/pactflow-contract-testing.html] [INFERENCE] This fits the category failure mode (acquisition, then absorption). Releases are still shipping in 2026, so it is not neglect yet.

**Finding F5.2: Practitioner accounts point to organisational and coordination failure, not only test-writing cost.**
- GOV.UK made failing Pact tests non-blocking (F2.4). [PRIMARY: https://docs.publishing.service.gov.uk/manual/pact-testing.html]
- A practitioner postmortem on 12 microservices: after a consumer added a dependency on a new field, "We had to coordinate deployments". Contract tests passed while production config pointed at a different version: "If production config says 'use v2' but your contract tests 'v1', you're testing the wrong thing." The team "didn't abandon Pact entirely. But we stopped trusting it as our only defense." [SECONDARY, single anonymous Medium author: https://medium.com/@codexlab/contract-testing-with-pact-looked-good-broke-anyway-3774270bdd15]
- Pact's own docs warn that over-specified consumer scenarios block non-breaking provider changes: "These are not breaking changes, but by over-specifying our scenarios we are stopping the User Service Team from implementing them." [PRIMARY: https://docs.pact.io/consumer/contract_tests_not_functional_tests]
- PactFlow published "A disastrous tale of UI testing with Pact" and a companion "Why Pact implementations fail". [PRIMARY: https://pactflow.io/blog/a-disastrous-tale-of-ui-testing-with-pact/]
- A widely shared claim that "Contract testing as a category has been stuck at ~5% adoption since around 2018" comes with no cited data. I treat it as unsupported opinion. [SECONDARY: https://dev.to/deepaksatyam/pact-is-great-its-also-why-most-teams-dont-do-contract-testing-5c14, 2026]

**Finding F5.3: Steelman.** PactFlow says "The approach and tooling we're using has been shown to be effective at Atlassian". [PRIMARY: https://pactflow.io/blog/bi-directional-contracts/] GOV.UK runs it with a broker across many apps. [PRIMARY: GOV.UK docs] [INFERENCE] What successful users have in common is Pact's own README scope: one organisation controls both consumer and provider and has a platform team that owns the broker. That is centralised ownership, not cheaper tests.

**Finding F5.4: The question that matters.** [INFERENCE, from F5.1 to F5.3 and the settled findings] INVAR removes one actual blocker. It is one-sided: a consumer can check against a producer's OpenAPI or source without the provider team doing anything. That answers PactFlow's own "It can be hard to get provider teams on board". But SmartBear already removed the provider test-writing burden with Bi-Directional Contract Testing (provider publishes OpenAPI), and that did not make the category standard. The remaining blocker is ownership of the gate: who is allowed to block whose release. GOV.UK's non-blocking configuration shows that even an organisation with a working gate chose not to let it block. INVAR's no-server design removes broker operations cost. It does not create an owner. With a published 46% miss rate, a blocking gate would face the npm-audit credibility problem from the other direction: misses rather than noise. **Verdict: INVAR's advantage is real for a single team auditing its own dependencies on others. For a cross-organisation trust layer it is largely illusory.**

What I could not verify (F5): commit and release cadence and issue age across pact-foundation repos; Postman State of the API or JetBrains contract-testing figures; job-posting counts; PactFlow pricing changes after the rebrand; the dates and authors of the "Why Pact implementations fail" post. None were measured; my search budget ran out.

## F6. Become-the-format precedents and their economics

**Finding F6.1: Buf shipped the free checker first, planned the registry from early on, and monetised the registry about three years later, inside a format silo that already existed.**
- A user asked about BSR timing and open-sourcing on 6 Feb 2020, so the registry was planned while the `buf` CLI (lint, breaking, generate) was the shipping product. [PRIMARY: https://github.com/bufbuild/buf/issues/24, 6 Feb 2020]
- Buf's "Introducing the Buf Schema Registry" post, dated 2 February 2022, calls the `buf` tool "our first major contribution", with "detecting breaking changes, and linting", and says the BSR "is officially in beta" after a soft launch "a few months ago". [PRIMARY: https://buf.build/blog/announcing-bsr, 2 Feb 2022]
- The BSR left beta in a post dated 12 April 2023, "nearly two years" after launch, with types-based pricing: Teams "$0.50 per type/month" and Pro "$1000 base price, including 200 types", with discounts for sign-ups "before October 15, 2023". [PRIMARY: https://buf.build/blog/plans-pricing-updates, 12 Apr 2023]
- Breaking-change governance (review and approve breaking commits server-side) is an Enterprise feature. [PRIMARY: https://buf.build/blog/review-governance-workflow]
- [INFERENCE] The money is in the registry and governance, not the checker. And Buf had protobuf/gRPC as an existing silo.

**Finding F6.2: Confluent's Schema Registry rode Kafka's adoption, and was relicensed to block cloud resellers once it had value.** On 14 Dec 2018 Confluent moved Schema Registry, REST Proxy, KSQL and some connectors from Apache 2.0 to the Confluent Community License from version 5.1. Avro/JSON serializers and the Schema Registry client stayed Apache 2.0. [PRIMARY competitor statement: https://aiven.io/blog/aiven-statement-on-kafka-license; SECONDARY: https://www.infoq.com/news/2018/12/confluent-license-changes/] InfoQ connects the change to AWS's managed Kafka announcement. Schema Registry is still Community-licensed today, with security plugins and Schema Linking under the Enterprise license. [PRIMARY: https://docs.confluent.io/platform/current/schema-registry/index.html] [INFERENCE] The registry was valuable only because Kafka already ran everywhere. INVAR has no such host format.

**Finding F6.3: SARIF took about 2.5 years from TC charter to OASIS Standard. It succeeded because a big consumer committed to ingest it.**
- The Call for Participation was issued in August 2017. Industry members were announced on 12 Oct 2017 (Microsoft, NIST, DHS, Synopsys, GrammaTech and others). [PRIMARY: https://www.oasis-open.org/committees/sarif/charter.php; https://www.oasis-open.org/committees/tc_home.php?wg_abbrev=sarif]
- v2.1.0 became an OASIS Standard on 27 March 2020, with Errata 01 on 28 August 2023. The editors were both Microsoft. [PRIMARY: https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html]
- GitHub code scanning "only supports SARIF version 2.1.0" and adds requirements beyond the standard. [PRIMARY: https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning]
- [INFERENCE] The adoption driver was a platform with distribution (Microsoft, then GitHub after its acquisition) committing to read the format and render it in a UI developers already use. Tool vendors emitted SARIF because GitHub would display it. INVAR has no equivalent committed reader.

**Finding F6.4: Failure calibration (partial).** SPDX 3 is a published spec that GitHub's attestation action refuses ("Unsupported SBOM format"), and "There is no settled predicate type to send SPDX 3 as a custom predicate". [PRIMARY: https://github.com/kusari-oss/waybill-action/pull/6, 2026] The open in-toto predicate queue (F1.3) is itself a set of specified-but-unadopted formats. I could not source a clean example of a format that was published and fully abandoned, with a documented reason.

What I could not verify (F6): the exact BSR beta date; the first `buf breaking` release date; Confluent revenue tied to Schema Registry; the GitHub SARIF upload or code-scanning GA date; a documented failed-format postmortem.

## Ranking the first consumers, and the cheapest falsifying experiment

**Ranking by evidence of existing demand:**

1. **Agents citing receipts in PRs: the strongest evidence, but it does not need in-toto.** GitHub's "60 million Copilot code reviews and counting" post reports that usage "has grown 10X, now accounting for more than one in five code reviews on GitHub", and that "More than 12,000 organizations now run Copilot code review automatically on every pull request" (settled). The incumbent contract vendor shipped its contract matrix and test review to agents via MCP in April 2026 (F3.2). Third parties are already attesting test results from CI (lfn-cnti PR #15, F1.4). LLM breaking-change detection is weak at 30.3% (settled), which leaves room for a deterministic checker as a tool the agent calls. [INFERENCE] What agents need is a deterministic, readable receipt and an exit code. A signed envelope adds nothing they use today.
2. **Dependency-bump gating bots: real pattern, weak demand.** The compatibility gate exists (Pact can-i-deploy), but GOV.UK disarmed it. Deploy-time attestation gates exist as Kyverno capability with no named deployments. No bump bot gating on attestations was found. The 46% miss rate makes a blocking gate an npm-audit risk.
3. **GUAC-style graphs: weakest.** There is no parser for unknown predicates, no ontology for field dependencies, and no named production adopters in the repo. Getting a parser in would be a second upstream adoption campaign.

**The single cheapest falsifying experiment (about 3 weeks):**
- **What to do:** Pick 10 repositories, or teams you can reach directly, that consume another repo's API and receive dependency-bump PRs. Run INVAR in CI on every bump PR for three weeks, in two arms per PR. Arm A posts the verdict as a check run plus a short machine-readable JSON receipt that AI reviewers and humans can read. Arm B additionally emits the same receipt via `actions/attest` with a custom predicateType and publishes a ready-to-use Kyverno or policy-controller snippet that gates on it. Log three things: whether the receipt is cited in review (human or Copilot comments referencing it), whether any team enables a blocking rule, and whether anyone fetches or verifies the attestation (ask teams directly, and watch for `gh attestation verify` in their workflows).
- **Confirms the attestation layer:** at least 3 of 10 teams enable a policy or verification step on the Arm B attestation, or ask for it unprompted, within three weeks.
- **Refutes the attestation layer:** 0 or 1 teams touch the attestation, while Arm A receipts are cited or acted on. That result keeps the checker and kills the envelope.
- **Refutes both:** neither arm's output is cited or acted on in any of the 10 repos. The demand problem is upstream of format.
- **Duration:** about 1 week to set up and 2 to 3 weeks to observe.

## Zen pass

Struck or downgraded on reread:
- "GUAC errors on unknown predicates" is downgraded to an unconfirmed inference (F2.1). The error branch was not read in source.
- "cosign passes any URI through `--type`" is supported by the Kyverno-documented example, not by the cosign source.
- The "~5% contract-testing adoption" figure is excluded as unsupported (F5.2).
- The absence of a Renovate/Dependabot provenance gate is weakly established, because those tools were not specifically searched.

Contradictions surfaced, not smoothed:
- **Carriage versus consumption.** F1 shows custom predicates flow freely through cosign, GitHub and (probably) Rekor. F2 shows the main graph consumer cannot parse them. "Standard tooling carries it" is true and does not matter much.
- **Registry count.** The maintainers' ten-predicate probe is accurate at v1.0.2 but stale against `main` (twelve or more entries, including two generic verdict predicates). That weakens the "missing layer" framing.
- **Open-PR count.** A direct fetch shows 9 open PRs with the newest at #575. A search snapshot lists PRs up to #603. This is unresolved.
- **Platform threat versus thin demand.** F3 treats GitHub as a threat. F2 says the layer has little demand. Both can be true only if the threat is to the checker via Copilot review, not to the envelope.
- **Pact's verdict.** F5's steelman (Atlassian, GOV.UK) and its autopsy (GOV.UK non-blocking) cite the same organisation. Pact works there as an advisory signal, not as a gate.

**The three findings most likely to make the maintainers not build this:**
1. **The best-evidenced consumer, agents in PRs, has no use for an in-toto envelope.** The incumbent (SmartBear) is shipping contract data to agents over MCP, not as attestations.
2. **Nothing consumes it.** GUAC cannot parse custom predicates and has no field-dependency ontology. Attestation consumption is unmeasured everywhere. npm provenance emission sits at 12.6% in one sample. And the closest real compatibility gate was configured not to block by a named production user (GOV.UK).
3. **No regulation pulls on it.** The CRA has no compatibility-claim obligation, and EO 14306 plus OMB M-26-05 removed the US machine-readable artifact path. With a self-published 46% miss rate, positioning INVAR as a supply-chain trust claim invites the npm-audit credibility collapse.

## Dated source list

- INVAR README: https://github.com/Ramesh-35/invar (read 2026-10-12, per brief)
- in-toto predicates at v1.0.2: https://github.com/in-toto/attestation/blob/v1.0.2/spec/predicates (accessed 2026-10-12)
- in-toto predicates on main: https://github.com/in-toto/attestation/tree/main/spec/predicates (accessed 2026-10-12)
- in-toto open PRs: https://github.com/in-toto/attestation/pulls (fetched 2026-10-12)
- in-toto issue #604: https://github.com/in-toto/attestation/issues/604 (2026)
- in-toto new predicate guidelines: https://github.com/in-toto/attestation/blob/main/docs/new_predicate_guidelines.md (accessed 2026-10-12)
- in-toto spec README: https://github.com/in-toto/attestation/blob/main/spec/README.md (accessed 2026-10-12)
- in-toto spec v0.1.0: https://github.com/in-toto/attestation/tree/v0.1.0/spec (v0.1.0)
- in-toto friends: https://github.com/in-toto/friends (accessed 2026-10-12)
- SLSA blog, in-toto and SLSA: https://slsa.dev/blog/2023/05/in-toto-and-slsa (2 May 2023)
- cosign attestation package: https://pkg.go.dev/github.com/sigstore/cosign/v2/pkg/cosign/attestation (accessed 2026-10-12)
- cosign PR #5108: https://github.com/sigstore/cosign/pull/5108 (2026)
- Kyverno Sigstore docs: https://main.kyverno.io/docs/policy-types/cluster-policy/verify-images/sigstore/ (accessed 2026-10-12)
- Kyverno 1.9 verify images: https://release-1-9-0.kyverno.io/docs/writing-policies/verify-images/ (Kyverno 1.9)
- actions/attest: https://github.com/actions/attest (accessed 2026-10-12)
- actions/attest fork README: https://github.com/scalaaiinc/attest (accessed 2026-10-12)
- lfn-cnti testsuite-action PR #15: https://github.com/lfn-cnti/testsuite-action/pull/15 (10 Oct 2026)
- kusari waybill-action PR #6: https://github.com/kusari-oss/waybill-action/pull/6 (2026)
- GUAC ingestor package: https://pkg.go.dev/github.com/guacsec/guac/pkg/ingestor (accessed 2026-10-12)
- GUAC v1.1.0: https://guac.sh/blog/2026-03-13-guac-v1.1.0/ (13 Mar 2026)
- GUAC blog: https://guac.sh/blog/ (accessed 2026-10-12)
- GUAC profile: https://rywalker.com/research/guac (2026)
- Kusari Guidewire case study: https://www.kusari.dev/case-studies/innovative-supply-chain-security-for-enterprise-cloud-platform-service (undated)
- OpenSSF GUAC incubation: https://openssf.org/blog/2024/03/07/guac-joins-openssf-as-incubating-project/ (7 Mar 2024)
- InfoQ GUAC v0.1: https://www.infoq.com/news/2023/07/google-announces-guac-0-1/ (Jul 2023)
- Exaforce npm provenance: https://www.exaforce.com/blogs/npm-provenance-bridging-the-missing-security-layer-in-javascript-libraries (undated, c. 2024)
- Sigstore npm provenance GA: https://blog.sigstore.dev/npm-provenance-ga/ (2023)
- dlaa npm provenance: https://dlaa.me/blog/post/npmprovenance (2024)
- GOV.UK Pact testing: https://docs.publishing.service.gov.uk/manual/pact-testing.html (accessed 2026-10-12)
- EUR-Lex CRA: https://eur-lex.europa.eu/eli/reg/2024/2847/oj/eng (Regulation of 23 Oct 2024)
- CRA Annex I mirror: https://cyber-resilience-act.com/annexes/annex-i/ (accessed 2026-10-12)
- CRA Annex I mirror: https://streamlex.eu/annexes/cra-en-annex-i/ (accessed 2026-10-12)
- CRA Annex I with recitals: https://www.springlex.eu/en/packages/cra/cra-regulation/annex-1/ (accessed 2026-10-12)
- Wikipedia CRA: https://en.wikipedia.org/wiki/Cyber_Resilience_Act (accessed 2026-10-12)
- WilmerHale on EO 14306: https://www.wilmerhale.com/en/insights/client-alerts/20250613-new-executive-order-modifies-cybersecurity-requirements-to-be-imposed-on-federal-contractors-and-subcontractors (13 Jun 2025)
- PilieroMazza on EO 14306: https://www.pilieromazza.com/cybersecurity-ai-and-eo-14306-what-government-contractors-must-do-to-stay-compliant/ (2025)
- DWT on OMB M-26-05: https://www.dwt.com/blogs/privacy--security-law-blog/2026/02/omb-changes-course-on-software-security (Feb 2026)
- Wiley on OMB rescission: https://www.wiley.law/alert-OMB-Rescinds-Secure-Software-Development-Mandate-in-Favor-of-a-Risk-Based-Approach (2026)
- JD Supra supply chain topic: https://www.jdsupra.com/topics/software/nist/supply-chain (accessed 2026-10-12)
- SmartBear Swagger Contract Testing release notes: https://support.smartbear.com/swagger/contract-testing/docs/en/release-notes.html (entries 18 Jan to 28 Aug 2026)
- SmartBear API Hub design integration: https://support.smartbear.com/api-hub/design/docs/en/manage-apis/pactflow-contract-testing.html (accessed 2026-10-12)
- SmartBear MCP server docs: https://docs.pact.io/ai_tools/smartbear-mcp (accessed 2026-10-12)
- SmartBear HaloAI release: https://smartbear.com/news/news-releases/expanding-haloai-footprint-smartbear-boosts-code/ (2024)
- SmartBear acquires Pactflow: https://smartbear.com/news/news-releases/smartbear-acquires-pactflow/ (5 Apr 2022)
- PactFlow bi-directional contracts: https://pactflow.io/blog/bi-directional-contracts/ (accessed 2026-10-12)
- PactFlow UI testing tale: https://pactflow.io/blog/a-disastrous-tale-of-ui-testing-with-pact/ (accessed 2026-10-12)
- Pact docs, contract vs functional tests: https://docs.pact.io/consumer/contract_tests_not_functional_tests (accessed 2026-10-12)
- Medium Pact postmortem: https://medium.com/@codexlab/contract-testing-with-pact-looked-good-broke-anyway-3774270bdd15 (undated)
- dev.to contract testing chasm: https://dev.to/deepaksatyam/pact-is-great-its-also-why-most-teams-dont-do-contract-testing-5c14 (2026)
- Buf issue #24: https://github.com/bufbuild/buf/issues/24 (6 Feb 2020)
- Buf BSR announcement: https://buf.build/blog/announcing-bsr (2 Feb 2022)
- Buf plans and pricing: https://buf.build/blog/plans-pricing-updates (12 Apr 2023)
- Buf governance workflow: https://buf.build/blog/review-governance-workflow (accessed 2026-10-12)
- Aiven on Confluent license: https://aiven.io/blog/aiven-statement-on-kafka-license (Dec 2018)
- InfoQ on Confluent license: https://www.infoq.com/news/2018/12/confluent-license-changes/ (Dec 2018)
- Confluent Schema Registry docs: https://docs.confluent.io/platform/current/schema-registry/index.html (accessed 2026-10-12)
- OASIS SARIF charter: https://www.oasis-open.org/committees/sarif/charter.php (2017)
- OASIS SARIF TC page: https://www.oasis-open.org/committees/tc_home.php?wg_abbrev=sarif (accessed 2026-10-12)
- SARIF v2.1.0: https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html (27 Mar 2020; Errata 28 Aug 2023)
- GitHub SARIF support docs: https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning (accessed 2026-10-12)