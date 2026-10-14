# Experiment C exposure log (append-only)

Every way the frozen corpus has been examined before the official
evaluation. Not one of the five frozen evaluation files; it changes none
of them. Entries are appended, never removed or reworded.

## Entries

- 2026-10-13, all 30 repositories, ground-truth audit (directive 58317):
  producer-consumer field relationships were examined by hand in every
  repository, by the same engineer who implements the engine. This is
  the widest exposure; it is inherent in a single-rater design (see
  EVALUATION-PROTOCOL.md, measurement limitations).
- 2026-10-13, all 30 repositories, h2.py (rough H2 impact script):
  workspace markers only; no source files or field reads.
- 2026-10-14, biosshot/easyeda-copilot (POSITIVE), feasibility review
  (directive 73186): source of this repository only; no other corpus
  repository's source was inspected.
- 2026-10-14, biosshot/easyeda-copilot, fixture F01, slice 1 development
  (directive 86427): the full `invar init` output (6 proposals, 79
  abstentions), `invar lock` and `invar check` on the clone at fe7b2a5
  were run and read. No other corpus repository was run through the
  slice 1 engine.

## Sensitivity analysis, specified before any result

When the official evaluation runs, it is reported exactly under
EVALUATION-PROTOCOL.md. Alongside, labeled SENSITIVITY ANALYSIS, with no
threshold: primary coverage over POSITIVE repositories not listed above
as development cases (denominator 10 while only easyeda-copilot is
listed), and precision over proposals emitted in repositories not
listed as development cases. The audit and h2.py entries are not
development cases and do not change these denominators. This analysis
never replaces the official figures.
