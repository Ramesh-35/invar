# STAGE3-EVAL-001 selection log

One recorded search, executed 2026-10-18T16:14:41Z (UTC), saved raw as
selection-search-raw.json: GitHub search/repositories,
q=language:TypeScript, sort=stars, order=desc, per_page=50, page=1.
Filters applied in rank order by scripts/stage3/select.py; the full walk
with one verdict per rank is selection-walk.jsonl. 19 repositories
qualified; every 2nd qualifying by rank was selected until N = 10
(S3-INT-1); SHAs pinned at selection time. This file, the manifest and
the driver are committed BEFORE any clone of any candidate (R-S-4).

Operationalizations (recorded, S3-INT-1 to S3-INT-4): every 2nd
qualifying means the 1st, 3rd, 5th and so on in walk order; the 100-file
floor counts .ts/.tsx excluding .d.ts, which excludes type-definition
corpora mechanically rather than by name; a tree listing truncated below
100 counted files is unverifiable and does not qualify; the OSI set is
listed in select.py.

S3-F-01 (protocol finding): the recorded search returned
incomplete_results=true, and the star-sorted index visibly includes some
recently star-inflated repositories alongside the established ones. The
population is therefore the committed snapshot, not a re-runnable query;
selection stays reproducible from the raw artifact. Star-inflated repos
are unseen real-world input and stay in, per R-S-5: nothing is swapped.

| rank | repository | stars | license | ts files | pinned SHA |
|---|---|---|---|---|---|
| 1 | freeCodeCamp/freeCodeCamp | 457410 | BSD-3-Clause | 989 | d8ad7a793290 |
| 4 | vuejs/vue | 212621 | MIT | 364 | 9e8870794008 |
| 9 | excalidraw/excalidraw | 134106 | MIT | 668 | 4c37956ba5f0 |
| 11 | earendil-works/pi | 116082 | MIT | 1590 | 482c1381a00d |
| 13 | thedotmack/claude-mem | 101562 | Apache-2.0 | 1544 | b85cd123daf8 |
| 15 | microsoft/playwright | 97997 | Apache-2.0 | 1363 | b335271cdc2b |
| 17 | puppeteer/puppeteer | 95715 | Apache-2.0 | 403 | ff3103c8e46f |
| 20 | ChatGPTNextWeb/NextChat | 88855 | MIT | 182 | 9b4ac8e1c543 |
| 22 | Egonex-AI/Understand-Anything | 86869 | MIT | 260 | f2d0164f72fe |
| 25 | diegosouzapw/OmniRoute | 78001 | MIT | 13541 | 3b52f62d9cc3 |
