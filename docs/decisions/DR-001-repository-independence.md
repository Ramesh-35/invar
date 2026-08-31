# DR-001 — INVAR is an independent repository

- **Status:** Accepted
- **Date:** 2026-08-31

## Context

INVAR and NEXTLOGIC have overlapping intellectual history. There is a standing
temptation to treat INVAR as a NEXTLOGIC subsystem or to fork NEXTLOGIC's tree.

## Decision

INVAR is a standalone platform with its own repository at
`/Users/nikhilramesh/Documents/Projects/invar-platform`, its own history, build,
tests, and release lifecycle. It does **not** import NEXTLOGIC source, history,
or documents, and it never writes outside its own root. There is **no runtime
coupling** between the two today.

## Consequences

- NEXTLOGIC may eventually *consume* INVAR through an explicit, versioned
  interface. That interface does not exist yet and is a `[TARGET]`.
- The two working trees are verified independent before any commit (a NEXTLOGIC
  baseline diff must be identical pre- and post-INVAR work).
- INVAR must acquire its own canonical documentation rather than inheriting
  NEXTLOGIC's.
