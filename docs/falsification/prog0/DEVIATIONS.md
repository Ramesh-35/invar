# PROG-0 deviations, each written before the run it affects

1. pixi (51b3fa1a0): excluded. No route to pixi 0.81.0 from this workspace (GitHub closed;
   the crates.io "pixi" is an unrelated 0.15 package). Listed, not counted.
2. Traefik v3.7.6 built from the Go proxy source; the nested module
   github.com/traefik/traefik/dynamic/ext (absent from the proxy zip) stubbed as two empty
   structs. It extends dynamic HTTP config only; static config, where the changed key lives,
   is the real v3.7.6 code.
3. claude-code-settings 2f74efc3f and 5fcc2e0ed: excluded, no counterexample exists. Their
   only witnesses are `pattern` changes in which the new pattern accepts strictly more
   (names added to an alternation; a lookahead removed). INVAR's VIOLATED here is not a
   decided break. Recorded as a finding about INVAR (see RESULTS.md).
4. claude-code-keybindings 2f74efc3f: excluded, no observable. `claude doctor` 2.1.220
   reports nothing even for {"bindings": 5}; no other offline loading path found.
5. Every program runs in a network namespace with no network and `env -i` (PATH, HOME only).
   Before this was in place two probe runs of Claude Code (`config list`, `--print hi`) used
   this session's environment and reached the model API; neither is a measurement.
6. pnpm ships parallel release lines. The frozen rule ("latest release on or before the date")
   is applied as written and is the counted result; the newest-major release on that date is
   also run and reported, not counted.
7. Hand-built counterexamples (pyproject 6b9673c4c, package a83ce6f19) use the shortest value
   the new schema rejects at the witness path, "" in both, inside the smallest document the
   old schema accepts; chosen before any run, and used only after jsonschema 4.26 and Ajv 8.20
   both confirm old-accepts and new-rejects.
8. package a83ce6f19: counted command is `npm install --offline` (fixed before running); `npm pack --dry-run` is run and reported, not counted.
9. helmfile e7373bd3f: helmfile calls a separate `helm` binary for its version before building; a stub `helm` that only prints a v3 version string is put on PATH (fixed before running). helmfile's own config loading is unchanged.
