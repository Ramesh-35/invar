# Security policy

## Reporting a vulnerability

Use GitHub's private vulnerability reporting on this repository (Security >
Report a vulnerability). Please do not open a public issue for a security
problem. You will get an acknowledgment, and the advisory stays private
until a fix ships.

## Verifying what you run

- Release binaries are signed with SSH signatures (namespace
  `invar-release`); the trusted public key is committed at
  `action/release-signers`. Verification steps:
  [docs/reference/RELEASE-SIGNING.md](docs/reference/RELEASE-SIGNING.md).
- The GitHub Action verifies the checksum and signature of any release
  binary before running it, and builds from exact pinned source otherwise.
  Threat model: [docs/reference/ACTION-HARDENING.md](docs/reference/ACTION-HARDENING.md).
- Evidence receipts and lock signatures use SSH signatures in their own
  namespaces (`invar-receipt`, `invar-lock`); a signature from one namespace
  never verifies in another.

## Supported versions

Pre-1.0: only the latest tagged alpha release receives fixes. The engine is
a single static binary with no runtime dependencies beyond optional `curl`
(remote fetches) and `ssh-keygen` (signature verification) subprocesses.

## Scope notes for researchers

The interesting attack surfaces, honestly stated: the producer/consumer
text parsers (all input is untrusted), the lockfile and receipt parsers,
the Action's download-and-verify path, and the report renderer (contract
ids and residue from fork PRs are attacker-controlled and are escaped; see
`md_code` in `check.rs`). The engine never executes checked code.
