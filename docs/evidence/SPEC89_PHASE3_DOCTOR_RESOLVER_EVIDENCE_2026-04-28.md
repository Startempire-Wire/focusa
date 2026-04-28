# Spec89 Phase 3 Doctor/Resolver/Evidence Evidence — 2026-04-28

Active phase: `focusa-bcyd.4`.

## Implemented in this slice

- `focusa_tool_doctor` Pi composite: probes daemon health, active Workpoint, and work-loop status, then returns readiness summary plus `tool_result_v1` via the global wrapper.
- `focusa_active_object_resolve` Pi tool: resolves active object candidates from active Workpoint packet, action target, work item, and optional hint without claiming canonical verification.
- `focusa_evidence_capture` Pi tool: captures bounded evidence refs and links them to active Workpoint through `/v1/workpoint/evidence/link` unless `attach_to_workpoint=false`.
- Existing Workpoint evidence API/reducer path from Phase 2 serves as evidence capture API path.

## Validation

- `cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit` passed.
- `./tests/spec89_tool_envelope_contract_test.sh` passed and now enumerates 39 `focusa_*` tools.

## Remaining Phase 3 work

CLI parity for doctor/resolver/evidence remains open in beads `focusa-bcyd.4.3`, `focusa-bcyd.4.6`, and `focusa-bcyd.4.9`.
