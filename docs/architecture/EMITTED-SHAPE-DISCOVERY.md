# EMITTED-SHAPE-DISCOVERY (slice 2): feasibility review and draft architecture

Status: DRAFT for ruling. Implementation NOT authorized. Grounded in the
terminal Stage 2 record (docs/falsification/stage-2/WAVE2-CLOSURE.md):
32 repositories, 0 Category C cases on type surfaces, 2 verified real
breaks, both on VALUE surfaces. Probes below were run read-only on the
existing clones on 2026-10-15; no engine was built or run.

## 1. What slice 2 is for

Slice 1 locks TYPE declarations. The mining showed type drift is closed
atomically by tsc wherever a typed consumer exists, and type-blind
consumers are not constrained by types at all, so the breaks that reach
production live where VALUES drift: emitted payloads, string field
lists, published contract surfaces. Slice 2 moves init's discovery and
the lock to those value surfaces.

Record corrections carried into canon with this document:
- Formbricks (S2-EB-01): about 3 months of broken v1 integrations until
  the compatibility layer, documented in the fix commit. "Outage" is not
  established by the evidence; broken integrations are.
- Strapi lastUsed: a 4-day window on the main branch in which a JS field
  list selected a renamed column through the query layer; production
  impact is not established by the evidence; the broken window is.
- "Slice 1 adds zero value on surface 1" overstates: it adds none where
  tsc gates the change, and the corpus found zero verified cases where
  it did not; the measured claim is 0 of 32, not an impossibility proof.

## 2. Three contract kinds, with probe numbers

K1. EGRESS LOCK (the formbricks class): a self-contract on the
repository's own published emitted shape. No consumer evidence needed;
the lock is the compatibility promise at a declared boundary.
- Probe P3 (formbricks HEAD): 114 route.ts files under app/api, 53
  under versioned v1/v2 paths; only 3 files emit raw Response.json
  literals, while 84 call sites emit through wrappers. Conclusion: the
  emitted shape is usually NOT a literal at the route; it is the zod
  schema the route serializes. K1 therefore binds to schema VALUE
  surfaces (K2), with the route file as the boundary evidence.
- Check semantics are additive-tolerant: removing or renaming a locked
  emitted field alarms; adding holds. This is a new contract kind in
  the manifest (a new `kind` value), same lock format, same five exit
  codes; the ISA does not change shape.

K2. ZOD-VALUE PRODUCERS: `z.object({...})` literals read as value
surfaces (object keys), not types.
- Probe P1 (formbricks): 315 z.object literals in packages/types alone;
  their member lines are plain `name:` keys, which the existing
  assembled_keys scanner already parses; no AST needed (R1 holds).
- This converts slice 1's dominant abstention into coverage: on F01, 73
  of 79 abstentions were UNSUPPORTED-SHAPE, mostly z.infer.
- Honesty bound, stated up front: consumers typed via z.infer are tsc
  territory; K2 pairs propose only for type-blind consumers (JS files,
  bracket reads) and otherwise feed K1 as the emitted-shape source.

K3. STRING-COUPLING CONTRACTS (the strapi class): a schema-literal's
keys versus string field lists that index the same store.
- Probe P2 (strapi at e3961fb2~1): producer = content-type `attributes`
  object keys (name, description, type, accessKey, lastUsed,
  permissions); consumer = `SELECT_FIELDS = ['id','name','description',
  'lastUsed','type',...]` in the same package. Overlap includes the
  distinctive lastUsed and three others; both sides pattern-extractable
  with existing machinery. The verified break is precisely this pair
  drifting through merge skew.
- K3 requires same-package scope (a new ruling; N08 remains the rule
  for type pairs) and is the noisiest kind: see CM1.

## 3. Hard constraints, inherited

Pattern extraction only, no AST, no new dependencies (R1); the manifest
and lock formats extend by new `kind` values and evidence keys only;
the five exit codes are untouched; check.rs and lock.rs changes, if
any, are named scope expansions in the implementation directive, never
assumed.

## 4. Countermodels the contract must survive (fixtures before code)

- CM1 receiver collision: a string list naming fields of a DIFFERENT
  store that happens to share names. Pairing requires at least 2
  distinctive overlapping fields plus proximity evidence (same file,
  same directory, or an import path between the files); otherwise
  abstain with a stated code.
- CM2 additive egress: a new field added to a locked emitted shape must
  HOLD.
- CM3 versioned schemas (the hoppscotch class): verzod or v/N schema
  families with translators preserve compatibility and must not alarm;
  a version-family detector abstains rather than pairing across
  versions.
- CM4 dynamic or spread keys: `...rest`, computed keys, `.extend`/
  `.passthrough` chains make the surface unrepresentable; abstain,
  never guess.
- CM5 test and fixture lists: category-labeled as in slice 1, never
  presented as production usage.

## 5. Decision points for rulings before any implementation

- S2-D1 slice boundary: recommend slice 2 = K1 + K2 (egress locks fed
  by zod-value surfaces: single-repo self-contracts, no pairing noise,
  the clearest product story, and the formbricks class covered); K3 as
  slice 2b with its own countermodel pass, since same-package scope and
  string pairing carry the noise risk.
- S2-D2 boundary identification for K1: path evidence (api/v1-style
  versioned routes), package.json "private": false, or an explicit
  marking in invar.toml; every egress proposal states which evidence it
  used.
- S2-D3 K1 check semantics: removal, rename, or incompatible
  re-type of a locked emitted field alarms; additions hold; lock
  update is the declared way to make a breaking change.
- S2-D4 K2 pairing rule: type-blind consumers only (JS, bracket reads),
  with the slice 1 evidence discipline (E-lines, evidence keys, lock
  narrowed to evidence).
- S2-D5 exposure: formbricks and strapi are now DEVELOPMENT cases
  (mined, read, probed) and are excluded from any Stage 3 evaluation
  corpus from this document forward.
- S2-D6 evaluation: a Stage 3 protocol in the Stage 2 mold, frozen
  before any engine result; the Stage 2 miner is reusable with
  value-surface detectors (string lists, zod literals, route files);
  seed break classes are the two verified ones, as development
  references only.

## 6. Market honesty

Where OpenAPI or JSON Schema exists, spec-diff tools already detect
egress breaking changes; Experiment B measured 24 of 30 real monorepos
with no schema files at all, and that unspecified majority is K1's
ground. The agent-oracle framing lands here too: an agent's refactor
that drops an emitted field gets a local, deterministic VIOLATED before
the PR exists. Both claims remain product hypotheses until a Stage 3
run says otherwise; nothing here asserts established capability.

## 7. Feasibility verdict

- K1: FEASIBLE with K2 as its shape extractor; the boundary detector is
  the main open design (S2-D2); probe evidence above.
- K2: FEASIBLE with existing extraction machinery; measured surface is
  large; honesty bound on typed consumers stated.
- K3: FEASIBLE mechanically (probe P2), gated on the same-package
  ruling and CM1; recommended as slice 2b.
- Overall: a slice 2 contract document (INIT-DISCOVERY-SLICE2, with
  numbered requirements, abstention codes, fixtures F/N/U/M, a mutation
  set, and gates 0 to 5 as in 86427) can be drafted on acceptance of
  S2-D1 through S2-D6. Implementation remains NOT authorized until that
  contract is ruled on.
