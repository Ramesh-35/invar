# `invar demo` (spec, frozen 2026-10-11)

Ratified direction; implementation follows observed-reality locking or in
parallel. One honesty correction to the ratifying dispatch, recorded here
so the record is straight: the demo replays cases from OUR corpus of real
cross-repository breaks. It does not replay the CrowdStrike channel-file or
Azure Front Door incidents: those producers (kernel C++, binary channel
files, closed config planes) are outside INVAR's supported scope, and
demoing them would claim coverage we do not have. The horizon report's
counterfactual analyses of those incidents stay what they are: analyses.

## What it does

`invar demo` (no arguments, no flags needed):

1. Ships 2 or 3 real corpus cases' pinned artifacts inside the repository
   (and inside release tarballs) under `demo/`: producer base, producer
   head, consumer, exactly the bytes the corpus pins, with their BLAKE3
   checks. Candidates, chosen for small artifacts and clear stories, all
   CAUGHT on held-out scoring: r1-109 (authentik removed the coordinate
   fields a dashboard summed), r1-112 (headscale renamed machines to
   nodes under a UI), r1-117 (ghostfolio removed the performance fields a
   widget read). RATIFY the final pick.
2. Runs the normal engine on them, fully offline, and prints the normal
   `[VIOLATED]` output with the real repository names, the field, the
   consumer file and line, and one line of what happened in the real world
   (from case.toml), plus measured wall time.
3. Ends with the honest coda, verbatim: "This is a real break INVAR
   catches. On held-out real breaks overall it currently catches 38 of 70;
   what it cannot evaluate it states instead of passing. invar init to try
   it on your own repositories."

## Constraints

- Zero network, zero setup, works in a fresh clone and from a release
  tarball. Wall time is printed as measured, never promised in advance.
- The demo uses the stock engine path (`evaluate_with`), not a special
  mode: what it shows is what `invar check` does.
- Artifact licenses: the embedded files are small excerpts of open-source
  code pinned by the corpus, shipped with their repository names and
  licenses noted in demo/NOTICE. RATIFY.

## Measure

TTFV (MOAT R6) is measured WITH the demo as the entry point: time from
binary download to the demo verdict, and separately to a first real
verdict on the user's own pair. Published in MOAT.md as measured.
