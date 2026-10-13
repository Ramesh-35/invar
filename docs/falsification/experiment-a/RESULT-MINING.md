# Experiment A, mining half: verified result (2026-10-13)

Thresholds, frame and method: PREDICTIONS.md (frozen before data,
addendum A1, smoke-test corrections, run-1 abort recorded).

## Frame funnel (measured)

Official MCP registry, version=latest: 42,777 records, 24,335 unique
GitHub repos; 20,872 reachable; 8,062 with at least 3 tags; 1,878 (23.3%)
mention outputSchema/output_schema in non-test source. Seeded sample
(random.Random(20261013)): 98 examined to reach 30 qualifying
(58 had no statically extractable schema present in 2 tags, 10 were not
servers). Over the 30: 229 release transitions, 4,713 tool-pair
comparisons. Static extraction coverage is poor and stated: across all
examined tags, 3,354 vacuous output schemas (object, no properties),
3,544 unparsed object shapes, 2,067 other expressions, 1,376 schemas
defined in another file, 455 unparsed zod objects.

## Breaking output-change events, every flag verified by hand

71 automatic flags = 45 (server, tool, release) events.
- VERIFIED REAL, detected on the correct field: 18
  - cyanheads/git-mcp-server v2.14.2 to v2.15.0: 3 required output
    fields renamed (git_blame file to filePath; git_clone localPath to
    path; git_branch operation to mode).
  - the-40-thieves/obsidian-tc v1.31.8 to 1.32.0: 14 tools made
    required output fields optional (GH #1027, response_format=concise);
    v1.32.0 to v1.33.0: get_attachment.content made optional (1).
- VERIFIED REAL, flagged under the wrong (top-level) field: 6
  - ankimcp/anki-mcp-server v0.24.2 to v0.25.0: get_due_cards and
    get_cards cards[].back required to optional, omitted by default;
    v0.25.0 to v0.25.1: notesInfo notes[].mod required to optional.
  - obsidian-tc 1.32.0: nested optionality in find_unresolved_links
    (kind), get_backlinks (display), get_outgoing_links (candidates).
- EXTRACTOR FALSE POSITIVES: 19 events (all 17 metabase-ai-assistant
  events were positional-key artifacts, zero breaks when keyed by name;
  alza-mcp optional nested addition; obsidian-tc add_observation).
- INCONCLUSIVE, not counted: 1 (obsidian-tc reset_vault_cache).
- Incidental, outside the frozen count: metabase-ai-assistant v4.2.1
  widened list_users users[].first_name from string to string|null.

TOTAL VERIFIED BREAKING OUTPUT CHANGES: 24 (18 strictly on-target).

## Against the frozen thresholds

- Condition 1 (at least 8 breaking changes across at least 30 servers):
  MET (24; 18 strict).
- Condition 2 (at least 3 with an identifiable public consumer):
  PENDING. Requires GitHub code search, unavailable from the engineer's
  environment; founder runs it.
- Condition 3 (at least 3 of 15 maintainers positive): PENDING, founder.

## Predictions scored

- M1 (frame hard to reach): FALSIFIED. 30 qualifying reached at 98
  examined, from a 1,878-repo hit frame.
- M2 (fewer than 8 breaks): FALSIFIED. 24 verified.
- M3 and overall: pending conditions 2 and 3.

## What the data says beyond the thresholds

Concentration: all 24 breaks are in 3 of 30 servers (10%) and 5 of 229
release transitions (2.2%); one feature in one server produced 18. This
matches the external finding that churn concentrates in a small set of
servers. The checker's value, if any, is concentrated in exactly those
actively developed servers. The extractor undercounts: it reads top-level
fields only and could not resolve most schemas, so 24 is a floor.

## Addendum: disclosure check (2026-10-13, after the count, before any outreach)

All three servers with verified breaks DOCUMENTED them as deliberate in
their own changelogs: cyanheads/git-mcp-server v2.15.0 ("Breaking:
Renamed inputs and outputs on git_add, git_blame, git_branch, and
git_clone", each rename listed, plus old-name rejection tests);
ankimcp/anki-mcp-server v0.25.0 ("BREAKING: get_due_cards / get_cards
no longer return back by default") and v0.25.1 (mod no longer required,
fixes #66); the-40-thieves/obsidian-tc 1.32.0 ("the advertised output
schemas mark the concise-dropped fields optional", in four entries).

Consequences recorded:
1. Producer-side detection is not where the pain is. Maintainers who
   break output contracts here know and announce it. The exposure is on
   CONSUMERS holding the old schema, which is where INVAR's locks live.
2. Condition 3 as frozen (offer maintainers "a consumer lock") cannot be
   run honestly: INVAR does not parse MCP output schemas today, and these
   maintainers have no detection gap to fill. It is NOT run as written;
   the reason is recorded here instead of a fabricated outreach result.
   Candidate A therefore cannot reach CONFIRM under the frozen rule.
3. Condition 2 (public consumers reading the broken fields) remains the
   informative test, and runs next.
4. One in-the-wild instance of the declared-versus-observed class:
   obsidian-tc 1.32.0 notes index_vault returned a field its declared
   output schema omitted, so "a strict client rejected every detailed
   result". One instance, an anecdote, not a rate.

## Condition 2: public consumers (scored 2026-10-13). RESULT: REFUTE

Search: founder-run `gh search code`, four queries (chunks_unchanged,
obsidian-tc, @cyanheads/git-mcp-server, get_due_cards), 30 results
each, default branches of public repos only. Every plausible hit was
cloned and read by the engineer.
- shaqmughal/seekstone (benchmark adapter): a real code consumer of
  obsidian-tc, reading search_text items[].path, read_note .content and
  list_notes notes[].path. None of these is among the 24 broken fields;
  reads are defensive (`??` fallbacks); pinned to 1.15.0. Not counted.
- provos/ironcurtain: no field reads. Documents that
  @cyanheads/git-mcp-server returns error structuredContent that does
  not match its declared outputSchema, and ships an always-valid
  validator to bypass SDK output validation entirely.
- zouwei/moraya, neiii/bridle: agent configuration presets only.
- microsoft/conductor: installs @cyanheads/git-mcp-server@2.15.1,
  after the break; configuration only.
- All other hits: CVE databases, directories, the servers' own repos,
  or unrelated code sharing a common name.
Identifiable public consumers reading a broken field by name: 0.
Frozen REFUTE condition "zero identifiable consumers" is met.

Predictions scored: M3 (0 to 2 consumer-backed breaks) CONFIRMED (0).
Overall (REFUTE or inconclusive more likely than CONFIRM) CONFIRMED.

## What the refutation teaches (recorded, not a new claim)

1. MCP tool outputs are consumed mostly by models at runtime, not by
   code. The consumer-side lock ("lock the fields your code reads") has
   almost nothing to read: the reader is an agent, wired in by config.
2. The pain observed twice in the wild is declared-versus-observed
   mismatch under strict SDK validation (obsidian-tc index_vault;
   git-mcp-server error responses), and one consumer's response was to
   disable output validation altogether: the disarmed-gate pattern
   again. Two instances, not a rate. Checking it would require running
   the server, which is outside this experiment's static frame.
3. Candidate A is closed. Shelved, not deleted.

Limits: four queries, 30 results each, public default branches only;
private consumers are invisible to this method.
