# Experiment C exposure log (append-only)

Every way the frozen corpus has been examined before the official
evaluation. Not one of the five frozen evaluation files; it changes none
of them. Entries are appended, never removed or reworded.

## Entries

- 2026-10-13, all 30 repositories, ground-truth audit (directive 58317):
  producer-consumer field relationships were examined by hand in every
  repository, by the same engineer who implements the engine. This is
  the widest exposure; it is inherent in a single-rater design (see
  EVALUATION-PROTOCOL.md, measurement limitations).
- 2026-10-13, all 30 repositories, h2.py (rough H2 impact script):
  workspace markers only; no source files or field reads.
- 2026-10-14, biosshot/easyeda-copilot (POSITIVE), feasibility review
  (directive 73186): source of this repository only; no other corpus
  repository's source was inspected.
- 2026-10-14, biosshot/easyeda-copilot, fixture F01, slice 1 development
  (directive 86427): the full `invar init` output (6 proposals, 79
  abstentions), `invar lock` and `invar check` on the clone at fe7b2a5
  were run and read. No other corpus repository was run through the
  slice 1 engine.
- 2026-10-14, biosshot/easyeda-copilot, directive 86428 passes: `invar
  init`, `lock` and `check` were re-run on the clone at fe7b2a5 after
  each correction pass, and the resulting invar.toml and invar.lock
  were read. No other corpus repository was run or read.

## Sensitivity analysis, specified before any result

When the official evaluation runs, it is reported exactly under
EVALUATION-PROTOCOL.md. Alongside, labeled SENSITIVITY ANALYSIS, with no
threshold: primary coverage over POSITIVE repositories not listed above
as development cases (denominator 10 while only easyeda-copilot is
listed), and precision over proposals emitted in repositories not
listed as development cases. The audit and h2.py entries are not
development cases and do not change these denominators. This analysis
never replaces the official figures.

## Slice 2 development probes, 2026-10-15

Read-only probes of the slice 2 engine on sparse checkouts, run as development work during implementation. Both repositories are DEVELOPMENT cases (S2-D5) and are excluded from any Stage 3 evaluation corpus. Neither run is an evaluation result.

- formbricks, sparse checkout at HEAD 062a49126: the pre-F1 engine produced 2 egress proposals, both judged likely request-side (the trigger of finding F1), and 1133 alias-UNRESOLVED lines. F1 was then remediated (response markers restricted to call and constructor positions). The probe was not re-run after the fix.
- hoppscotch, sparse checkout at HEAD 63273f8e: 0 proposals in 0.4s, consistent with the verzod negative control K1-N2 (versioned schema family abstains).

## Slice 2b record, 2026-10-15

No real-repository probe was run for slice 2b. K3 was developed and calibrated on synthetic fixtures only (18 fixtures, 12 mutations) and one synthetic scale run (3200 generated files, 0.74s). Strapi (e3961fb2~1 and fix e54e5f7d) remains a DEVELOPMENT case under S2-D5, excluded from any Stage 3 corpus; its string-list mechanism is reproduced synthetically in fixtures K3-F1, F3 and F4, not measured on the repository. Nothing in this entry is an evaluation result.

## Slice 2b development probes on real repositories, 2026-10-15

DEVELOPMENT probes under S2-D5, read-only: `git archive` of the pinned commits from the existing local clones into fresh directories, then `invar init --stdout` at 34f1e52. No clone, worktree or manifest was modified. Single rater (the engineer), triage from source reading, no second rater. These are NOT evaluation results; none of these repositories may enter a Stage 3 corpus. Pins: formbricks 062a49126382, hoppscotch 63273f8e599e, strapi c8af2cef6b05 (= e3961fb2~1, the tree before the lastUsed rename).

### Yield

| Repo | Files | `strings` proposals | Time | K3 abstentions |
|---|---|---|---|---|
| hoppscotch | 2495 | 0 | 0.94s | 0 (negative control holds) |
| formbricks | 6023 | 13 | 3.6s | AMBIG-RECEIVER 17, NON-DISTINCTIVE-KEY 9, VERSION-FAMILY 1 |
| strapi | 3587 | 13 | 1.0s | AMBIG-RECEIVER 8, NON-DISTINCTIVE-KEY 3 |

Abstentions were counted, not triaged for correctness.

### Triage of all 26 proposals (classification from source)

| Class | formbricks | strapi |
|---|---|---|
| TRUE coupling, not type-governed | 2 (SEND_EMAIL_PII_FIELDS, SEND_EMAIL_REQUIRED_CONTENT_FIELDS vs ZWorkflowSendEmailActionConfig) | 11 (admin User fieldsToPick x8 incl. 6 in one test file, Permission fields x3) |
| TRUE coupling but tsc-governed (`as const satisfies readonly (keyof TBaseStyling)[]`) | 3 (THEME_DIMENSION, FONT_WEIGHT, COLOR fields vs ZBaseStyling) | 0 |
| UNCLEAR | 1 (requiredAttributes: attribute keys loosely mapped to ZContactTableData) | 0 |
| FALSE, wrong receiver | 7 (ContactInfo survey-element field lists vs ZContactTableData) | 2 (users-permissions field-permission lists vs the admin User shape) |

Strict precision (true and not type-governed): formbricks 2 of 13, strapi 11 of 13, pooled 13 of 26. False-receiver rate: formbricks 7 of 13, strapi 2 of 13, pooled 9 of 26 (35%). Strapi's 11 true proposals include 6 near-duplicates in one test file, so distinct couplings are fewer.

### Findings

- P1, recall on the motivating case: the strapi break pair (SELECT_FIELDS in packages/core/admin/server/services/api-token.js against the api-token content type) was NOT proposed. The list is id, name, description, lastUsed, type, createdAt, updatedAt; after the frozen stoplist only `lastUsed` is distinctive, so the engine abstained NON-DISTINCTIVE-KEY (1 distinctive; needs 3). Correction to the record: the feasibility probe's "lastUsed and three others" counted stoplisted words; synthetic fixture K3-F1 modelled the case with extra distinctive names and so overstated capture. At D1 as ruled, K3 misses the one real break it was chartered around. No threshold was changed.
- P2, dominant noise is wrong receiver on generic personal-field names (firstName, lastName, email, phone) shared by unrelated concepts in one package; the unique-receiver rule cannot fire when only one shape qualifies.
- P3, type-governed lists: lists declared `satisfies readonly (keyof T)[]` are checked by tsc and add nothing; 3 of 26.
- P4, category labels: `testUtils` paths were labeled production and `server/` paths unknown; one proposal (strapi data.js) was a test fixture labeled production.
- P5, near-duplicate proposals: 6 strapi proposals come from one test file.
- Scale: 6023-file tree in 3.6s including slice 1 and 2 passes (slice 1 alias-UNRESOLVED lines dominate output: 7650 on formbricks).

### Candidate remedies (none applied; each needs a ruling and a revision)

(a) abstain TSC-GOVERNED on `satisfies ... keyof` lists (principled, by construction); (b) harden test-path categories (`testUtils`, `__mocks__`, `fixtures`); (c) collapse same-file same-shape duplicates into one proposal with multiple list sites; (d) for P1 and P2 no rule change is recommended from one missed case and one noise class on development data alone: the choices are to keep D1 and accept the miss, to add a containment rule (all literal names found in the shape, raw overlap of at least 4) which would still miss this list because createdAt and updatedAt are not declared attributes, or to widen evidence with a usage cue (list passed to pick, select or populate calls). Any such change must be fixture-first and validated on an unseen repository set, not tuned on these three.
