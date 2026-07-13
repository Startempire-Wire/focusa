# Spec104 API scope singleton migration evidence — 2026-07-13 (partial)

Scope: `crates/focusa-api/src/routes/turn.rs`, `snapshots.rs`, request scope core, and `AppState` scoped runtime state.

Proof run (short non-building checks only):

```text
$ cargo fmt --all
$ python3 tests/spec104_api_scope_singleton_closure_static_test.py
spec104 api scope singleton closure static proof: ok
$ python3 tests/spec104_singleton_inventory_gate.py
Spec104 inventory: findings=22 classified=32 open=5 closure=False
PASS: every detected singleton/non-scoped state marker is classified; no unknown or stale inventory entries
```

Proven closures:

- API-06 `RECENT_COMPLETED_TURNS`: eliminated static singleton; turn completion hot dedupe is now `AppState.recent_completed_turns_by_scope`, keyed by typed `WorkstreamKey` built from request-local scope.
- API-07 `SNAPSHOTS`: eliminated static singleton; snapshot hot store and disk index path are keyed by typed `WorkstreamKey`.
- Bounded runtime statics remain open: operator policy requires explicit typed Host ScopeRef state rather than process-global mutable infrastructure.

Known remaining open inventory outside this slice: API-05 metacognition store/global dir, menubar bridge runtime maps, and Pi extension state.
