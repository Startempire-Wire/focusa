# Spec88 Golden Eval Evidence Packet — Workpoint Continuity

Date: 2026-04-28
Spec: `docs/88-ontology-backed-workpoint-continuity.md`
Epic: `focusa-a2w2`
Phase: `focusa-a2w2.10` / Phase 9 golden evals and evidence packet

## Authority

Spec88 law: meaning lives in the typed Workpoint, not in the transcript.

Primary invariant: a resumed Pi agent must continue from Focusa reducer-owned `WorkpointResumePacket` after compaction, fork, model switch, provider overflow, or degraded local compact fallback.

## Golden Eval Matrix

| Eval | Trigger | Required survival evidence | Implemented gate |
| --- | --- | --- | --- |
| G1 compaction continuity | `session_before_compact` / `session_compact` | checkpoint before compaction; compact prompt includes `WorkpointResumePacket`; resumed directive names typed packet as canonical continuation contract; Pi submits exactly one hidden auto-resume turn | `apps/pi-extension/src/compaction.ts` contains `checkpointBeforeCompaction`, `refreshWorkpointResumePacket`, `## WorkpointResumePacket` injection, `triggerTurn: true`, and `lastCompactResumeKey` dedupe |
| G2 context overflow continuity | provider overflow-like response (`400`, `413`, `422`) | checkpoint with `context_overflow`; no blind retry without Workpoint checkpoint | `apps/pi-extension/src/turns.ts` contains `after_provider_response` and `checkpointDiscontinuity("context_overflow")` |
| G3 model switch continuity | `model_select` | checkpoint with `model_switch`; refresh `WorkpointResumePacket`; preserve provenance | `apps/pi-extension/src/turns.ts` calls `checkpointDiscontinuity("model_switch")` before emitting model-change signal |
| G4 fork continuity | `session_before_fork` | checkpoint with `fork`; refresh packet before branch divergence | `apps/pi-extension/src/session.ts` posts `/workpoint/checkpoint` with `checkpoint_reason: "fork"` and refreshes packet |
| G5 degraded fallback | Focusa unavailable during local compact fallback | non-canonical packet is persisted; session entry marks degraded fallback and must not silently promote | `apps/pi-extension/src/compaction.ts` appends `focusa-workpoint-fallback` and sets `canonical: false` |
| G6 drift detection | turn-end output diverges from active action intent | `/workpoint/drift-check` emits reducer drift event and telemetry trace | `apps/pi-extension/src/turns.ts` posts `emit: true`; `/work-loop/status` exposes `workpoint_replay_summary` |
| G7 pickup desirability | Pi tools are first-class pickup tools | checkpoint/resume tools expose bounded canonical/degraded envelopes | `apps/pi-extension/src/tools.ts` contains `focusa_workpoint_checkpoint` and `focusa_workpoint_resume` |
| G8 replay evidence | persisted events are replay-countable | replay summary counts checkpoint, resume, drift, degraded fallback | `crates/focusa-core/src/replay/mod.rs` contains `WorkpointReplaySummary` counters |

## ASAP-style Regression Scenario

Scenario: an agent previously verified `/api/audio/today` and Kokoro TTS for a homepage audio widget. After compaction, the agent must continue to inspect or patch the homepage audio widget play/pause/loading/error state binding. It must not drift to notes-only work, generic validation, or unrelated docs.

Expected Workpoint before boundary:

- `mission`: homepage UI renders real AI→TTS output.
- `current_action.action_type`: implementation/verification action targeting homepage audio widget.
- `verified_evidence`: `/api/audio/today` returned 200; Kokoro TTS path verified.
- `next_slice`: inspect or patch widget binding and verify UI behavior.
- `do_not_drift`: notes-only updates; generic validation; unrelated cleanup.

Golden acceptance:

1. Boundary event creates or refreshes a canonical Workpoint checkpoint.
2. Resume prompt contains `WorkpointResumePacket` with bounded packet JSON or rendered summary.
3. First resumed action targets homepage widget binding or verification.
4. Drift check flags notes-only/generic validation as drift from expected action intent.
5. Replay/status exposes a recent Workpoint drift/checkpoint signal without raw transcript dependency.

## Observed Compaction Resume Bug

During this implementation run, compaction emitted repeated visible `# Compaction Complete [compaction #60]` messages and then waited for a physical operator prompt. Root cause: the extension queued post-compact continuation as a steer plus direct `agent.continue()` retry loop, but Pi `sendCustomMessage` does not start a new idle turn unless `triggerTurn: true` is used. The gate now requires `triggerTurn: true` plus one-shot compaction-key dedupe.

## Evidence Commands

Run from repo root:

```bash
./tests/spec88_workpoint_golden_eval_contract_test.sh
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
cargo check -p focusa-cli -p focusa-api -p focusa-core --target-dir /tmp/focusa-cargo-target
cargo test -p focusa-core workpoint --target-dir /tmp/focusa-cargo-target
cargo test -p focusa-core replay::tests::test_workpoint_replay_summary_counts_recent_events --target-dir /tmp/focusa-cargo-target
```

## Closure Criteria

Phase 9 is complete when the contract test passes and validates all eight golden gates above, plus TypeScript and Rust compile/test gates remain green.
