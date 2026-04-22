# SPEC81 Highest-Quality Follow-up Note

**Date:** 2026-04-22  
**Follow-up bead:** `focusa-g18b`

## What was improved

### 1) Truly strict tool inputs
The 10 core tree/metacognition tools now reject undeclared fields at runtime and use strict object schemas.

Key code:
- no-extra-fields runtime guard: `apps/pi-extension/src/tools.ts:1067`
- strict object schemas: `apps/pi-extension/src/tools.ts:1083`
- first strict tool surface starts at: `apps/pi-extension/src/tools.ts:1100`

### 2) Better LLM-facing tool output
Tool success content now carries useful summary details instead of only short status strings.

Examples:
- tree head summary includes branch/session: `apps/pi-extension/src/tools.ts:1129`
- tree diff summary includes version + decision change flags: `apps/pi-extension/src/tools.ts:1278`
- metacog retrieve summary includes top candidate info: `apps/pi-extension/src/tools.ts:1375`
- metacog reflect summary includes strategy updates: `apps/pi-extension/src/tools.ts:1411`
- metacog evaluate summary includes observed metrics: `apps/pi-extension/src/tools.ts:1484`

### 3) Stronger proof for real tool usefulness
Contract tests now verify:
- richer human-readable content exists
- extra parameters are rejected with `SCHEMA_INVALID`

Key test lines:
- meaningful summary text check: `tests/spec81_pi_extension_runtime_contract.ts:133`
- extra param rejection checks: `tests/spec81_pi_extension_runtime_contract.ts:179`

### 4) Actual prompt-level pickup smoke
A live `pi` smoke test now proves the model can pick up and use the extension tools through prompts against a mock Focusa API.

Key test lines:
- mock head endpoint: `tests/spec81_tool_pickup_smoke_test.sh:42`
- mock diff endpoint: `tests/spec81_tool_pickup_smoke_test.sh:56`
- head pickup prompt: `tests/spec81_tool_pickup_smoke_test.sh:88`
- diff pickup prompt: `tests/spec81_tool_pickup_smoke_test.sh:94`
- pass marker: `tests/spec81_tool_pickup_smoke_test.sh:123`

## Verification run

```bash
apps/pi-extension/node_modules/.bin/tsc -p apps/pi-extension/tsconfig.json
bash tests/spec81_impl_pi_extension_runtime_contract_test.sh
bash tests/spec81_tool_pickup_smoke_test.sh
bash tests/spec81_cli_high_order_runtime_test.sh
```

## Result

Pass.

This closes the remaining gap I found in the second audit:
- strictness is stronger
- tool summaries are more useful to the model
- there is now prompt-level evidence that tools get picked up and used
