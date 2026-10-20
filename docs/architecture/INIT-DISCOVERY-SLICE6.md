# INIT-DISCOVERY-SLICE6: tsconfig paths and earned silence in egress discovery

Status: DRAFT r1 for ruling. Implementation NOT authorized. Parent: the
founder's direction of 2026-10-20 (an egress honesty slice, pre-registered
at zero yield). Baseline: d4488ac.

## 0. Evidence, measured on d4488ac before this draft (throwaway prototype)

The prototype (evidence/slice6/paths-prototype.patch, binary d40fd244faa0)
reads the root tsconfig.json `compilerOptions.paths` and `baseUrl` in
egress discovery (valuepairs, K1) and resolves an alias specifier as
TypeScript does: an exact key, else the longest single-`*` pattern, each
target tried in order, the first existing file wins.

| Repository | Effect |
|---|---|
| OmniRoute | 619 `UNRESOLVED ... path alias (tsconfig paths are not read)` resolved; abstentions 682 to 142; 79 new, specific abstentions (54 TSC-GOVERNED, 22 BARREL2, 3 AMBIG-EGRESS); proposals 6 to 6, byte-identical; 9,145 alias resolutions and 1 prototype miss (below) |
| the other 9 | init output byte-identical (excalidraw has 14 aliases; egress discovery never reaches them there) |

The same run on the 10-19 prototype engine gave the same OmniRoute output,
byte for byte apart from the SLICE5 header sentence.

The 1 miss, traced while drafting: `@omniroute/open-sse/vendor/
codex-chatgpt-web/browser-login.ts`. The file exists, but it lies under a
`vendor` directory, which init never scans (init.rs SKIP_DIRS), so the
probe rightly does not accept it. The importing file is not one egress
discovery scans, so nothing was printed; had it been, the prototype's
reason ("no target file exists") would have been false. B3 names the true
cause.

The catch: 1,393 OmniRoute imports newly reach K1's silent branch (an
import resolved to a file where K1 finds no Zod object schema of that
name: `Found::NotShape => {}`, valuepairs.rs). Classified on masked text
(evidence/slice6/classify_k1.py, k1-silences.tsv):

| Class | Count | Silence |
|---|---|---|
| declared in the target file as a non-Zod value or as a type | 1,225 | earned |
| declared through the target's re-exports (one alias specifier among them: `PROVIDER_ID_TO_ALIAS`) | 166 | earned, only if the check follows re-exports, alias specifiers included |
| declared as a Zod schema that is not an object (`z.unknown().transform(...)`: proxySubscriptionCreateSchema, schema.ts:139; proxySubscriptionUpdateSchema, :207) | 2 | NOT earned |

Correction of the 2026-10-19 record, stated: it counted 4 unearned (2 Zod
schemas "unreached", 2 names "declared nowhere"). Both were classifier
errors. The two schemas are reached (one barrel level); they are not
object shapes. The two "declared nowhere" names are a value re-exported
through an alias specifier, which the 10-19 classifier did not follow.

Fixture impact, measured: with path aliases on, 328 of 328 tests pass
unchanged; no test workspace reaches K1's silent branch (a hook logging
every such import during the suite logged none), so earned silence
changes no existing fixture.

Bounds, stated before any code:

B1 Zero yield. No proposal is added, removed or changed by a byte.
B2 Root config only: `compilerOptions.paths` and `baseUrl` of the root
tsconfig.json, read with the existing JSONC strip. `extends` and nested
tsconfig files are not followed (S6-D1).
B3 Resolution as TypeScript does it: exact key, else longest single-`*`
match; targets in order; first existing file (the existing probe). A
target that exists only inside a directory init does not scan abstains
saying so, never "no target file exists". An alias that matches but
whose targets do not exist abstains UNRESOLVED with the key named.
B4 Earned silence in K1, the SLICE3B rule carried over: an import that
resolves to no Zod object schema is silent only if the name is declared,
in the target file or along its re-exports (alias specifiers included), as
a non-Zod value or as a type, and no file on that path is unread. Otherwise
UNRESOLVED with the exact reason: a Zod schema that is not an object (its
site), declared nowhere, or an unread file.
B5 K1's barrel depth stays one level (S6-D3); egress code-pair depth is
not this slice.
B6 Zero new dependencies, codes or exit changes.

