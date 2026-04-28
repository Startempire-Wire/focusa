# Spec88 Critical Implementation Audit — 2026-04-28

## Scope

Operator directive: low-quality implementation counts as an implementation gap.

This audit rereads `docs/88-ontology-backed-workpoint-continuity.md` against the current code and treats marker-only gates, weak semantics, and missing lifecycle behavior as gaps.

## Reopened Beads

- `focusa-a2w2` — epic reopened for critical repair.
- `focusa-a2w2.7` — lifecycle/compaction integration gaps.
- `focusa-a2w2.8` — overflow/model switch/degraded recovery gaps.
- `focusa-a2w2.9` — drift detection quality gaps.
- `focusa-a2w2.10` — golden eval gate quality gaps.

## Findings and Repairs

### G1 — Workpoint packet not injected into `before_agent_start` / context slice

Spec88 §12.2 and §12.3 require a behavioral law and typed context sections: `WORKPOINT`, `ACTIVE_OBJECT_SET`, `ACTION_INTENT`, `VERIFICATION_HOOKS`, `DRIFT_BOUNDARIES`.

Previous implementation only stored `S.activeWorkpointPacket` and used compaction summaries. Ordinary turns could still rely on transcript tail.

Repair:

- `apps/pi-extension/src/turns.ts` now adds a `Focusa Workpoint Continuity Law` to `before_agent_start` when a packet exists.
- The context hook now injects typed Workpoint sections with highest priority.

### G2 — Session resume did not create a low-confidence checkpoint when no Workpoint exists

Spec88 §12.1 and §14 require creating/refreshing Workpoints on session start/resume when none exists.

Previous implementation only tried to fetch `/workpoint/resume`; no checkpoint was created from Focus State/work-loop/current ask when absent.

Repair:

- `apps/pi-extension/src/session.ts` now creates a low-confidence canonical checkpoint on session start/switch when no active packet exists, then refreshes the resume packet.

### G3 — Overflow detection was too shallow

Spec88 §12.7 requires provider context overflow to checkpoint before trim/retry and avoid blind retry.

Previous implementation only keyed on HTTP status `400/413/422`; it ignored overflow text and could treat any 400 as overflow.

Repair:

- `apps/pi-extension/src/turns.ts` now has `providerStatusSuggestsContextOverflow()` and `textSuggestsContextOverflow()`.
- It checkpoints on overflow-like provider headers/status and assistant/error text containing `context_length_exceeded` style signatures.

### G4 — Drift detection was marker-level, not semantic enough

Spec88 §12.9 and §15.3 require drift classes like notes-only work, wrong object, repeated validation, action-intent ignore, and work-item switching.

Previous implementation only checked whether assistant output contained the expected action type substring. That would miss notes-only drift and wrong-object work.

Repair:

- `crates/focusa-api/src/routes/workpoint.rs` now classifies drift by action intent, active target object mention, notes-only markers, and `DO_NOT_DRIFT` boundaries.
- Event-emitting drift checks now require `work-loop:write`, while preview remains read-only.
- Responses include `drift_classes`, `severity`, `reason`, and `recovery_hint`.

### G5 — Checkpoint idempotency was only accepted, not implemented

Spec88 §9.2 and §15.4 require idempotency behavior, not a decorative field.

Previous API accepted no idempotency key; CLI described it as informational.

Repair:

- `WorkpointRecord` now stores `idempotency_key`.
- `/v1/workpoint/checkpoint` returns existing record with `idempotent_replay: true` for repeated keys.
- Resume packets include the key for auditability.

### G6 — Golden eval was marker-heavy

Spec88 §15 requires acceptance tests that would catch continuity and drift failures.

Previous contract test verified surface markers but would not catch the missing context injection, low-confidence resume checkpoint, semantic drift classification, or real idempotency.

Repair:

- `tests/spec88_workpoint_golden_eval_contract_test.sh` now asserts context/before-agent Workpoint sections, low-confidence resume checkpoint creation, overflow text handling, semantic drift classifier, permissioned drift emission, and idempotency replay markers.

## Validation

- `./tests/spec88_workpoint_golden_eval_contract_test.sh`
- `cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit`
- `cargo test -p focusa-api drift_classifier --target-dir /tmp/focusa-cargo-target`
- `cargo test -p focusa-api workpoint_packet --target-dir /tmp/focusa-cargo-target`
- `cargo check -p focusa-core -p focusa-api -p focusa-cli --target-dir /tmp/focusa-cargo-target`

## Remaining Caveat

This pass improves implementation quality and gates the obvious low-quality gaps. Full end-to-end provider-overflow simulation still depends on harness-level provider error reproduction, which should be a future live integration test rather than a marker-only contract.
