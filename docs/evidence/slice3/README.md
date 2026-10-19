# INIT-DISCOVERY-SLICE3 pre-draft evidence

Measured 2026-10-19 at the STAGE3-EVAL-001 pinned SHAs, over the 6,394
exports-blocked imports recorded in docs/evidence/stage3/phase1/. Both
simulators share no code with the engine and read blobless clones.

- simulate_exports.py: Node exports resolution (condition order as
  written) and its variants; results in node-variants/.
- simulate_consensus.py: the drafted rule (order-free consensus over
  active conditions, tsconfig.json and tsconfig.build.json back-maps,
  declarationDir), plus the sanity check that every resolved file names
  the import or is an `export * from` barrel; results in consensus/.

Headline: Node alone reaches 0 of 6,394 TypeScript sources; the drafted
rule reaches 5,971, 0 ambiguous, 0 suspects.
