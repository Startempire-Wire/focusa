# SPEC80 Tool Suite Runtime Evidence (2026-04-21)

Status: implemented-now

## Code anchors
- Canonical envelope + code mapping + retry path:
  - `apps/pi-extension/src/tools.ts:916` `spec80ErrorCode`
  - `apps/pi-extension/src/tools.ts:931` `spec80Result`
  - `apps/pi-extension/src/tools.ts:955` `callSpec80Tool` (transient retry for `0/429/502/503/504`)
- Spec80 first-class tool registrations:
  - `focusa_tree_head` `apps/pi-extension/src/tools.ts:979`
  - `focusa_tree_path` `apps/pi-extension/src/tools.ts:1002`
  - `focusa_tree_snapshot_state` `apps/pi-extension/src/tools.ts:1034`
  - `focusa_tree_restore_state` `apps/pi-extension/src/tools.ts:1057`
  - `focusa_tree_diff_context` `apps/pi-extension/src/tools.ts:1102`
  - `focusa_metacog_capture` `apps/pi-extension/src/tools.ts:1125`
  - `focusa_metacog_retrieve` `apps/pi-extension/src/tools.ts:1150`
  - `focusa_metacog_reflect` `apps/pi-extension/src/tools.ts:1175`
  - `focusa_metacog_plan_adjust` `apps/pi-extension/src/tools.ts:1198`
  - `focusa_metacog_evaluate_outcome` `apps/pi-extension/src/tools.ts:1221`

## Runtime contract evidence (extension tool layer)
- Test file: `tests/spec80_pi_extension_runtime_contract.ts`
- Validates all 10 tools execute and return canonical envelope:
  - Tool envelope assertions at `tests/spec80_pi_extension_runtime_contract.ts:111`
- Validates exact outbound request shape/tool routing:
  - endpoint + body assertions at `tests/spec80_pi_extension_runtime_contract.ts:128`
- Validates k-clamp behavior (`k=999 -> 50`):
  - `tests/spec80_pi_extension_runtime_contract.ts:146`
- Validates writer-header consistency for write tools:
  - `tests/spec80_pi_extension_runtime_contract.ts:158`
- Validates retry-on-transient path for retrieve:
  - `tests/spec80_pi_extension_runtime_contract.ts:165`
- Validates invalid restore mode typed failure:
  - `tests/spec80_pi_extension_runtime_contract.ts:174`
- Runner wrapper:
  - `tests/spec80_impl_pi_extension_runtime_contract_test.sh`

## Runtime flow evidence (live daemon API)
- End-to-end scenario script:
  - `tests/spec80_live_flow_runtime_test.sh`
- Ephemeral daemon harness (self-start/stop):
  - `tests/spec80_impl_live_flow_with_ephemeral_daemon_test.sh`
- Validates live snapshot chain:
  - create/diff/restore + typed invalid restore code (`DIFF_INPUT_INVALID`) in `tests/spec80_live_flow_runtime_test.sh:45`
- Validates live metacognition chain:
  - capture/retrieve/reflect/adjust/evaluate + typed missing-reflection code (`REFLECTION_NOT_FOUND`) in `tests/spec80_live_flow_runtime_test.sh:107`

## Executed verification commands
- `npx tsc -p apps/pi-extension/tsconfig.json --pretty false`
- `bash tests/spec80_impl_pi_extension_runtime_contract_test.sh`
- `bash tests/spec80_impl_live_flow_with_ephemeral_daemon_test.sh`

Observed result: all green for the new runtime-quality Spec80 tool tests.
