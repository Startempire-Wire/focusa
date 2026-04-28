# Spec89 Phase 5 Dedupe and State Hygiene Evidence — 2026-04-28

Active phase: `focusa-bcyd.6`.

## Implemented

- Cognitive write dedupe helpers in Pi bridge:
  - `cognitiveWriteKey()` normalizes cognitive write payloads or uses explicit keys.
  - `duplicateCandidateForWrite()` maintains a bounded recent-key window for duplicate candidate detection.
- State hygiene tools:
  - `focusa_state_hygiene_doctor`: read-only stale/duplicate diagnostic surface.
  - `focusa_state_hygiene_plan`: proposal-style plan; non-mutating.
  - `focusa_state_hygiene_apply`: approval-gated, non-destructive placeholder; blocks without approval and never deletes silently.
- Merge/update/supersede path represented by hygiene plan guidance: review duplicate candidates, prefer supersede/update over deletion, apply only with explicit approval.

## Validation

Commands passed:

```bash
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
./tests/spec89_tool_envelope_contract_test.sh
```

Envelope skeleton now checks:

- 43 current `focusa_*` tools.
- `cognitiveWriteKey` dedupe helper.
- `focusa_state_hygiene_doctor` tool.

## Safety semantics

State hygiene is proposal-first and approval-safe. No tool silently deletes or rewrites Focus State; current apply path is a no-op acknowledgement unless future reducer-supported hygiene events are added.
