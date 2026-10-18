//! INVAR-EXPLAIN-001: the code registry and `invar explain <CODE>`
//! (rulings D-E1 to D-E6, 2026-10-18).
//!
//! explain is documentation, never evaluation (B1): it reads no repository,
//! no manifest, no lock, no clock and no environment; output depends only on
//! this binary and the argument. The registry describes THIS binary's rules
//! (B2). Class 1 files cannot reference the registry, so it is by
//! construction a second copy of each code string (B3); the no-drift
//! guarantee is the bidirectional source-scan test in tests/explain_test.rs,
//! which compares the binary's own index against the crate's emission sites.
//! Deliberately, the entries below are TUPLES: the scan patterns (`code: "`
//! and `abstain("`) can never match this file, so the registry cannot
//! satisfy the drift test by matching itself.
//!
//! Exit codes of explain itself: 0 (explained) or 3 (UNKNOWN-EXPLAIN-CODE).
//! Never 1: exit 1 is the VIOLATED verdict, and explain issues no verdicts
//! (ruling D-E5 as flagged and ratified).

/// (code, kind, scope, condition, action, defined_in), sorted by code.
/// kind is "abstention", "error" or "exit".
const REGISTRY: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("0", "exit", "every command",
     "every evaluated contract holds; a skip or abstention did not occur",
     "none",
     "the frozen exit-code contract (`invar --help`)"),
    ("1", "exit", "invar check",
     "at least one locked contract is VIOLATED: a stale reference, or a relied-on field missing from a proof",
     "fix the consumer, or review the producer change and re-lock deliberately",
     "the frozen exit-code contract (`invar --help`)"),
    ("2", "exit", "ledger commands",
     "the evidence ledger's bi-temporal chain is broken; this code keeps that one meaning and is never returned by proof verification",
     "treat the ledger as untrusted and investigate before any further run",
     "the frozen exit-code contract (`invar --help`)"),
    ("3", "exit", "every command",
     "an error or refusal that is never a verdict: unreadable config, lock drift, a failed authoring precondition, a tampered or forked proof",
     "read the printed reason; nothing was evaluated to a verdict",
     "the frozen exit-code contract (`invar --help`)"),
    ("4", "exit", "invar check",
     "skipped or not evaluated: an unsupported language, an unlocked contract, or an abstention; a skip is never a pass",
     "read the ABSTAIN line and resolve what it names, then evaluate again",
     "the frozen exit-code contract (`invar --help`)"),
    ("AMBIG-EGRESS", "abstention", "invar init, slice 2 (K1)",
     "an eligible boundary file imports a shape but never references it in a response position, or the reference is ambiguous; no emitted-shape contract is proposed",
     "return the parsed shape at the boundary if the contract is wanted; otherwise nothing is claimed",
     "INIT-DISCOVERY-SLICE2.md"),
    ("AMBIG-PACKAGE", "abstention", "invar init, slice 2",
     "an import specifier resolves to more than one workspace package",
     "disambiguate the package names so the import resolves to one",
     "INIT-DISCOVERY-SLICE2.md"),
    ("AMBIG-RECEIVER", "abstention", "invar init, slice 2b (K3)",
     "a string list qualifies against more than one shape, overlaps a shape that no import, directory or package links it to, or sits outside a query position; the receiver cannot be told",
     "none required; split the shapes or move the list next to its store if the contract is wanted",
     "INIT-DISCOVERY-SLICE2B.md"),
    ("AMBIG-SHAPE", "abstention", "invar init, slice 2",
     "an imported name resolves to more than one declaration",
     "rename one declaration so the name resolves to one",
     "INIT-DISCOVERY-SLICE2.md"),
    ("BARREL2", "abstention", "invar init, slice 2",
     "a symbol is re-exported through more than one barrel level; resolution follows exactly one level",
     "import the symbol from its declaring module if the contract is wanted",
     "INIT-DISCOVERY-SLICE2.md"),
    ("H2", "abstention", "invar init, slice 1 (repository level)",
     "the workspace has fewer than two qualifying packages, or its workspace marker is unparsable, so cross-package discovery cannot run",
     "nothing to fix for a single-package repository; for a workspace, repair the marker file",
     "INIT-DISCOVERY-SLICE1.md"),
    ("H2-ENTRY", "abstention", "invar init, slice 1",
     "a workspaces entry cannot be expanded; that entry is skipped",
     "fix the workspaces entry (path or glob)",
     "INIT-DISCOVERY-SLICE1.md"),
    ("H2-MEMBER", "abstention", "invar init, slice 1",
     "a declared workspace member is missing, unreadable or inside a skipped directory, so its files are not scanned",
     "restore the member or remove it from the workspaces list",
     "INIT-DISCOVERY-SLICE1.md"),
    ("NON-DISTINCTIVE-KEY", "abstention", "invar init, slice 2b (K3)",
     "a query-position list overlaps a shape on fewer than 3 distinctive names after the stoplist, or on under half of the list's distinctive entries",
     "none; the threshold is frozen (ruling D1) and a near miss is stated, never proposed",
     "INIT-DISCOVERY-SLICE2B.md"),
    ("PROOF-AMBIG-SCHEMA", "abstention", "invar check --proof",
     "more than one proof file in the bound directory attests the same contract_id",
     "point the binding at one file, or remove the duplicate proof",
     "INVAR-PROOF-001.md"),
    ("PROOF-FORK", "error", "invar check --proof",
     "two proofs claim the same epoch with different digests, or an epoch one above the accepted one does not chain to the accepted digest",
     "treat the provider lineage as disputed; review both proofs before accepting either",
     "INVAR-PROOF-001.md"),
    ("PROOF-NOT-FOUND", "abstention", "invar check --proof",
     "the bound proof path does not exist, or the bound directory holds no proof for the contract_id",
     "fix the proof path in the binding, or obtain the provider's proof",
     "INVAR-PROOF-001.md"),
    ("PROOF-SCOPE-MISMATCH", "abstention", "invar check --proof",
     "the binding's contract_id or anchor differs from what the proof attests",
     "align the binding with the proof deliberately; never edit the proof",
     "INVAR-PROOF-001.md"),
    ("PROOF-TAMPERED", "error", "invar check --proof",
     "the digest does not match the canonical bytes, the proof or a binding is malformed, the format or level is unknown, or a [[proof.import]] key is unknown or missing",
     "re-obtain the proof from the provider and fix the binding; a proof is never hand-edited",
     "INVAR-PROOF-001.md"),
    ("PROOF-UNSUPPORTED-KIND", "abstention", "invar check --proof",
     "the binding or the proof names a contract kind other than egress; v1 verifies egress only (ruling D5)",
     "none; non-egress proofs are a later version, never a silent pass",
     "INVAR-PROOF-001.md"),
    ("R6", "abstention", "invar init, slice 2 inventory",
     "a file reads two or more fields of an exported shape without importing it by name; out of v1, stated, never proposed",
     "import the shape by name if the pair should become a contract",
     "INIT-DISCOVERY-SLICE2.md"),
    ("SLICE-SCOPE", "abstention", "invar init, slice 1 (repository level)",
     "the workspace is declared by a marker outside the frozen slice (only package.json workspaces and pnpm-workspace.yaml are read)",
     "none; other workspace markers are out of the frozen scope by design",
     "INIT-DISCOVERY-SLICE1.md"),
    ("STALE-PROOF-COMMIT", "abstention", "invar check --proof",
     "the proof's epoch is below the consumer's accepted epoch: an older proof was replayed or rolled back",
     "obtain the provider's current proof; never lower accepted_epoch to make it pass",
     "INVAR-PROOF-001.md"),
    ("TEMPORAL-STALENESS-UNANCHORED", "abstention", "invar check --proof",
     "no accepted_epoch and accepted_digest are recorded in the binding, or the opt-in max_age_days window is exceeded; freshness cannot be established and HOLDS is not offered",
     "review the proof once, then record accepted_epoch and accepted_digest in the [[proof.import]] table",
     "INVAR-PROOF-001.md"),
    ("TSC-GOVERNED", "abstention", "invar init, slice 2 (K2) and slice 2b REV1 (K3)",
     "the pair is already governed by the TypeScript compiler: the shape is consumed through types (z.infer, typeof or typed reads), or the string list carries a recognized keyof constraint (satisfies or annotation); a lock would add nothing",
     "none; the compiler is the stronger guarantee here, and INVAR yields to it",
     "INIT-DISCOVERY-SLICE2.md and INIT-DISCOVERY-SLICE2B-REV1.md"),
    ("UNKNOWN-EXPLAIN-CODE", "error", "invar explain",
     "the argument is not a code this engine emits and not an exit numeral",
     "run `invar explain` with no argument for the full index",
     "INVAR-EXPLAIN-001.md"),
    ("UNREADABLE", "abstention", "invar init, all slices",
     "a file in scope could not be read or is not UTF-8; it is skipped, never guessed at",
     "fix the file's encoding or permissions if it should be scanned",
     "INIT-DISCOVERY-SLICE1.md"),
    ("UNREPRESENTABLE", "abstention", "invar init, all slices",
     "a proposal's path or name cannot be written in flat TOML (whitespace or control characters); not proposed",
     "rename the offending path or symbol if the contract is wanted",
     "INIT-DISCOVERY-SLICE1.md"),
    ("UNRESOLVED", "abstention", "invar init, slice 2",
     "an import in an eligible boundary file cannot be resolved to a file",
     "check the import specifier and the package name",
     "INIT-DISCOVERY-SLICE2.md"),
    ("UNSUPPORTED-DYNAMIC", "abstention", "invar init, slice 2",
     "the shape cannot be stated statically (spread, computed members or a method chain on the literal)",
     "declare the shape as a plain object literal if the contract is wanted",
     "INIT-DISCOVERY-SLICE2.md"),
    ("UNSUPPORTED-ORM-SOURCE", "abstention", "invar init, slice 2b (K3)",
     "the only candidate shape source is a .prisma or .sql declaration, which slice 2b does not parse",
     "none; ORM sources are deferred by ruling D4 of slice 2b, never half-parsed",
     "INIT-DISCOVERY-SLICE2B.md"),
    ("UNSUPPORTED-STRING-DYNAMIC", "abstention", "invar init, slice 2b (K3)",
     "the string list has a non-literal element (spread, interpolation, concatenation, a computed entry or a method chain) and is unrepresentable; a partial list is never paired",
     "declare the list as plain string literals if the contract is wanted",
     "INIT-DISCOVERY-SLICE2B.md"),
    ("VERSION-FAMILY", "abstention", "invar init, slices 2 and 2b",
     "the shape belongs to a version family (a v<N> path segment or a ...V<N> name); pairing across versions is refused",
     "none; version families are never paired by design",
     "INIT-DISCOVERY-SLICE2.md"),
];

