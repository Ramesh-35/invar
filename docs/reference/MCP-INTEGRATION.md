# Wiring `invar mcp` into coding agents

`invar mcp` is a read-only MCP server on stdio (shipped v0.1.0-alpha.26).
It exposes five tools about the repository it is started in: `judge`
(v0.2.32), `check`, `contract`, `dependents`, `coverage`. It never writes files, performs no
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

## The five tools

- `judge` (v0.2.32): judge a change to a JSON Schema contract before it is
  committed, in one direction: every instance valid before must stay
  valid. Repository mode: `contract` (a path in this repository) as it
  stands in the working tree, against its version at `base_ref` (default
  `HEAD`; use `origin/main` for a whole branch). File mode: `base` and
  `head`, two paths in this repository. Returns the same JSON document as
  `invar judge ... --format json`: `verdict` HOLDS, VIOLATED (each broken
  promise with a dotted path, line numbers and, where one can be built, a
  counterexample document the old contract accepts and the new one
  rejects) or NOT EVALUATED (with the reason and every path not judged).
  Each witness says what it breaks: `compatibility_scope`
  `DECLARED_INPUT` or `UNDECLARED_PROPERTY_SPACE` (only instances that
  sent a key the base never declared), and `findings` counts the two
  apart. Caller usage is never observed (`observed_caller_usage:
  "UNKNOWN"`). Needs no invar.toml and no network. Paths outside the repository and
  option-looking refs are refused as errors, never verdicts; nothing a
  client sends can set the verdict.

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

## First workflow: judge a schema change before committing

The developer asks the agent to change a schema. Before committing, the
agent calls:

    judge {"contract": "schemas/media_player/jsonschema.json"}

On VIOLATED it shows each witness and its counterexample. A
`DECLARED_INPUT` witness means instances that use what the old contract
declared are rejected: do not commit the in-place edit; the usual fix is
a new versioned schema beside the old one. An `UNDECLARED_PROPERTY_SPACE`
witness means only senders of a key the old contract never declared are
rejected: say so, and check whether any caller forwards unknown keys
before deciding; the judge cannot tell. On NOT EVALUATED it says which paths were not judged and
asks a person to review them. On HOLDS it says the judged constructs
held; that is not a safety claim for anything listed as not judged.

To make the agent do this without being asked each time, add one line
to the project's agent instructions (CLAUDE.md, AGENTS.md or your
client's equivalent): "Before committing a change to any JSON Schema
file, call the invar `judge` tool on it and report the verdict."

## What an agent should do with the output

A VIOLATED contract is residue: deterministic, line-addressed evidence
that a declared field dependency no longer holds. It is not proof of a
runtime failure, and a holding check is not proof of safety: the
engine's measured miss rate is published
(docs/WHY-WE-PUBLISH-OUR-MISS-RATE.md). Cite the receipt, fix the read
or the producer, re-run `check`. A skipped or not-evaluated contract was
NOT checked; treat it as unknown, never as passing.