## B. Numbered requirements

R-S6-1 Aliases resolve per B2 and B3. The generic "tsconfig paths are not
read" reason is emitted only when no root tsconfig defines `paths`; when it
does and no key matches, the reason says that instead (`matches no key of
the root tsconfig paths; not guessed`). On OmniRoute every alias matched a
key (0 such lines).
R-S6-2 Every K1 silent branch emits nothing only under B4; otherwise
`UNRESOLVED {file}:{line} {name} from "{spec}": {reason}; not guessed`.
R-S6-3 Determinism: two runs byte-identical.

## C. Countermodels

CM-S6-1 an alias guessed when no target exists. CM-S6-2 a shorter pattern
chosen over a longer match. CM-S6-3 targets tried out of order. CM-S6-4 a
Zod non-object schema left silent. CM-S6-5 a name declared nowhere left
silent. CM-S6-6 a re-exported value made loud (silence not followed through
the chain). CM-S6-7 a proposal changed. CM-S6-8 `extends` silently
followed or silently ignored when it carries the only `paths`.

## D. Fixtures (declared outcomes, before any code)

S6-F1 `@/*` to `./src/*`: the import resolves; the shape proposes exactly
as a relative import would. S6-F2 two patterns, longest wins. S6-F3 two
targets, the first missing: the second resolves. S6-F4 a matching alias
with no target file: UNRESOLVED naming the key. S6-F4b an alias whose
target exists only under `vendor/`: UNRESOLVED naming the skipped
directory. S6-F5 a value imported
through an alias: silent. S6-F6 a value reached through the target's
re-export via an alias specifier: silent. S6-F7 `z.unknown().transform`
imported: UNRESOLVED naming its site. S6-F8 a name declared nowhere:
UNRESOLVED. S6-F9 a root tsconfig with only `extends`: the existing
generic reason, unchanged (S6-D1). S6-F10 root `paths` defined, an alias-looking specifier matching no key:
UNRESOLVED with the no-key reason. S6-N1 a workspace with no tsconfig:
byte-identical golden. S6-M1 two runs byte-identical.

## E. Mutation set

1 longest-match replaced by first-match (S6-F2). 2 targets not tried in
order (S6-F3). 3 missing target guessed (S6-F4). 3b the skipped-directory reason replaced by the no-file reason (S6-F4b). 4 the K1 silent branch
restored (S6-F7). 5 re-exports not followed by the silence check (S6-F6).
6 a declared-nowhere name earns silence (S6-F8). 7 the Zod-site omitted
from the reason (S6-F7).

## F. Gates, then STAGE3-EVAL-006

Gates 0 to 5 as usual. STAGE3-EVAL-006 pre-registration (proposed for
ruling): proposals 113, byte-identical to EVAL-005; OmniRoute abstentions
682 to 144 (the 142 of the prototype plus the 2 Zod non-object
abstentions); the other nine repositories byte-identical; 0 silent drops,
audited over every newly reachable silence (all 1,393, not a sample);
M-S-5 and M-S-6 unchanged (no proposal moves). No adjudication: nothing is
proposed that was not proposed before.

## H. Decision points for ruling

S6-D1 Root tsconfig only, `extends` not followed (recommended: every alias
measured on the manifest is defined in a root config; following `extends`
means resolving package specifiers inside config files, its own slice).
S6-D2 The earned-silence rule of B4, including re-exports through alias
specifiers (recommended: 166 of the 1,391 earned silences depend on it).
S6-D3 K1 barrel depth stays one level (recommended: the 22 new BARREL2
lines are honest, and depth would make this a yield slice, which the
founder ruled it is not).
S6-D4 STAGE3-EVAL-006 runs without adjudication, as section F.

## I. Stop conditions

Stop and report on: any proposal byte changing; any of the 1,393 silences
outside the three classes above; any fixture collision; any need for a
parser or dependency.
