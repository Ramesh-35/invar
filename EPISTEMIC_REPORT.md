# EPISTEMIC REPORT (template)

This root file documents the **shape** of the epistemic report. The real report
is regenerated per run at `<output>/EPISTEMIC_REPORT.md` (e.g.
`runs/latest/EPISTEMIC_REPORT.md`) by:

```sh
cargo run -p invar-falsify -- run \
  --manifest corpus_manifest.json --composite composite.toml --output runs/latest
```

Every generated report preserves:

- **Run identity** — a `run_id` derived from the timestamp and manifest hash.
- **Timestamp** — microseconds since the Unix epoch.
- **Manifest identity** — path plus a BLAKE3 hash of the manifest bytes.
- **Engine version** — `invar-falsify 0.1.0-alpha`.
- **Per-case results** — verdict, rationale, and every composite leg's state.
- **Per-tool results** — the five leg states (`PASS`/`FAIL`/`SKIPPED`/`ERROR`/
  `UNKNOWN`), with a `[CAUGHT designated failure]` annotation where applicable.
- **Category status** — `HYPOTHESIS_UNTESTED` or
  `CATEGORY_RESIDUE_CANDIDATE (FIXTURE tier only)`. Never any "VALIDATED" /
  "PROVEN CATEGORY" wording on fixture evidence.
- **Epistemic integrity ledger** — explicit declarations that no CBR-001 claim
  was transferred, no benchmark manufactured, and no verdict fabricated.
- **Replicability hash** — a BLAKE3 hash over the run artifact.

> This template asserts no verdicts. See
> [`docs/epistemic/STATUS.md`](docs/epistemic/STATUS.md) for the vocabularies and
> [`docs/falsification/METHODOLOGY.md`](docs/falsification/METHODOLOGY.md) for the
> gate.
