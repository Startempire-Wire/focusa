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

## Compiled/runtime reconciliation — 2026-07-13

- `cargo test -p focusa-api --no-fail-fast`: 337 tests passed after adding mandatory `WorkstreamKey` to stale ontology fixtures.
- Focusa project identity: `/home/wirebot/focusa` verified high-confidence; broad `/root` rejected unsafe/low-confidence.
- Current-continuity Workpoint resume returned canonical; alternate continuity returned `not_found`, canonical false, with no fallback authority.
- `focusa_metacog_doctor`, recent snapshots, and bounded telemetry traversal succeeded through scoped Pi tools.
- Prediction Pi tools still fail in wrapper rendering with `undefined.status`; `focusa-fsrc.14` remains open despite compiled API tests.

This evidence supports API-01/02/03/05/06/07 only; API-04 is explicitly excluded from closure.
