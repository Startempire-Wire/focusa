# SPEC87 Existing Tool Desirability Upgrades Note

**Date:** 2026-04-22  
**Spec:** `docs/87-focusa-first-class-tool-desirability-and-pickup-spec.md`  
**Task:** `focusa-eulk.2`

## What changed

The existing tree/metacognition tools were upgraded to be more attractive to LLMs.

### Improvements made
- payoff-first descriptions instead of purely mechanical descriptions
- richer visible summaries instead of terse status-only text
- next-step hints (`next_tools=...`) to support tool chaining
- better zero-result guidance where applicable
- strict no-extra-fields validation for first-class tool params

## Key code

- runtime no-extra-fields guard: `apps/pi-extension/src/tools.ts:1067`
- strict object schemas: `apps/pi-extension/src/tools.ts:1083`
- payoff-first tree head description: `apps/pi-extension/src/tools.ts:1113`
- tree head chaining hint: `apps/pi-extension/src/tools.ts:1143`
- payoff-first metacog retrieve description: `apps/pi-extension/src/tools.ts:1356`
- metacog retrieve richer summary + next steps: `apps/pi-extension/src/tools.ts:1390`
- metacog evaluate richer summary + next steps: `apps/pi-extension/src/tools.ts:1500`

## Verification

Existing runtime tests still pass after the desirability upgrade:
- `tests/spec81_impl_pi_extension_runtime_contract_test.sh`
- `tests/spec81_cli_high_order_runtime_test.sh`

## Result

Existing first-class tools are now more obviously useful, easier for the model to chain, and stricter about input shape.
