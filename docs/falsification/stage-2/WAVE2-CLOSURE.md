# Stage 2, Directive 13002: wave 2 evidence closure

Status determination, per the preregistered rules and nothing else:
**BLOCKED-INSUFFICIENT-CORPUS.** Not CONFIRMED (no qualifying corpus to
evaluate), and not REFUTED (refutation requires a corpus meeting the
section 4 minimum, which was not reached). General detection capability
remains NOT ESTABLISHED. No threshold, classification rule, or protocol
text was changed after any result was seen.

## 1. Protocol identity

STAGE-2-PROTOCOL.md as frozen in dd52067 plus the ratified pre-run-1
amendments A1-A4 in 1300b1b. Engine under test: slice 1 at 4a7bab6.
**The engine was never run on any mined repository**: zero engine runs,
zero VIOLATED verdicts, so the noise-rate denominator (total VIOLATED
verdicts, the ruled definition) is zero and no gate is evaluable.

## 2. Search-scope inventory (32 repositories, all logged before any run)

Batch 1 (8): novu, cal.com, twenty, refine, logto, formbricks,
documenso, teable. Batch 2 (8): immich, medusa, payload, trigger.dev,
infisical, hoppscotch, nocodb, dub. Wave 2 under the A1 cap (16,
selected for tsc blind spots): strapi, gatsby, redwood, ToolJet,
budibase, directus, storybook, astro, docusaurus, n8n, wp-calypso,
gutenberg, material-ui, expo, Ghost, chakra-ui. The A1 cap is exhausted.

## 3. Funnel accounting

| Stage | Count |
|---|---|
| Raw drift events (removed exported-shape members in shared-type files) | 482 (112 + 221 + 149) |
| Distinctiveness-filtered at source (the 16 A1 repos) | 149; earlier batches filtered during analysis |
| Events with fix-forward message signals | 239 |
| Drift commits with surviving reads outside the producer package (worktree sweep) | 75 |
| Case-level curations completed to a classification | 8 |
| ACCEPTED | 1 (external-boundary column) |
| REJECTED with evidence | 7 |
| UNRESOLVED (shortlist signals not curated within the cap; none showed the Category C shape on first inspection) | ~20 lead events |
| C-ELIGIBLE accepted cases | **0 of the 12 minimum** |

Effort: approximately 4 hours of attended mining and curation across
2026-10-14 and 2026-10-15, plus roughly 3 hours of unattended machine
mining. Approximations, recorded as such.

## 4. Case ledger (every classification evidence-backed, from local clones)

| Case | Repo, drift | Classification | Evidence |
|---|---|---|---|
| S2-EB-01 | formbricks 939fedfc (environmentId dropped in the environment-to-workspace rename) | ACCEPTED, EXTERNAL BOUNDARY column (never counts toward C, per A2) | Fix e114d0a4 (#8855) documents in its own code comment that v1 integrations (Zapier, Make, n8n) read environmentId at runtime for ~3 months; legacy layer restored. Intra-repo reads at the drift commit are deliberate compat code, so the intra-workspace window was closed atomically |
| novu 92b5be29 | approvals, inputTokens, outputTokens removed | REJECTED | Candidate fix 54f8916c documents orphaned tool approvals, a behavioral gap in agent(), not a consequence of the removed fields |
| teable 3d254ef5 | cellValueType | REJECTED | Candidate fix is two years later and does not tie the defect to the drift |
| hoppscotch e8ed938b | folders, requests, headers | REJECTED | Members moved into versioned zod schemas (InferredEntity, v/4, v/5) with translators carrying old shapes via fallbacks; compatibility preserved, no consumer contract unsatisfied |
| documenso 8448e333 | LimitOptions removed | REJECTED | The whole type was deleted; git grep at the drift commit finds zero consumers; dead-code removal |
| strapi e3961fb2 | lastUsed renamed to lastUsedAt | REJECTED for scope, recorded as a real out-of-scope break | At the rename commit the tree is consistent; a concurrent branch merge reintroduced 'lastUsed' strings (merge skew); fix e54e5f7d four days later repairs a JS SELECT_FIELDS string list. Real, type-blind, runtime-adjacent, but same-package and JS-era: fails section 3 |
| wp-calypso fd8b63b2 | site_icon, stats_visitors removed from packages/api-core dashboard-site-list type | REJECTED | The type mirrors an external API's response; surviving .site_icon JS reads are different receivers (site-settings objects); removing a member from a type that describes external data changes no runtime value |
| expo 7aec4eb1 | edgeToEdge revert in @expo/config-types | REJECTED | The prebuild-config types fix is 15 months later, after edge-to-edge was reintroduced; signature cleanup, not a drift consequence |

## 5. Gate comparison

- CONFIRM (at least 3 Category C, noise rate below 15% of VIOLATED
  verdicts): not evaluable; 0 C-eligible cases, 0 verdicts.
- REFUTE (0 Category C over a corpus meeting the minimum, or noise rate
  at or above 30%): not available; the minimum was not met.
- Result: BLOCKED-INSUFFICIENT-CORPUS under section 4 and amendment A1.
  No further waves are authorized without a new ruling.

## 6. The mechanism finding the corpus forced into view

Across 32 repositories the Category C cell stayed empty for a reason
that is now articulable: a type-surface lock can only alarm on TYPE
drift, and type drift has runtime consequences only where the type
co-moves with an in-workspace VALUE producer. Typed consumers of such
pairs are updated atomically because tsc forces it. Type-blind
consumers are not constrained by the type at all, so their real breaks
come from VALUE drift the type never records. Both verified real breaks
found by this mining lived exactly there: formbricks (external API
payload changed; internal code green) and strapi (a string field list
against a renamed DB column, reintroduced by merge skew). Each sits
outside slice 1's observation field, one at the external boundary, one
below the cross-package bar.

Implication, stated for a future ruling and NOT authorized here: the
next slice with a live Category C cell binds value producers (emitted
or literal shapes, string field lists, the repository's own published
contract surface) rather than type declarations alone.

## 7. Stop-condition check

No protocol or threshold was altered after outcomes were visible; no
classification relied on evidence outside the authorized scope; the
distinctiveness filter was committed per A4 before wave 2 ran and did
not change the sampling method of the earlier batches (their events
were re-filtered during analysis, with the unfiltered counts preserved
above); the minimum evidence was not obtainable within the cap, which
is the stop condition that fired.
