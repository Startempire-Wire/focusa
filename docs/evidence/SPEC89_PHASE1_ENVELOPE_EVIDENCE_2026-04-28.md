# Spec89 Phase 1 Unified Result Envelope Evidence — 2026-04-28

Active phase: `focusa-bcyd.2`.

## Implementation

`apps/pi-extension/src/tools.ts` now installs a thin `pi.registerTool` wrapper inside `registerTools()` so every registered `focusa_*` tool returns legacy Pi content plus `details.tool_result_v1`.

Shared helpers:

- `FocusaToolResultV1`
- `focusaToolResult()`
- `focusaToolDetails()`
- `inferToolResult()`
- `withToolResultEnvelope()`

## Coverage by family

| Family | Coverage mechanism |
|---|---|
| Focus State and scratchpad | Wrapper infers accepted/completed, validation rejected, offline/write failures, side effects, retry posture, and raw details. |
| Workpoint | Wrapper preserves legacy Spec88 output and adds canonical/degraded/status/retry/next-tools envelope fields. |
| Tree/snapshot | Wrapper adds read-vs-state-changing side effects and safe/check-side-effects retry posture. |
| Metacog and lineage intelligence | Wrapper marks read-like helpers safe retry and write-like artifacts as side-effecting. |
| Work-loop | Wrapper preserves writer details in raw payload and adds blocked/offline/error envelope taxonomy. |

## Validation

- `cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit` passed.
- `./tests/spec89_tool_envelope_contract_test.sh` passed.

## Compatibility

Visible `content[0].text` summaries remain unchanged. Existing `details` fields remain unchanged and are extended by `details.tool_result_v1`; raw compatibility is preserved through the `raw` slot.
