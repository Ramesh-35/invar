# E2 packet: the exit-4 comprehension test (needs 5 real humans; ~2 hours)

This experiment cannot be simulated or run by INVAR's authors: it measures
whether strangers read the output correctly, and a synthetic "simulated
user" would be synthetic evidence graded by the thing being tested.

Protocol (per person, ~10 minutes, one at a time, no coaching):
1. Show exactly the terminal output below. Nothing else, no explanation.
2. Ask, verbatim: (a) "Did this pass?" (b) "What did it not check?"
3. Write both answers down before saying anything.

Output to show (produced by invar check, exit code 4 shown at the end):

    INVAR check (2 contract(s))
      [HOLDS]    payments (schema)
      [NOT EVALUATED] inventory (schema): 1 site(s)
          sku changed; mentioned at reader.go:41, not evaluated (pattern not read) [not-evaluated:pattern-not-read]
    result: NOT FULLY EVALUATED, 1 of 2 contract(s) skipped or not evaluated, the rest hold
    summary: held 1 | violated 0 | skipped or not evaluated 1
    measured: 38 of 70 held-out real cross-repository breaks caught (batches 4-8; docs/epistemic/STATUS.md)
    (process exited with code 4)

Scoring, fixed in advance:
- Correct (a): any answer meaning "partly / it didn't fail, but it didn't
  check everything". Plain "yes it passed" or "no it failed" is incorrect.
- Correct (b): mentions the inventory contract, the sku field, or "one site
  it could not read". "Nothing" or "don't know" is incorrect.
- CONFIRM the assumption if at least 4 of 5 answer both correctly.
- REFUTE it if 2 or more answer (a) with a flat pass or fail.
- Record every answer verbatim in docs/epistemic/STATUS.md either way.
