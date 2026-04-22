# SPEC87 Pickup and Effectiveness Proof Note

**Date:** 2026-04-22  
**Spec:** `docs/87-focusa-first-class-tool-desirability-and-pickup-spec.md`  
**Task:** `focusa-eulk.4`

## Test files

- desirability contract: `tests/spec87_extension_desirability_contract.ts`
- pickup/effectiveness smoke: `tests/spec87_tool_pickup_and_effectiveness_smoke_test.sh`
- runner: `tests/spec87_impl_tool_desirability_test.sh`

## What was proven

### Contract proof
- new helper/composite tools are registered
- descriptions now advertise payoff better
- richer summaries are returned
- composites call the right backend flow
- doctor uses retrieve with `summary_only=true`

Key lines:
- `tests/spec87_extension_desirability_contract.ts:154`
- `tests/spec87_extension_desirability_contract.ts:157`

### Pickup/effectiveness smoke proof
- real `pi` picks up the recent reflections helper when the prompt asks for a recent reflection id
- real `pi` picks up the diagnostic path and uses `summary_only=true` retrieve for signal-quality work
- the outputs contain the expected useful result content

Key lines:
- mock recent reflection id: `tests/spec87_tool_pickup_and_effectiveness_smoke_test.sh:43`
- prompt output expectation for helper: `tests/spec87_tool_pickup_and_effectiveness_smoke_test.sh:106`
- pass marker: `tests/spec87_tool_pickup_and_effectiveness_smoke_test.sh:118`

## Commands run

```bash
bash tests/spec87_impl_tool_desirability_test.sh
bash tests/spec81_impl_pi_extension_runtime_contract_test.sh
bash tests/spec81_cli_high_order_runtime_test.sh
```

## Result

Pass.
