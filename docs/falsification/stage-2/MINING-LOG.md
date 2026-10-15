# Stage 2 mining log (append-only)

Every repository examined while mining the Stage 2 corpus, appended
BEFORE any engine run on it, with what was read and whether a case was
accepted. Governed by STAGE-2-PROTOCOL.md section 4: mining criteria
only, blind to the engine; the corpus list freezes in this file before
run 1. Entries are appended, never removed or reworded.

## Entries

Method, batch 1 (2026-10-14): blobless clone; shared-type files
located by path pattern (up to 45 per repo); per-file history diffs
scanned for removed exported-shape members (stoplist applied); then a
fix-forward message grep (later commits whose message names the removed
member or shape with fix-like words). History and source reads only; NO
engine run on any of these repositories yet.

- 2026-10-14, novuhq/novu: mined; 17 drift events, several with
  fix-forward signals (approvals/inputTokens 92b5be29 strongest).
- 2026-10-14, calcom/cal.com: mined; 10 drift events; fix-forward
  signals mostly generic-name noise.
- 2026-10-14, twentyhq/twenty: mined; 1 drift event; no signal.
- 2026-10-14, refinedev/refine: mined; 23 drift events; strongest is
  successNotification 5a805c78 -> c630b0905 (type-level fix, likely
  Category A/B, kept for the comparator).
- 2026-10-14, logto-io/logto: mined; 3 drift events; weak signals.
- 2026-10-14, formbricks/formbricks: mined; 31 drift events; strongest
  lead of the batch: 939fedfc (Formbricks 5) removed environmentId from
  shared types; later fixes ef18aacf (#8079) and e114d0a4 (#8855)
  document restoring legacy environmentId on API v1 surveys and
  webhooks. Caveat: types are largely zod inferences, so slice 1 may
  abstain (UNSUPPORTED-SHAPE); the case can still enter the corpus and
  score honestly.
- 2026-10-14, documenso/documenso: mined; 8 drift events; envelopes
  migration (7f09ba72, d05bfa9f) has field removals with later API
  fixes; needs R verification.
- 2026-10-14, teableio/teable: mined; 19 drift events; cellValueType
  3d254ef5 with later e9655c2d fix is a candidate; needs R check.

No case is accepted yet; acceptance requires the section 3 R evidence
(documented runtime consequence), verified from the fix PR or issue
text, and a C_prior scope check. The corpus freezes in this file before
run 1.

Batch 1 curation (2026-10-15), primary git evidence in local clones:
- formbricks/formbricks 939fedfc: ACCEPTED as candidate case S2-01. R
  established from fix commits ef18aacf (#8079) and e114d0a4 (#8855,
  2026-08-19): v1 API integrations (Zapier, Make, n8n, named in the
  fix's own doc comment) kept reading environmentId at runtime for ~3
  months after the rename; a legacy compatibility layer was restored.
  Structure notes: shapes are zod (slice 1 likely abstains); surviving
  intra-repo reads at the drift commit are deliberate compat code
  (js-core migration, workspace-resolver fallbacks), so the intra-repo
  window was closed atomically and the breakage was external-boundary.
- novuhq/novu 92b5be29: REJECTED. The candidate fix 54f8916c (NV-9007)
  documents orphaned tool approvals, a behavioral gap in agent(), not a
  consequence of the removed AgentEventContext fields.
- teableio/teable 3d254ef5: REJECTED. The candidate fix e9655c2d is two
  years later and does not tie the defect to the drift.
Method addition (2026-10-15): survivors detector, worktree checkout of
the drift commit plus ripgrep for reads of the removed member outside
the producer directory, classifying .js/.jsx, bracket access, and .ts
sites. Applied to formbricks 939fedfc (12 surviving reads, all
deliberate compat). Batch 2 mining resumed: immich-app/immich (mined, 1
event), medusajs/medusa (mined, 3 events), then payloadcms/payload,
triggerdotdev/trigger.dev, infisical/infisical, hoppscotch/hoppscotch,
nocodb/nocodb, dubinc/dub. Still NO engine run on any mined repository.
