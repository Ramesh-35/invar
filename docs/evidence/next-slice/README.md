# Next-slice evidence (2026-10-19), throwaway prototypes, never the work tree

Option 2, tsconfig `paths` in egress discovery (K1), OmniRoute at its pinned SHA:
619 alias abstentions resolved, 0 new proposals (6 before, 6 after); abstentions
682 to 142 (the 619 generic ones become 54 TSC-GOVERNED, 22 BARREL2, 3
AMBIG-EGRESS); 1,393 imports newly reach K1's silent NotShape branch: 1,389
earned (1,339 values, 50 types), 4 unearned (2 Zod schemas unreached, 2 names
declared nowhere). Landing it needs earned silence in K1 as well.

Barrel depth (re-exports followed up to 5 levels, cycle guarded, one distinct
declaration required) plus `export type *` parsing, code-pair discovery:
pi 113 to 139 proposals (BARREL2 118 to 0), puppeteer 5 to 8 (unreached types
25 to 16), excalidraw and playwright unchanged; 0 AMBIG-SHAPE; deterministic.
Total +29 proposals. Barrel depth alone gives pi +26; `export type *` alone
gave 0 (SLICE3B census); together they give puppeteer +3.
