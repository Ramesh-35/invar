# Not-evaluated reason codes

Every statement that INVAR did not evaluate something carries a stable code
(research adjudication 2026-10-11, mypy-style). Codes are append-only: a
code's meaning never changes, and new codes get new names. They never carry
a severity, so no severity filter can hide them; they count in the fixed
summary line and in exit 4.

| Code | Meaning |
|---|---|
| [not-evaluated:unsupported-language] | An artifact is in a language INVAR does not read (C, C++, C#, PHP, Swift, Java, Kotlin, Ruby). |
| [not-evaluated:pattern-not-read] | A changed field the consumer mentions in a form INVAR's read patterns do not parse (A1 rule 1). |
| [not-evaluated:field-moved] | A field moved between producer types; the consumer's use is stated, not evaluated (A1 rule 2). |
| [not-evaluated:default-switch] | A default version or format selector changed at the producer; which output path the consumer gets cannot be told from text (F1). |
| [not-evaluated:stale-sample] | The contract's observed sample is older than 90 days; refresh with `invar lock --sample`. |
| [not-evaluated:optional-field-absent] | A locked sample path is absent from the fresh payload, but some observation also lacked it: the field is known optional; stated, never a violation (MULTI-SAMPLE-WITNESSES.md). |
| [not-evaluated:witness-count-low] | A locked sample path is absent or retyped but was observed fewer than 3 times: too few witnesses to accuse; stated. |

Exit 4 notes (research A3/A4, adopted as documentation): exit 4 is non-zero,
so a plain `invar check` step fails a CI job by default; the GitHub Action's
`fail_on_violation` input is the knob for report-only mode. Not-evaluated
items deliberately have no severity level: severity filters are where true
findings go to die.
