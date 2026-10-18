# INVAR-BACKTEST-002: backtest replays strings contracts with their own reader

Status: DRAFT r1 for ruling. Implementation NOT authorized. Parent:
STAGE3-EVAL-001 findings S3-F-05, S3-F-06, S3-F-07. Baseline: the clean
tree at b484d3d. Built from the founder's outline of 2026-10-18 (23:49),
with the diagnosis corrected against the code in section 1.

## 0. The defect, and the honest bounds

On ten unseen repositories, backtest printed 61 absence events for locked
strings contracts, and all 61 were false: printed on producer bytes
identical to HEAD, where `invar check` reads every bound field as
present. Each was then followed by "never absent after introduction", a
reassurance about a field the replay never once saw.

Bounds, stated before any code:

B1. Backtest stays never-a-verdict. Exit codes unchanged: 0 replayed, 4
replayed with stated blind spots, 3 errors.
B2. One reader per contract kind, shared with check. Backtest must read a
past producer with exactly the function check uses on the live producer,
so the two can never disagree on the same bytes again. No new parser.
B3. Egress and schema replay is byte-identical, text and JSON.
B4. Machine surface frozen: the JSON keys and state labels do not change.
New human text arrives as new lines (the standing rule from INVAR-HOLDS-001
B4).

## 1. Diagnosis, corrected against the code

The outline locates the miss in the consumer's string array. The code says
otherwise. Backtest replays the PRODUCER's history only (`git log -- <producer>`),
and a strings contract's producer is the Zod shape (memory-item.ts and its
MemoryItemSchema), not the list. Check reads that producer with
`valuepairs::strings::read_any_shape(text, shape)`, the depth-1 key set of
the named shape (check.rs strings_outcome). Backtest instead calls
`check::source_fields`, the code-surface reader of egress and schema
contracts, which does not read a Zod object as that shape's key set. It
returns fields without the bound keys, so every binding reads absent.

So the fix is not a new string-array extractor (that would read the wrong
side of the contract, and would be a second parser that can drift from
check). It is one dispatch: strings contracts replay through
`read_any_shape`, the function check already trusts. There is also no
`Kind::Strings` in the engine; a contract's kind is carried by its frozen
id grammar (`strings:<Shape>@<listat>`), and the shape name is every
binding's scope.

## A. Frozen scope

A1. backtest.rs only. read_any_shape is already public in
valuepairs/strings.rs; check.rs and strings.rs stay byte-identical.
A2. Dispatch keys on the id prefix `strings:` (starts_with, never
contains), the same grammar check and lock already rely on.
A3. Past producer states map exactly as follows: Found(keys) gives present
or absent per binding; NotFound (the shape did not exist at that commit)
gives absent, which before introduction is never counted as a window;
Dynamic gives unextractable, a stated blind spot (exit 4), never guessed.

## B. Numbered requirements

R-B2-1 Strings contracts replay through read_any_shape with the shape name
taken from the bindings' scope; egress and schema keep source_fields
unchanged.
R-B2-2 A binding that is never present anywhere in the replayed window
renders `never present in the replayed window`, never `never absent after
introduction`. Text only; JSON events already state each commit's truth.
R-B2-3 When `git rev-parse --is-shallow-repository` answers true, the text
output gains one line after the header: `note: shallow history; the oldest
replayed commit is the shallow boundary, and its date is the cutoff, not a
change`. A full clone prints nothing new. Text only in v1.
R-B2-4 JSON byte-identical in keys and labels for every kind; only the
truth of strings events changes.
R-B2-5 Zero new dependencies, codes or exit changes; one new read-only
local git call (R-B2-3).

## C. Countermodels (defined before fixtures and code)

CM-B-1 a strings field present at every replayed commit reported absent
(the S3-F-05 defect itself).
CM-B-2 a field removed and restored in history reported without its window.
CM-B-3 a shape not yet introduced at an old commit counted as an absence
window (it is absence before introduction).
CM-B-4 a Dynamic shape guessed present or absent instead of stated blind.
CM-B-5 a never-present binding reassured as "never absent after
introduction" (S3-F-06).
CM-B-6 a shallow history replayed in silence (S3-F-07), or a full history
falsely labelled shallow.
CM-B-7 an egress or schema replay changing by one byte.
CM-B-8 the JSON surface gaining or losing a key.
CM-B-9 an egress contract whose id merely contains "strings" routed to the
strings reader.

