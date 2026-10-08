# R1: Real-Tier Falsification Corpus

Critical path of the locked roadmap (docs/architecture/ROADMAP.md). Its job
is to find out, on real breaks, whether INVAR catches anything the existing
free tools miss. A result where INVAR adds nothing is a valid, publishable
outcome.

## Status

| Item | State |
|---|---|
| Case format and runner (`corpus-real/`, `scripts/r1-run.sh`) | built; all six outcomes exercised on test cases |
| Real cases admitted | **0 of 30** |
| Baselines installed and run | none yet |

`r1-000` is REAL-INCIDENT-001, a constructed demo kept as a harness
self-test. It is marked `real = "false"` and never counts.

## Admission rules (a case enters only if all hold)

1. **Provenance first.** A public source (issue, PR, postmortem or commit)
   shows that a consumer actually broke because a producer changed. Link it in
   `source`; quote the decisive line in `expected.evidence`.
2. **Pinned.** Producer and consumer are pinned to full commit SHAs in public
   repositories, so anyone can replay the case.
3. **Expected residue comes from the source, never from INVAR.** Fill
   `expected.residue` before running INVAR. Do not edit it afterwards to match.
4. **Cases INVAR will likely miss are admitted on equal terms.** Do not select
   cases because they suit INVAR's patterns.
5. **Controls.** Include real producer changes that did *not* break the
   consumer (`residue = "none"`), at least one control per five breaks, to
   measure false positives.
6. **No manufacturing.** No edits to either side. If a minimal reproduction is
   needed, it lives in a separate case marked `real = "false"`.

## Outcomes (scripts/r1-run.sh)

| Outcome | Meaning |
|---|---|
| CAUGHT | every expected field is in INVAR's residue |
| PARTIAL | some found, some missed |
| MISSED | none found, or INVAR said the contract holds on a real break |
| CLEAN | control case, INVAR reported nothing |
| FALSE_POSITIVE | control case, INVAR reported residue |
| ERROR | INVAR could not evaluate (exit 3); never counted as a verdict |

Fields INVAR reports beyond the expected set are listed in `extra`, not
hidden.

## Baselines

Each case records, per tool, `caught`, `missed`, `not_applicable` or
`not_run`, filled by running the tool on the same pinned commits: oasdiff or
openapi-diff (OpenAPI), `buf breaking` at the matching category (Protobuf),
schema-registry compatibility checks (Avro, JSON Schema), Semgrep, and one AI
reviewer as a non-deterministic reference. The runner copies these values; it
never infers them.

## Falsification thresholds (locked)

With 30 or more real cases: the category claim fails if the baselines together
catch 90% or more of what INVAR catches, or if INVAR misses more than 20% of
true breaks.

## Known limit, stated before any case runs

Schema and event contracts find consumer dependencies only as
`payload.<field>` reads. Real consumers name things differently
(`order.amount_cents`, `data["amount_cents"]`, typed models), so INVAR is
expected to MISS many real cases as it stands. That is what R1 is for; misses
will be recorded and will decide whether the consumer side needs a better
extractor before anything else is built.
