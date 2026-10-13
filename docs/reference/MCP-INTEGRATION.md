# Wiring `invar mcp` into coding agents

`invar mcp` is a read-only MCP server on stdio (shipped v0.1.0-alpha.26).
It exposes four tools about the repository it is started in: `check`,
`contract`, `dependents`, `coverage`. It never writes files, performs no
network calls with `--offline`, and a failed tool call returns an MCP
error result labeled "not a verdict", never a verdict. Exit codes and
output shapes are covered by docs/COMPATIBILITY-COVENANT.md.

## Claude Code

One command, from the repository root (project scope writes a shareable
.mcp.json):

    claude mcp add --transport stdio --scope project invar -- invar mcp --offline

Or the .mcp.json entry directly:

    {
      "mcpServers": {
        "invar": {
          "type": "stdio",
          "command": "invar",
          "args": ["mcp", "--offline"]
        }
      }
    }

Drop `--offline` only if contracts reference remote `github:` producers
and a GITHUB_TOKEN is available in the environment (never in argv).

## Cursor and other MCP clients

Every MCP client that speaks stdio takes the same `mcpServers` entry as
above in its own config file; consult your client's MCP documentation
for where that file lives. We deliberately do not pin other clients'
config paths here: client configs drift, and documenting a moving
surface without witnesses is the class of mistake this tool exists to
catch.

## The four tools

- `check` (no args): evaluate invar.toml against invar.lock and its
  waivers. Per-contract status (holds, holds_waived, violated, skipped,
  not_evaluated) with Type.field residue, file:line read sites, reasons.
- `contract` (no args): what this repository depends on, from
  invar.lock: bound fields with types and acceptance dates, producer
  hash, waivers with days remaining.
- `dependents` (query, locks, optional producer): who depends on a
  producer field, from invar.lock files under a local directory. Same
  JSON document as `invar dependents --format json` (proven by test 47).
- `coverage` (optional contract): what INVAR evaluates here and what it
  states instead, with reason codes and the measured record. Never a
  verdict.

## What an agent should do with the output

A VIOLATED contract is residue: deterministic, line-addressed evidence
that a declared field dependency no longer holds. It is not proof of a
runtime failure, and a holding check is not proof of safety: the
engine's measured miss rate is published
(docs/WHY-WE-PUBLISH-OUR-MISS-RATE.md). Cite the receipt, fix the read
or the producer, re-run `check`. A skipped or not-evaluated contract was
NOT checked; treat it as unknown, never as passing.
