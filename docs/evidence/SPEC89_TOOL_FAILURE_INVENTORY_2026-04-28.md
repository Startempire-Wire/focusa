# Spec89 Tool Failure Inventory — 2026-04-28

## Scope

Active bead: `focusa-bcyd.9.1` — inventory current failing Focusa tool paths before urgent fixes.

## Environment

- Focusa daemon health: `GET /v1/health` returned HTTP 200 with `ok=true`.
- Stress suite command: `./tests/focusa_tool_stress_test.sh`.

## Findings

### F1 — Workpoint checkpoint reason taxonomy mismatch

- Tool/API: `focusa_workpoint_checkpoint` / `POST /v1/workpoint/checkpoint`
- Input: `checkpoint_reason=operator_checkpoint`
- Expected: either accepted as an operator/manual checkpoint reason or rejected with clear field-level validation.
- Actual Pi tool result: `workpoint checkpoint unavailable → blocked: request failed (422)`.
- Actual API result:

```json
{"code":"validation_error","details":{"http_status":422,"reason":"Unprocessable Entity"},"message":"Request schema validation failed"}
```

- Classification: real tool/API contract failure.
- Root-cause hypothesis: API request deserializes `checkpoint_reason` directly into `WorkpointCheckpointReason`; enum lacks `operator_checkpoint`, while Spec88 tool input described operator checkpoint semantics and CLI maps unknown reasons to `unknown` instead of rejecting clearly.
- Follow-up beads: `focusa-bcyd.9.2`, `focusa-bcyd.9.3`.

### F2 — Workpoint validation errors are too generic at Pi boundary

- Tool/API: `focusa_workpoint_checkpoint` / `POST /v1/workpoint/checkpoint`
- Input: unsupported `checkpoint_reason=operator_checkpoint`.
- Expected: validation result names the invalid field, allowed values, and retry posture; not daemon unavailable/request failed.
- Actual: Pi summary says unavailable/blocked; API generic schema validation failed.
- Classification: real user-facing error envelope failure.
- Root-cause hypothesis: schema rejection occurs before handler can produce typed Workpoint validation response; Pi wrapper collapses non-2xx result to generic blocked text.
- Follow-up bead: `focusa-bcyd.9.3`.

### F3 — Stress suite drift no-drift case now false positives on do-not-drift boundary

- Tool/API: `POST /v1/workpoint/drift-check`
- Input from stress suite: latest action `stress verify FocusaToolSuite api cli pi`, expected action `stress_verify`, emit false.
- Expected: `status=no_drift` and `drift_detected=false`.
- Actual: `status=drift_detected`, class `do_not_drift_boundary`, reason `latest action touches prohibited boundary: Do not demote existing tools.`
- Classification: real classifier false positive and stress regression.
- Root-cause hypothesis: `latest_mentions_object()` uses substring token matching; boundary token `tools` matches inside `FocusaToolSuite`.
- Follow-up bead: add to urgent failure lane under regression/fix work; linked to `focusa-bcyd.9.6` and immediate fix.

### F4 — Constraint operator directive acceptance now passes

- Tool: `focusa_constraint`
- Input: `Operator directive forbids demotion of existing Focusa tools...`, source `operator directive`.
- Expected: accepted because operator directives are hard requirements.
- Actual: accepted.
- Classification: no current failure; keep regression test because an earlier wording failed.
- Follow-up bead: `focusa-bcyd.9.4` should add regression coverage and clarify accepted wording/source behavior.

### F5 — Focus State false-offline path not reproduced in current probe

- Tools: `focusa_recent_result`, `focusa_note`, `focusa_constraint`.
- Expected: successful writes while daemon healthy.
- Actual: all tested writes succeeded.
- Classification: no current recurrence; keep regression test because previous live failures existed.
- Follow-up bead: `focusa-bcyd.9.5`.

## Current stress result

`./tests/focusa_tool_stress_test.sh` returned:

```text
passed=35 failed=1
```

Only failure: F3 drift false positive.

## Next fixes

1. Align checkpoint reason taxonomy across core/API/CLI/Pi/docs/tests.
2. Improve Workpoint validation error envelope and Pi summary.
3. Fix drift classifier substring false positive.
4. Add regression tests for F1–F5.
5. Rerun stress suite to zero unexpected failures.
