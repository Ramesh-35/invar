# Public readiness audit (2026-10-11, repo at 02ac095 / v0.1.0-alpha.14)

Method: the full tracked tree (390 files) read from a stranger's seat, the
alpha.14 binary exercised from a clean directory, claim scans over every doc
and the CLI, secret and PII scans, licensing and standard-file checks.
Severity: BLOCKER (a stranger is misled on day 1), FIX (do before public),
DECISION (founder call needed), PAPERCUT (polish). Each item carries its
remediation. Nothing here has been changed yet except as noted.

## A. Front door: the repository does not introduce the product

1. BLOCKER. README.md is the Phase-1 falsification README. It introduces
   INVAR as "EICS, category claim not yet proven", its quick start runs the
   internal research harness (`invar-falsify run` over the fixture corpus),
   and it never mentions what the tool does today: cross-repository contract
   checking, `invar check`/`lock`/`init`, the GitHub Action, waivers, MCP,
   evidence receipts, the dependents graph, signed releases. A stranger
   cannot discover the product from the front page.
   Remediation: rewrite README product-first: one paragraph on what it
   checks; a 60-second quick start (download a signed release per
   RELEASE-SIGNING.md, `invar init`, `invar check`); the Action snippet
   (SHA-pinned, from ACTION-HARDENING.md); the exit-code table; an honest
   status section quoting the measured record (38 of 70 held-out breaks
   caught, R1 FALSIFIED) and linking STATUS.md. Keep the falsification
   harness as a clearly-labeled research section. The honesty is the brand;
   the current README just points it at the wrong product.

2. BLOCKER. The CLI's top-level about line is "Deterministic falsification
   harness for the INVAR category-residue hypothesis", and the first four
   commands listed (run, validate, report, verify) are the internal research
   harness, before `check` and `lock`. `invar --help` is many users' real
   front page. Remediation: product-first about text; research-harness
   commands kept but labeled as such in their help lines.

3. FIX. README's "What exists today" table is stale: version string
   `0.1.0-alpha`, "five FIXTURE-tier failure cases" (the corpus is 123 real
   cases today), deferred-crates framing unchanged since Phase 1. Covered by
   the README rewrite.

## B. Private context in a to-be-public tree

4. DECISION. NEXTLOGIC is named in README.md, INVAR.md, DR-001 and DR-002.
   Options: (a) genericize in README/INVAR.md ("a separate, unrelated
   internal platform") and leave the DR decision records as written history;
   (b) scrub everywhere, which rewrites decision records; (c) keep as is.
   Recommendation: (a). Decision records that get edited stop being records.

5. FIX. Personal absolute paths (/Users/nikhilramesh/...) appear in
   INVAR.md, INVAR-SOVEREIGNTY-REPORT.md and DR-001. They add nothing and
   name the machine layout. Remediation: neutral wording ("this repository")
   in INVAR.md and the report; DR-001 per the item 4 ruling on records.

6. PAPERCUT. `owner = "nikhil"` in lock examples (lock.rs doc comment,
   INVAR-LOCK.md, one test). Harmless; optionally switch examples to a
   neutral name. Git commit history will still carry the author identity,
   which is normal and public on any repo.

7. CHECKED, CLEAN: no tokens, keys, or credentials in the tree or in any
   committed file; landing bundles were never committed; workflows are
   SHA-pinned (scripts/check-pins.sh enforced in CI); CI runs with
   contents: read.

## C. Missing standard files

8. FIX. No SECURITY.md: a public repo with signed releases and a hardened
   Action needs a disclosure channel. Remediation: SECURITY.md naming a
   contact address, pointing at action/release-signers and
   RELEASE-SIGNING.md for key verification, and stating that only the
   latest alpha is supported.

9. FIX. No CONTRIBUTING.md, and the corpus's own admission rules (the
   actual contribution surface per MOAT Asset 1: "publish the corpus format
   and runner, so others can add cases") live only inside R1-CORPUS.md.
   Remediation: a short CONTRIBUTING.md: how to add a corpus case (pinned
   SHAs, public evidence, admission rules 1-2, controls), how to run the
   runner, and the honesty rules (no synthetic evidence upgraded to real).

10. CHECKED, CLEAN: dual licensing is complete and consistent (LICENSE,
    LICENSE-MIT, LICENSE-APACHE, Cargo.toml `MIT OR Apache-2.0`, README
    license section, release notes).

## D. Documentation coherence

11. DECISION. Six Phase-0/1 documents sit at the root (INVAR.md,
    EPISTEMIC_REPORT.md template, INVAR-BUILD-ORDER.md,
    INVAR-DEVELOPER-JOURNEY.md, INVAR-OPERATIONAL-DEFINITION.md,
    INVAR-SOVEREIGNTY-REPORT.md). All honest, all historical, all competing
    with README for a stranger's first click, and some carry stale targets
    (INVAR.md line 110: "[TARGET] Sub-millisecond-to-low-millisecond AST
    processing", a claim family we have since banned even as a target;
    EICS category framing). Recommendation: move all six to docs/history/
    with a two-line banner ("historical, superseded by README and
    docs/epistemic/STATUS.md"), and fix the README links.

12. FIX. docs/releases/ is missing v0.1.0-alpha.13.md and
    v0.1.0-alpha.14.md (alpha.9 was deliberately never tagged; worth one
    line in a releases README so the gap reads as a decision, not a hole).

13. CHECKED, OK: A4-DEPENDENTS.md's "guarantee" is the no-network-by-
    construction property, verifiable in the module; DR-006 already bans
    unverified performance figures; STATUS.md, MOAT.md, R1-CORPUS.md and
    the release notes are mutually consistent and are the repository's
    strongest asset for a skeptical reader.

## E. Onboarding, first 60 seconds (tested with the alpha.14 binary)

14. FIX. `invar init` in a directory with a JSON schema and a Python
    consumer (`t = r["total"]`) finds 0 consumers; the identical read in a
    .ts file finds 1 and proposes the contract. Either init's discovery
    should read .py consumers (the checker itself already does), or init's
    output should state which consumer languages discovery scans. An
    engine change either way: needs ratification, with a test.

15. FIX. Exit codes are documented only in action.yml and scattered docs.
    Remediation: the 0/1/2/3/4 table in README and in the long `--help`
    for check and lock (one clap after_help string each).

16. CHECKED, OK: `invar init` writes an honestly-commented proposal and
    says "heuristic; review before trusting"; `invar check` output states
    skips and not-evaluated sites; error messages carry "not a verdict".

## F. Record closures folded into this audit

- v0.1.0-alpha.13 release run 38049587630: SUCCESS, confirmed by the
  founder's terminal; recorded in STATUS alongside this audit.
- v0.1.0-alpha.14 tagged on 02ac095; CI and release run results to be
  recorded when confirmed.

## Recommended remediation order

1. README rewrite + CLI about text (items 1, 2, 3, 15) - one commit, no
   engine behavior change except help strings.
2. SECURITY.md + CONTRIBUTING.md + releases README + alpha.13/14 release
   docs (items 8, 9, 12).
3. Founder decisions on items 4 (NEXTLOGIC naming) and 11 (root docs to
   docs/history/), then that commit.
4. init Python-consumer discovery (item 14), as a normal ratified engine
   increment with a test.
After 1-3, nothing in the tree misleads a stranger; item 14 is a quality
papercut, not a blocker. Going public remains the founder's call; this
audit removes the embarrassment argument against it.