## D. Fixtures (declared outcomes, before any code)

Synthetic git repositories built in the test, a Zod producer plus a list
consumer, locked with `invar lock`:
BT-F1 field present at v1, v2, v3: one `present` event,
`never absent after introduction`, exit 0 (CM-B-1).
BT-F2 field removed at v2, restored at v3: present, absent, present,
`1 absence window(s) after introduction` (CM-B-2).
BT-F3 shape introduced at v2: absent, present, zero windows (CM-B-3).
BT-F4 shape made dynamic (spread) at v2: an `unextractable` event, blind
spot stated, exit 4 (CM-B-4).
BT-F5 a field added only in the working tree before lock: `never present
in the replayed window` (CM-B-5).
BT-F6 the BT-F2 repository cloned with --depth 2: the shallow line present;
the full repository: absent (CM-B-6).
BT-N1 the existing egress backtest fixture stays green unedited, plus a
recorded golden of its text and JSON output asserted byte-identical
(CM-B-7).
BT-N2 a strings JSON object carries exactly the key set of an egress one
(CM-B-8).
BT-N3 an egress contract named with "strings" inside its id replays through
the code reader (CM-B-9).
BT-M1 two runs byte-identical.

## E. Mutation set (backtest's own; CAUGHT / SURVIVED / BUILD-FAILED)

The INVAR-HOLDS-001 harness targets main.rs and cannot judge backtest.rs,
so this arc declares its own: 1 strings routed back to source_fields
(BT-F1). 2 the shape name read from the field instead of the scope
(BT-F1). 3 NotFound mapped to present (BT-F3). 4 Dynamic mapped to absent
(BT-F4). 5 the never-present line reverted (BT-F5). 6 the never-present
condition inverted (BT-F1). 7 the shallow line printed always (BT-F6 full
half). 8 the shallow line never printed (BT-F6 shallow half). 9 the
shallow note added to JSON (BT-N2). 10 dispatch by contains instead of
starts_with (BT-N3). Each must die by a named assertion failure.

## F. Gates 0 to 5

Gate 0 hash ledger at b484d3d. Gate 1 fixtures first, failing against the
unchanged engine for the declared reasons (the N and M fixtures pass as
controls and stay). Gate 2 implementation to backtest.rs only. Gate 3 the
section E harness, all CAUGHT, control green. Gate 4 full suite green,
zero warnings, zero frozen-suite edits. Gate 5 ledger here, STATUS.md
entry. Re-measurement on unseen code is STAGE3-EVAL-002 on the same pinned
manifest; the STAGE3-EVAL-001 record is never overwritten.

## G. Path classes

Class 1 (untouched, hash-verified): the standing protected set, explicitly
including check.rs, valuepairs/strings.rs, lock.rs and main.rs. Class 2:
tests/backtest_test.rs, scripts/backtest2/. Class 3 (REQUESTED, not
assumed): backtest.rs, a protected file. Class 4: none.

## H. Decision points for ruling

B-D1 Reuse read_any_shape (recommended; B2, no second parser) versus a new
string-array extractor (the outline's form; rejected: it reads the
consumer, while backtest replays the producer).
B-D2 Dispatch on the `strings:` id prefix (recommended) versus a kind field
added to invar.lock (rejected: a lock format change for no gain).
B-D3 A shape NotFound at a past commit reads as absent, before
introduction (recommended) versus unextractable.
B-D4 Dynamic reads as unextractable, a blind spot with exit 4.
B-D5 The never-present wording, text only.
B-D6 The shallow statement, text only in v1; a JSON field would be its own
ruling with its own byte-stability analysis.
B-D7 Class 3 for backtest.rs.

## I. Stop conditions

Stop and report on: any need to touch check.rs, strings.rs or any other
Class 1 file; any byte change to egress or schema replay; any JSON key or
label change; any new parser for a shape check already reads; any frozen
fixture failing; any new dependency, code or exit change.
