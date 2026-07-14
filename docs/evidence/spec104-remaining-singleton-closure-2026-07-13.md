# Spec104 remaining singleton closure evidence — 2026-07-13

Scope: bounded host runtime state, API metacognition runtime/persistence, and menubar bridge runtime state.

Proof run (short non-building checks only):

```text
$ cargo fmt --all
$ python3 tests/spec104_remaining_singleton_closure_static_test.py
spec104 remaining singleton closure static proof: ok
$ python3 tests/spec104_singleton_inventory_gate.py
Spec104 inventory: findings=19 classified=32 open=1 closure=False
OPEN PI-01 apps/pi-extension/src/state.ts::S -> eliminate
PASS: every detected singleton/non-scoped state marker is classified; no unknown or stale inventory entries
$ git diff --check
```

Proven closures:

- API-05 `METACOG_STORE`: eliminated. Metacognition routes use request-local `ScopeContext::require_workstream_key()` and `AppState.metacog_by_scope` keyed by typed `WorkstreamKey`.
- API-05 `GLOBAL_DIR:runtime/metacognition`: eliminated. Metacognition persistence moved to `runtime/scoped-metacog/<WorkstreamKey::storage_key()>/*`.
- MBN-01 `BRIDGE_COMPLETIONS` and `BRIDGE_LISTENERS`: eliminated. Menubar bridge runtime state is Tauri-managed `BridgeRuntimeState`, keyed by typed `BridgeAttachmentKey`.
- Bounded resource/pressure statics: converted from unkeyed mutable runtime cells to typed Host `ScopeRef` storage-key maps with no unkeyed process-global mutable fallback.

Remaining open inventory: Pi extension `apps/pi-extension/src/state.ts::S` (intentionally untouched in this slice).

## Work-loop scope enforcement — 2026-07-13

- Every work-loop route now uses `WorkLoopScope`, which requires explicit request `WorkstreamKey` and exact canonical active-Workpoint root+continuity.
- `routes::work_loop::tests::writer_scope_rejects_host_and_cross_continuity_authority` passed: exact project scope accepted; cross-continuity and Host `ScopeRef` rejected.
- Full `focusa-api` suite passed 337 tests before the new targeted test; API check remains green after enforcement.
