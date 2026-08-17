# CallGraph Implementation State — 2026-08-16

Program: Spec 155 CallGraph execution authority (#254) + export (#287) +
envelopes (#289) + team routing (#292).

## Landed (all green: tests + check + clippy)

| Slice | Surface | Where |
| --- | --- | --- |
| Definition v1 | FocusaCallGraphDefinition, frames, edges, policies, evidence, authority | `crates/focusa-core/src/callgraph.rs` |
| Validation | identity/endpoints/entries/joins/compensation/per-cycle policy | `validate_graph` |
| Eligibility | §12 steps 1-5,12 — join settlement, depth bounds, parent edges | `eligibility_for_frame` |
| Ledger | definitions/revisions, runs, dispatches, frame leases (SQLite) | `crates/focusa-core/src/callgraph_store.rs` |
| Dispatch boundary | `commit_dispatch` must succeed before any adapter call | `commit_dispatch` |
| HTTP | validate/eligibility/definitions/preflight/run-create/run-read/control/export/envelope | `crates/focusa-api/src/routes/callgraph.rs` |
| Leases | acquire/refuse-while-live/release/lapsed-list | `acquire_lease`/`release_lease`/`lapsed_leases` |
| Liveness | 30s sweeper: lapsed leases → released, runs → WaitingJoin | `crates/focusa-api/src/main.rs` |
| Replay | deterministic frontier from (definition, dispatches) | `replay_frontier` |
| Routing | deterministic single + ranked team routing | `route_frame`, `route_frame_team` |
| Events | CallGraphFrameDispatched/Settled log-only FocusaEvent variants | `crates/focusa-core/src/types.rs`, reducer |
| Export | JSONL/TODO.txt/DOT/CSV/TSV/Mermaid from one projection | `crates/focusa-core/src/callgraph_export.rs` |
| Envelopes | canonical 9-layer ItemEnvelope + content digest | `crates/focusa-core/src/callgraph_envelope.rs` |
| CLI | `focusa callgraph export`, `focusa workstream migrate` | `crates/focusa-cli/src/commands/` |
| Golden fixture | canonical definition fixture + conformance tests | `crates/focusa-core/tests/` |

## Remaining

- Adapter-side execution binding (invoke the routed adapter through the
  daemon action loop) — last #254 integration slice.
- Frame settlement route (receipt → settle → replay).
- ASAP-Digest TODO.txt profile (gated on #288 exporter parity verification).
- Governed import (preview/commit/migration authority).
- Grid visualization (#286), Item Envelope consumers (#284/#286).
- Typed frame compiler (#295), CallGraph Pi tools (#296).
- Workset runtime (#267-#274).

## Authority invariants in force

- TODO.txt/export surfaces are projections — never canonical graph truth.
- Dispatches are durable before any adapter activity.
- Replay is deterministic; effects are never re-executed on replay.
- All liveness state is ledger-derived (restart-safe).
