# STAGE3-EVAL-002 M-S-4 triage (every flag, by hand, with its cause)

Detector: scripts/stage3/audit_abstain.py, the same sample rule as EVAL-001
(S3-INT-5 to S3-INT-8). It flagged 9 sampled constructs with no proposal
and no ABSTAIN line naming file:line and name. Both counts are kept: the
raw detector result and the hand triage below. No threshold was moved.

Cause established with an instrumented throwaway copy of the engine
(INVAR_DBG, never the work tree), printing find_shape's result and the
engine's own read set for the flagged import.

| repo | file:line | name | engine result | verdict |
|---|---|---|---|---|
| Understand-Anything | dashboard/src/utils/__tests__/filters.test.ts:3 | GraphNode | reads id, type: 0 distinctive | detector false positive (stoplist) |
| Understand-Anything | dashboard/src/utils/__tests__/filters.test.ts:3 | Layer | 1 distinctive | detector false positive (stoplist) |
| Understand-Anything | dashboard/src/components/WarningBanner.tsx:2 | GraphIssue | 1 distinctive | detector false positive (stoplist) |
| Understand-Anything | dashboard/src/components/FileExplorer.tsx:2 | GraphNode | 1 distinctive | detector false positive (stoplist) |
| pi | durable/test/harness-host-inputs.test.ts:1 | Models | Decl found; engine reads only `model` | detector false positive (engine read set) |
| excalidraw | excalidraw/components/App.duplicate.ts:29 | ListMarkerAdvance | NotShape; 3 distinctive reads | SILENT DROP (S3-F-11, S3-F-12) |
| puppeteer | puppeteer/src/node/install.ts:14 | ChromeSettings | NotShape; 2 distinctive reads | SILENT DROP (S3-F-10, S3-F-12) |
| puppeteer | puppeteer/src/node/install.ts:14 | ChromeHeadlessShellSettings | NotShape | SILENT DROP (S3-F-10, S3-F-12) |
| puppeteer | puppeteer/src/node/install.ts:14 | FirefoxSettings | NotShape | SILENT DROP (S3-F-10, S3-F-12) |

Verdict: M-S-4 FAILS on STAGE3-EVAL-002. 4 genuine silent drops among 26
sampled constructs. The hard threshold is zero.

Findings (continuing the STAGE3-EVAL-001 ledger numbering):

S3-F-10 (ENGINE, slice 1 scanner) `export type * from` (TypeScript 5.0) is
not parsed as a star re-export; only `export *` and `export type {` are.
A barrel chain through it is invisible (puppeteer-core common/common.ts).

S3-F-11 (ENGINE, slice 1 scanner) Regex literals are not masked. An
unbalanced bracket inside one (`[.)]` in excalidraw listMarker.ts:22)
shifts bracket depth for the rest of the file, and every later
declaration becomes invisible (ListMarkerAdvance at line 193).

S3-F-12 (ENGINE, the honesty class) codepairs `Found::NotShape => {}` is
silent, and NotShape conflates two cases: the imported name is not a
shape (legitimately silent: a function, a const), and a declaration the
scanner could not see or reach (a chain deeper than the BARREL2 check, an
unparsed re-export form, a scanner miss). The second must abstain. This is
slice 1 code; before SLICE3 these packages abstained wholesale as
exports-blocked, so the path was unreachable at scale. SLICE3 is correct
by its own gates, and it is what made this path reachable.

S3-F-13 (detector bound) The detector omitted init's frozen stoplist and
reads `.member` anywhere in the file, so it over-flags; 5 of the 9 flags
were false positives on triage. Recorded as S3-INT-9 (stoplist-aligned
count reported beside the raw count, never replacing it).
