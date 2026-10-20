# R6 measurement probe (throwaway, 2026-10-20, on d4488ac)

R6 fires when a consumer file imports anything from package P and reads,
anywhere in the file, two or more distinctive fields that coincide with
members of some object shape exported by P that the file does not import
by name. There is no receiver check. 3,133 R6 lines on the manifest
(3,104 in the six cloned repositories; freeCodeCamp's 29 not probed).

r6probe.py asks, for each line, whether a receiver in the consumer can be
tied to the shape without naming it, by text alone: a namespace import
(`r: ns.S`), an inline `import("spec").S`, or a call to a function imported
from P whose single declaration in P declares the shape (or Promise, array
of it) as its return type. r6-probe.tsv has one class per line.

| Class | Lines |
|---|---|
| YIELD (>= 2 distinctive reads through such a receiver; all by return type) | 39 (38 new pairs, 1 shape on an existing pair) |
| a route exists, < 2 reads | 6 |
| NO-ROUTE | 3,059 (98.6 percent) |

Blind adjudication, two agents per packet, rubric D, conservative merge:
- YIELD, all 39: 38 TRUE (97.4 percent), agreement 39 of 39. The one NOT
  TRUE reads through a parameter typed with a different interface of the
  same field names. 31 of the 38 are one pattern in pi's tests
  (`faux = registerFauxProvider(...)`, FauxProviderRegistration).
- NO-ROUTE, a seeded sample of 30 (random.Random("R6-PROBE-2026-10-20")):
  2 TRUE (6.7 percent), agreement 30 of 30. Both reach the shape through a
  property whose declared type is the shape, in another file
  (`requests[n].failed.message`, `element.crop`). The 28 others are
  coincidences: Math.max read as RangeProps.max, JSDoc text, object-literal
  keys, same-named fields of other types.

What it means: R6 is mostly a coincidence detector. The text-visible route
(imported function return types) yields 38 real contracts at 97 percent,
concentrated in one repository and one pattern. What remains real behind
the rest (an estimated 7 percent of 3,059, about 200; the 95 percent
Wilson interval from a sample of 30 is 1.8 to 21.3 percent, about 57 to
652 lines) is reachable only by following property
types across files, which needs a cross-file declaration graph.
