# STAGE3-EVAL-001 findings ledger

Numbered monotonically in recording order (section F); a gap would mean a
trimmed ledger. Engine frozen at f9c4b29 throughout; no engine or fixture
edit was made during the run (B-S-4).

S3-F-01 (protocol) The one recorded search returned incomplete_results=true;
the population is the committed raw snapshot, not a re-runnable query.
Some star-inflated repositories sit in the ranking; none swapped (R-S-5).

S3-F-02 (evaluator) OmniRoute's first phase 1 attempt PIN-FAILED from the
evaluation container's disk exhaustion; retried at the same pinned SHA
after freeing space; both ledger rows kept.

S3-F-03 (instrument, fixed before the manifest existed) Runner r1 broke on
a relative outdir: init ran after cd into the clone, so its redirections
resolved against the wrong directory. Convicted by the founder's own dry
run (exit 1 rows that were the runner, not the engine). Fixed as r2,
committed (bc05870) before selection began. Recorded here rather than
first because F-01 and F-02 were already committed when this ledger was
opened; numbering follows recording order, never rewrites a committed id.

S3-F-04 (adjudication record) Proposal 5 (OmniRoute
strings:pricingFieldsSchema@const:NON_TOKEN_FIELDS) was judged NOT TRUE on
the stated ground that NON_TOKEN_FIELDS is disjoint from pricingFieldsSchema.
The source contradicts the stated ground: all 12 NON_TOKEN_FIELDS entries
are declared members of pricingFieldsSchema (src/shared/validation/schemas/
pricing.ts lines 42 to 53), and src/lib/pricingSync.ts copies exactly those
keys into each entry. The judgment stands as recorded (S-D3); re-ruling
requested. M-S-3 passes either way (8 of 9, or 9 of 9).

S3-F-05 (ENGINE DEFECT, backtest) Backtest replays every contract through
check::source_fields, the code-surface extractor of egress and schema
contracts. Strings contracts (valuepairs/strings.rs, which wrote the lock's
members line from the Zod key set) were never wired into backtest. Result:
61 of 61 absence events across all 8 locked strings contracts are false,
printed on producer bytes identical to HEAD, where check reads every bound
field as present. Backtest is NOT validated for strings contracts.

S3-F-06 (ENGINE DEFECT, backtest rendering) When a binding is never present
in the replay window, backtest prints "never absent after introduction"
(the (0, 0) arm, reached with first_present = None). That is a vacuous
truth that reads as reassurance; printed 61 times here for fields the
replay never once saw present. The honest line is "never present in the
replayed window".

S3-F-07 (protocol) The depth-200 pinned fetch makes the shallow boundary
commit look to git log like a change to every file, so backtest replayed
1 to 3 producer commits instead of up to 100, and the boundary dates
(2026-10-06, 2026-10-16) are the shallow cutoff, not real changes.
Backtest does not state that the history is shallow. Candidate engine
improvement under the stated-not-guessed doctrine: detect .git/shallow
and say so.

S3-F-08 (expected behavior confirmed; the yield gap) All 9 proposals are
strings contracts; zero code-pair proposals across the 8 multi-package
repositories. Of 7,685 UNRESOLVED abstentions, 6,394 (83 percent) are one
declared slice 1 limit, package.json "exports" maps, blocking discovery in
6 of 8 multi-package repositories; 619 more are tsconfig paths and 672
dist-only entries. Every one is coded, none guessed: honest, and the
single largest yield lever.

S3-F-09 (detector bound) The M-S-4 detector models named imports between
workspace packages only. Path aliases are a stated silent limit of slice 1
(INIT-DISCOVERY-SLICE1, membership section), so neither the engine nor the
detector counts them; M-S-4 certifies honesty inside init's declared
scope, not beyond it.