/// `invar explain [CODE]`: returns (stdout text, exit code). Exit 0 on an
/// explained code or the index; exit 3 on an unknown code. Reads nothing.
pub fn explain(arg: Option<&str>) -> (String, i32) {
    debug_assert!(REGISTRY.windows(2).all(|w| w[0].0 < w[1].0), "registry must be sorted and unique");
    match arg {
        None => {
            let mut out = String::new();
            for (code, kind, _, condition, _, _) in REGISTRY {
                out.push_str(&format!("{code} ({kind}): {condition}\n"));
            }
            (out, 0)
        }
        Some(code) => match REGISTRY.iter().find(|e| e.0 == code) {
            Some((code, kind, scope, condition, action, doc)) => (
                format!("{code} ({kind})\nscope: {scope}\ncondition: {condition}\naction: {action}\ndefined in: {doc}\n"),
                0,
            ),
            None => {
                let known: Vec<&str> = REGISTRY.iter().map(|e| e.0).collect();
                (
                    format!("UNKNOWN-EXPLAIN-CODE: {code} is not a code this engine emits\nknown codes: {}\n", known.join(" ")),
                    3,
                )
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_sorted_and_unique() {
        for w in REGISTRY.windows(2) {
            assert!(w[0].0 < w[1].0, "{} must sort before {}", w[0].0, w[1].0);
        }
    }

    #[test]
    fn registry_covers_the_five_exit_numerals_exactly() {
        let exits: Vec<&str> = REGISTRY.iter().filter(|e| e.1 == "exit").map(|e| e.0).collect();
        assert_eq!(exits, ["0", "1", "2", "3", "4"]);
    }

    #[test]
    fn unknown_code_is_exit_3_and_lookup_is_exit_0() {
        assert_eq!(explain(Some("TSC-GOVERNED")).1, 0);
        assert_eq!(explain(None).1, 0);
        let (out, code) = explain(Some("NOPE"));
        assert_eq!(code, 3);
        assert!(out.contains("UNKNOWN-EXPLAIN-CODE"));
    }
}
