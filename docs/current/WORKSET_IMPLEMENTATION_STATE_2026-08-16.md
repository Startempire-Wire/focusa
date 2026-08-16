# Workset implementation state — 2026-08-16

Spec 149 says "FINAL APPROVAL AND IMPLEMENTATION NOT AUTHORIZED" — the
operator directive of 2026-08-16 ("FINISH. THEN WORKSET") is the
authorization. Implementation follows the #267 authority separation
strictly.

## Landed

| Slice | What |
| --- | --- |
| #269/1 | workset_ledger: definitions, append-only events (admit/dispose/membership/contract), deterministic replay, canonical digests, settlement = all required requirements met |
| #269/2 | workset_store: SQLite definitions + events; replay from the store |
| #271/1 | /v1/worksets CRUD + events + projection (replay_rejected on ledger errors) |
| #271/2 | focusa workset define/event/projection CLI |
| #271/3 | focusa_workset_projection Pi tool |
| #270/1 | workset_providers: provider projections reconciled against the ledger; the ledger wins every conflict |

## Authority separation (invariants)

- Workset = membership boundary, requirement disposition, completion
  contract, immutable history. NEVER scheduling/execution — CallGraph
  owns execution.
- Providers CLAIM dispositions; the ledger decides.
- Settlement is deterministic replay, never a mutable flag.

## Next slices

#268 working-set freshness; #272 bounded context + Workpoint/CallGraph
binding; #273 generated Mission Canvas/Work Rail surface; #274
checkpoint/completion/release transitions.
