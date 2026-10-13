# Vacancy pass G (AI supply chain), adjudicated 2026-10-13

Provenance: founder-run deep research (LLM-assisted) answering
DEEP-RESEARCH-PROMPT-G, delivered 2026-10-13 and adjudicated by the
engineer the same day. This file records the verdict, the findings that
bind us, and the corrections; the full report text is held by the
founder and may be appended verbatim.

## Verdict (accepted)

No candidate cleared all four bars (vacant, high pain, buildable with
INVAR's physics, defensible). Nothing is crowned. Killed with stated
reasons: MCP tool-definition drift (crowded: 8+ lockfile tools plus a
mainstream SDK primitive), agent-action receipts (pain undocumented,
spec layer crowded), eval reproducibility (format war already tipping,
reproduce half needs inference), prompt/config drift as a product,
model-card claims (needs execution and semantics), RAG index identity
(state lives in databases). Three survivors remain experiments, not
products: (A) MCP declared outputSchema vs observed structuredContent
across releases, offline on fixtures; (B) INVAR's existing check on
agent-authored PRs; (C) Hugging Face artifact-pair structural
compatibility (adapter target_modules vs base safetensors headers).

## Findings that bind us

1. Our README was stale. It said payload ingestion was JSON-only and
   never mentioned the MCP server, backtest, coverage or SARIF, so an
   outside researcher could not verify features that exist in tagged
   releases (TOML subset since alpha.23, MCP server since R2.2 and four
   tools since alpha.26, backtest since alpha.27). Fixed in the same
   commit as this record. The researcher was right to treat them as
   unverified.
2. External adoption is not visible: 1 star, 0 forks at the time of the
   pass. This is the crux in Rumelt's sense. Expanding scope before the
   first stranger succeeds on their own repository would be retreating
   to the strong link.
3. The sixty-second test fails for survivor B for the same reason it
   constrains INVAR today: inside a single repository init proposes
   zero contracts by design. Survivor C is the only candidate with a
   plausible offline stranger moment, and its hit rate is unmeasured.
4. No evidence was found that this audience rewards disclosed
   uncertainty. Our published miss rate as a conversion asset is an
   assumption, not a finding.
5. No regulatory pull before December 2027 (EU AI Act Omnibus).

## Pre-registered decision experiment (from the report, unchanged)

A: mine at least 30 public MCP servers that declare outputSchema with
at least 3 tagged releases; classify consecutive-release output changes;
find public consumers; offer 15 consumer locks. CONFIRM: at least 8
breaking output changes AND at least 3 with identifiable public
consumers AND at least 3 of 15 positive maintainer responses. REFUTE:
fewer than 4 breaks, OR zero consumers, OR at most 1 of 15 positive.
Thresholds are frozen as written here, before any run.
