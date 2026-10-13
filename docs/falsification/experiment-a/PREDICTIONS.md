# Experiment A, mining half: predictions and frozen method (before any data)

Recorded 2026-10-13, before any candidate server was examined.
Thresholds come unchanged from docs/falsification/vacancy-gpass-2026-10-13.md.

## Frozen method

Frame: public MCP servers whose source DECLARES output schemas
statically (an `outputSchema` JSON Schema literal, a zod/pydantic schema
passed as outputSchema, or a committed tools/list snapshot), with at
least 3 tagged releases. Servers whose output schema exists only at
runtime (for example derived from Python return annotations) are counted
as "not statically extractable" and excluded, stated. No third-party
server code is executed at any point (report G's capacity gate).

Unit of change: (server, tool, consecutive tag pair), for a tool present
in both tags. A BREAKING output change is any of:
1. a property that was `required` is removed from properties or from
   required (a consumer reading it may now find it absent);
2. an existing property's declared type changes;
3. a property is added to an object declared closed
   (additionalProperties false), which breaks clients holding the
   previous tools/list (MCP issue cyanheads/mcp-ts-core #601 class).
Tool removal or rename is definition drift, not output-contract change,
and is excluded and counted separately. Adding optional properties to
open objects is non-breaking.

Consumer: a public repository, other than the server's own, whose code
reads a field of that tool's structured output by name. Found by search;
the search method is recorded with results.

## Predictions

- M1 (frame): reaching 30 qualifying servers requires examining many
  more candidates; central estimate 150+, because most servers declare
  no outputSchema and many lack 3 tags.
- M2 (breaks): FEWER than 8 breaking output changes across the frame;
  central estimate 3 to 6. Reason: churn is concentrated in a few
  servers (The Ops Log) and the open-schema convention suppresses class 3.
- M3 (consumers): 0 to 2 breaking changes with an identifiable public
  consumer.
- Overall: REFUTE or inconclusive is more likely than CONFIRM. If the
  frame cannot reach 30 qualifying servers at all, that is itself a
  finding (the checkable surface is too thin) and is reported as such,
  not papered over by loosening the frame.

Falsified predictions are reported as falsified.

## Method addendum A1 (frozen 2026-10-13, after stages 1-2 counts, before any stage-3 schema data)

Observed so far (counts only, no schema content): registry crawl with
version=latest returned 42,777 records; about 43% of reachable GitHub
repos have at least 3 tags; about 20% of those mention outputSchema or
output_schema somewhere in non-test source at HEAD (keyword hits only).

Stage-3 parameters, fixed now:
- Sampling: candidates are the stage-2 keyword hits, shuffled with
  random.Random(20261013); examined in that order until 30 QUALIFYING
  servers are reached. Every examined candidate is logged with its
  disqualification reason, so the frame denominator is reported.
- Qualifying: a statically extracted output schema (JSON literal, zod
  object or raw shape, JSON file) for at least one tool present in at
  least 2 of the examined tags, AND the repo is a server (declares
  tools), not a client reading others' schemas.
- Tags examined: the last 10 tags by creation date (or all, if fewer).
- Files: the stage-2 paths at HEAD, read at each tag; renamed or moved
  files are not followed (stated limitation, same as backtest).
- Unresolvable schemas (identifier defined in another file, runtime
  generation such as pydantic model_json_schema or Go reflection) are
  counted as "not statically extractable", never guessed.
- Every automatically flagged breaking change is verified by hand
  against the actual git diff before it counts. Automatic flags that
  fail verification are reported as extractor false positives.

## Smoke-test corrections (before the seeded sample was drawn)

The stage-3 runner was smoke-tested on two stage-2 hits outside the
experiment log (0rkz/byte-mcp-server, 1102tools/federal-contracting-mcps).
Three extractor defects were found and fixed BEFORE sampling: multi-line
zod chains (z on one line, .array( on the next) and inline comments
broke raw-shape parsing; declared-but-vacuous schemas (type object, no
properties) were silently dropped and are now counted as
"vacuous-no-properties"; committed tools/list JSON snapshots were not
recognized as server declarations. The synthetic break-class suite
passes after the fixes. Neither smoke-test repo's result enters the
record unless it is independently drawn by the seeded sample.

## Mid-run correction (run 1 aborted after 29 examined, 0 flags seen)

Run 1 misclassified framework-based servers as "not-a-server": its
server detector looked only for registration calls inside the files
that mention outputSchema, while frameworks (e.g. cyanheads/mcp-ts-core)
define tools in separate definition files. Fix: a file also counts as
server evidence when it contains inputSchema/input_schema, the
mcp-ts-core marker, a tool( builder call, or lives under a tools/ path.
Run 1 is preserved as stage3-aborted-run1.jsonl and does not count.
Stage 3 restarts from the same seed. Extraction keys are UNCHANGED:
framework conventions that declare output under another key (mcp-ts-core
uses `output:`) stay outside the frozen frame; this likely biases the
frozen result toward FEWER breaks. A separate sensitivity run that adds
the `output:` key will be reported alongside, never merged into the
frozen counts.
