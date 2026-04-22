# SPEC81 Extension Runtime Contract Note

**Date:** 2026-04-22  
**Spec:** `docs/81-focusa-llm-tool-suite-and-cli-development-reset-spec.md`  
**Task:** `focusa-p5hr.4`

## What was tested

Two things:

1. mocked extension runtime contract for all 10 required tools
2. live extension chain flow against the local daemon

## Test files

- contract test: `tests/spec81_pi_extension_runtime_contract.ts`
- live chain test: `tests/spec81_live_chain_extension_runtime_test.ts`
- runner: `tests/spec81_impl_pi_extension_runtime_contract_test.sh`

## What the tests prove

### Contract test
- all 10 tools are registered
- request shapes are correct
- shared envelope fields are present
- writer id headers are applied on write tools
- retry happens on transient retrieve failure
- stricter validation failures return `SCHEMA_INVALID`

Code:
- `tests/spec81_pi_extension_runtime_contract.ts:30`
- `tests/spec81_pi_extension_runtime_contract.ts:106`
- `tests/spec81_pi_extension_runtime_contract.ts:140`
- `tests/spec81_pi_extension_runtime_contract.ts:167`
- `tests/spec81_pi_extension_runtime_contract.ts:188`

### Live chain flow
- snapshot create works through extension tools
- snapshot diff works through extension tools
- metacognition capture -> retrieve -> reflect -> adjust -> evaluate works end to end
- final evaluation returns `promote_learning=true`

Code:
- `tests/spec81_live_chain_extension_runtime_test.ts:34`
- `tests/spec81_live_chain_extension_runtime_test.ts:109`
- `tests/spec81_live_chain_extension_runtime_test.ts:114`
- `tests/spec81_live_chain_extension_runtime_test.ts:120`

## Command run

```bash
bash tests/spec81_impl_pi_extension_runtime_contract_test.sh
```

## Result

Pass.
