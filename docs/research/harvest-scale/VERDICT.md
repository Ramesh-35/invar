# Harvest-Scale (76 specimens): REJECTED, 2026-10-30

An externally produced research report claimed 76 admitted cross-repository
coping specimens across 12 ecosystems, each with "directly observed"
producer and consumer commit SHAs. Mechanically checked against GitHub
with docs/falsification/verify-specimens.sh (sample.tsv in this folder):

- Control (B1 case b1-001, real SHAs): VERIFIED.
- Sampled specimens SPEC-001, 021, 029, 038, 049, 050: FAILED on all
  three checks each; GitHub answers "No commit found for SHA". Six of
  six, plus SPEC-006's producer file (a .py path in a Go repository)
  and SPEC-001's consumer file, neither of which exists.
- Internal signs of fabrication: a "SHA" containing a non-hex letter
  (4b3l99120abc), a literal a1b2c3d4e5f6, near-identical generated hex
  across dozens of rows, and specimens admitted that the same report
  rejects (SPEC-012 vs REJ-002, SPEC-013 vs REJ-004).
- Real fragments exist (the SGLang multi_platform.py deprecation, the
  CAGE repository), but the SGLang shim is internal to SGLang, which the
  report itself rejected; the cross-repository evidence does not exist.

Consequence: zero specimens admitted. The B1 corpus stays at 10. No
number from the report (yields, restatement counts, the 80% coping
claim) may appear in any public word. The runtime-permit proposal that
cited SPEC-012, 029 and 038 is already rejected in BACKLOG.md.

Standing rule from today: no externally produced harvest enters any
ledger until verify-specimens.sh reports every row VERIFIED. The report's
CANDIDATES list (git log -S lookups) is kept only as unverified leads.
