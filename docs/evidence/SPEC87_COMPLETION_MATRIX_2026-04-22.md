# SPEC87 Completion Matrix

**Date:** 2026-04-22  
**Spec:** `docs/87-focusa-first-class-tool-desirability-and-pickup-spec.md`  
**Epic:** `focusa-eulk`

## Final result

Spec 87 is complete.

## Acceptance check

### 1) Existing tools are more attractive and informative to LLMs
Pass.

Examples:
- payoff-first tree head description: `apps/pi-extension/src/tools.ts:1113`
- payoff-first metacog retrieve description: `apps/pi-extension/src/tools.ts:1356`
- chaining hint in tree head output: `apps/pi-extension/src/tools.ts:1143`
- chaining hint in metacog retrieve output: `apps/pi-extension/src/tools.ts:1390`
- richer evaluate output: `apps/pi-extension/src/tools.ts:1500`

Supporting note:
- `docs/evidence/SPEC87_EXISTING_TOOL_DESIRABILITY_UPGRADES_NOTE_2026-04-22.md:1`

### 2) New helper/composite tools reduce setup friction and workflow length
Pass.

New tools:
- `apps/pi-extension/src/tools.ts:1507`
- `apps/pi-extension/src/tools.ts:1539`
- `apps/pi-extension/src/tools.ts:1618`
- `apps/pi-extension/src/tools.ts:1650`
- `apps/pi-extension/src/tools.ts:1682`
- `apps/pi-extension/src/tools.ts:1784`

Supporting backend routes:
- `crates/focusa-api/src/routes/snapshots.rs:467`
- `crates/focusa-api/src/routes/metacognition.rs:671`
- `crates/focusa-api/src/routes/metacognition.rs:673`

Supporting note:
- `docs/evidence/SPEC87_HELPER_AND_COMPOSITE_TOOLS_NOTE_2026-04-22.md:1`

### 3) Prompt-level pickup tests pass
Pass.

Key proof:
- `tests/spec87_extension_desirability_contract.ts:157`
- `tests/spec87_tool_pickup_and_effectiveness_smoke_test.sh:118`
- `docs/evidence/SPEC87_PICKUP_AND_EFFECTIVENESS_PROOF_NOTE_2026-04-22.md:1`

### 4) Completion packet ties desirability claims to code and test output
Pass.

Evidence pack:
- `docs/evidence/SPEC87_TOOL_DESIRABILITY_AUDIT_MATRIX_2026-04-22.md:1`
- `docs/evidence/SPEC87_EXISTING_TOOL_DESIRABILITY_UPGRADES_NOTE_2026-04-22.md:1`
- `docs/evidence/SPEC87_HELPER_AND_COMPOSITE_TOOLS_NOTE_2026-04-22.md:1`
- `docs/evidence/SPEC87_PICKUP_AND_EFFECTIVENESS_PROOF_NOTE_2026-04-22.md:1`
- `docs/evidence/SPEC87_COMPLETION_MATRIX_2026-04-22.md:1`

## Final verification run

```bash
cargo check -p focusa-api -p focusa-cli
apps/pi-extension/node_modules/.bin/tsc -p apps/pi-extension/tsconfig.json
bash tests/spec81_impl_pi_extension_runtime_contract_test.sh
bash tests/spec81_cli_high_order_runtime_test.sh
bash tests/spec87_impl_tool_desirability_test.sh
```

## Result

Pass.

- cargo check: pass
- TypeScript compile: pass
- Spec81 extension/runtime regression: pass
- Spec81 CLI regression: pass
- Spec87 desirability contract + pickup smoke: pass

## Bottom line

The Focusa tool layer is now materially more desirable for LLM use:
- better descriptions
- better visible summaries
- stronger next-step guidance
- less id friction
- new helper/composite tools
- real pickup/effectiveness proof
