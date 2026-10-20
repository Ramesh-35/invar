# STAGE3-EVAL-006: INIT-DISCOVERY-SLICE6 on the pinned manifest

Engine: INIT-DISCOVERY-SLICE6 (root tsconfig paths and earned silence in
egress discovery) on d4488ac, release binary 03f3e34956d7. Manifest hash
2f5b6b0f798b, instrument r2, the same ten pinned repositories as
STAGE3-EVAL-001 to -005, all of which stand unmodified.

Verdict: PASS on every pre-registered target. No adjudication (ruling
S6-D4): no proposal changes.

| Pre-registered (founder, 2026-10-20) | Measured |
|---|---|
| 113 proposals, byte-identical to EVAL-005 (OmniRoute 6) | 113; every proposal section byte-identical in all 10 repositories |
| OmniRoute: 619 path abstentions become 79 specific ones | 619 removed; 79 added: 54 TSC-GOVERNED, 22 BARREL2, 3 AMBIG-EGRESS |
| (draft figure) OmniRoute abstentions 682 to 144 | 682 to 144 (the 79 plus 2 UNRESOLVED Zod non-object schemas, on 63 untouched) |
| 0 silent drops, audited over all 1,393 reached imports | 0: the engine is silent on exactly the 1,391 the independent classifier calls earned and loud on exactly the 2 it calls unearned (1,393 of 1,393 agree, m-s-4/k1-audit.txt) |
| suite green, 0 warnings | 343 of 343, 0 warnings |
| 10 of 10 exit 0, deterministic | 10 of 10, raw bytes; the 9 repositories other than OmniRoute byte-identical to EVAL-005 |

Also measured: M-S-5 round trip unchanged, 88 of 88 TRUE contracts lock,
hold with the durability line and re-check byte-identical, every lock
byte-identical to EVAL-005 apart from its recorded_at; M-S-6 backtest
byte-identical to EVAL-005 on all 4. M-S-7: init 0 to 8 seconds per
repository.

The 2 new loud lines (OmniRoute):

    UNRESOLVED src/app/api/v1/management/proxy-subscriptions/[id]/route.ts:3 proxySubscriptionUpdateSchema from "@/lib/proxySubscription": declared as a Zod schema at src/lib/proxySubscription/schema.ts:207 that is not an object shape; not guessed
    UNRESOLVED src/app/api/v1/management/proxy-subscriptions/route.ts:8 proxySubscriptionCreateSchema from "@/lib/proxySubscription": declared as a Zod schema at src/lib/proxySubscription/schema.ts:139 that is not an object shape; not guessed

Method of the M-S-4 audit: an audit build of the same source (one
environment-gated stderr line per verdict, output otherwise identical to
the evaluation binary, verified byte for byte on OmniRoute) logged the
engine's earned-silence verdict for every import reaching the silent
branch; each was compared with docs/evidence/slice6/classify_k1.py, written
independently of the engine on masked text.
