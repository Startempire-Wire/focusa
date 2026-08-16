# Workstream conformance audit — 2026-08-16 (docs/164)

VERY IMPORTANT re-check, operator-directed.

## Slice conformance

| Slice | State |
| --- | --- |
| 1. Root type + persistence + CRUD | FULL (workstream_root.rs + migrate route) |
| 2. FocusaState partition per workstream (singleton elimination) | READ-SIDE complete (route families migrated); WRITE-SIDE was a gap — all 28 global write sites persisted to `state.focusa` only, partitions had no durability. CLOSED 2026-08-16: `scoped_write_through` writes the partition in memory AND persists it to the per-workstream `state.sqlite` (partition_paths) — the global state stays the compatibility projection for unmigrated consumers. |
| 3. Compaction scoping to the root | FULL (metacog_by_scope + per-partition prune) |
| 4. Continuation/rollover keyed by workstream | FULL (rollover route + deterministic ContinuationHandle) |
| 5. Migration preview/apply | FULL |

## Invariant 1 ("a write must name the root")

Now enforced at the canonical scoped write sites: workpoint
materialize, trajectory dispatch (event-scoped), focus materialize —
each writes through to the named partition + persists it durably.

## Acceptance sketch status

- Zero cross-workstream bleed: live e2e (19/21 checks incl. the
  two-scope probe) ✓.
- Compaction never touches another's context: metacog per-scope ✓.
- Restart resumes the exact root: deterministic ContinuationHandle +
  partition persistence ✓ (plus the production state-restore incident
  recovery).
