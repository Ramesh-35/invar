# Corpus data: export, scoring, licensing

The real cross-repository break corpus is published as data so anyone
can verify or challenge the record without trusting this repository's
binaries.

## Files

- `invar-corpus.jsonl`: one JSON object per line; a meta record, then
  122 case records sorted by id. Regenerate with
  `python3 scripts/export-corpus.py` (stdlib only); the output is
  byte-deterministic. Each case carries: id, dir, title, source (the
  public PR/commit/issue evidencing the break), source_kind, admitted
  date, real flag, break_kind, batch, class (break or control),
  held_out flag, expected residue, evidence line, baseline tool
  results as scored, and the pinned invariant (producer, producer_base,
  consumer references with commit SHAs).
- `docs/falsification/HELD-OUT-MANIFEST.txt`: the sealed held-out set
  (70 in-scope breaks + 15 named controls, batches 4-8), committed
  while the repository was private; its BLAKE3 is recorded in
  docs/epistemic/STATUS.md in the same commit that added it.
- `scripts/score-corpus.py`: standalone scorer, stdlib only, no INVAR
  engine. Takes the JSONL plus any tool's results
  (`{"<case id>": {"verdict": "violated|holds|not_evaluated|error",
  "residue": [...]}}`) and prints per-case outcomes and a count-form
  summary in the same vocabulary as the committed scoring TSVs.

## Scoring rules, stated

A residue entry matches an expected field by exact name, as the final
`.field` of a `Type.field` form, or across naming forms (case and the
_/- separators ignored), which is the engine's own binding rule and
was the rule in force in the committed scoring TSVs; nothing else is
credited. In
particular, an ancestor path that structurally subsumes an expected
field (`packages` reported where `specifiers` lived beneath it, as
INVAR's own observed-reality ancestor dominance prints) is NOT
credited by this scorer; it scores field-level static findings. Extra
residue is counted and printed, never hidden. Summaries are counts,
never percentages.

## Licensing

- Corpus data (corpus-real/ case metadata, invar-corpus.jsonl, the
  scoring TSVs and the held-out manifest): CDLA-Permissive-2.0, see
  LICENSE-DATA.
- Engine and scripts: MIT OR Apache-2.0, as the repository LICENSE
  states.
- demo/ contains unmodified third-party source files under their own
  licenses, listed in demo/NOTICE; they are not covered by
  LICENSE-DATA.
- The corpus references third-party repositories by pin (owner, repo,
  commit SHA, path); it does not contain their file contents.

## Reproducing the held-out record

The 38 of 70 record reproduces from public data alone. The engine of
record per batch is each batch's FIRST scoring, on the engine frozen
before that batch was sourced (later rescorings of old batches are
tuning data, by the project's own rule): batch 4
r1-batch4-results.tsv (6 caught of 10), batch 5 r1-batch5-alpha5.tsv
(7 of 15), batch 6 r1-batch6-alpha8.tsv (9 of 15), batch 7
r1-batch7-alpha8.tsv (8 of 15), batch 8 r1-batch8-alpha10.tsv (8 of
15). Those verdicts, converted mechanically to the scorer's results
format, are committed as
docs/falsification/heldout-engine-of-record-results.json. Run:

    python3 scripts/score-corpus.py --corpus invar-corpus.jsonl \
      --results docs/falsification/heldout-engine-of-record-results.json \
      --held-out-only

Expected output: breaks caught 38 of 70; controls 13 CLEAN, 1
FALSE_POSITIVE (the batch 5 spec-$ref case, long on the record), 1
UNEVALUATED; 5 break UNEVALUATED; 25 extra residue items stated.
