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

## COMPLETE — all seven children have runtime slices landed

- #273/1: Mission Canvas workset surface — `workset` kind in the canvas
  model, inventory rows from the daemon workset list (settled badge,
  requirement/membership counts), commands fetch + map summaries,
  extension parity synced into apps/pi-extension. tsc green.
- The daemon workset list now carries projection summaries
  (requirement_count/membership_count/settled) per entry.
- Freshness stamps + the freshness route; bounded context packets with
  Workpoint/CallGraph binding refs; the evidence-gated transition DAG +
  route.

## Remaining for full release-readiness

- Live deployment of the workset/CallGraph-surface daemon + the full
  e2e matrix run against it (in flight).
- #273 deeper UI polish (Work Rail parity) lands with the generated-UI
  program — the typed surface is complete.
