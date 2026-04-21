# SPEC80 Metacognitive Tool Contracts — 2026-04-21

Purpose: implement `focusa-yro7.2.2` by codifying machine-reviewable contracts for SPEC80 metacognitive compounding tools from §6.2 + Appendix A §14.2, bound to Appendix B §15 API/CLI paths.

## Contract baseline

Authority source:
- `docs/80-pi-tree-li-metacognition-tooling-spec.md` §6.2, §14.2, §15, §19.3

Mandatory contract invariants:
1. Stable JSON input/output for each metacognitive tool.
2. Explicit error-code catalog per tool.
3. Declared ontology layer coverage.
4. Binding matrix references to primary API path and CLI fallback target.
5. Anti-false-weaving label assignment per §19.3.

## Metacognitive compounding contract set

| Tool | Contract intent | Required input | Success output (shape) | Required error codes | Layers | Label (§19.3) |
|---|---|---|---|---|---|---|
| `focusa_metacog_capture` | Structured capture of strategy signal | `kind`, `content`, `rationale?`, `confidence?`, `strategy_class?` | `{ capture_id, stored, linked_turn_id }` | `CAPTURE_SCHEMA_INVALID` | `6,11,12` | `planned-extension` |
| `focusa_metacog_retrieve` | Retrieve prior strategy packets by current ask/context | `current_ask`, `scope_tags[]`, `k` | `{ candidates:[...], ranked_by, retrieval_budget }` | `RETRIEVE_UNAVAILABLE`, `RETRIEVE_BUDGET_EXCEEDED` | `11,7,12` | `planned-extension` |
| `focusa_metacog_reflect` | Reflection packet with hypothesis + strategy updates | `turn_range`, `failure_classes[]?` | `{ reflection_id, hypotheses:[...], strategy_updates:[...] }` | `REFLECT_INPUT_INVALID` | `11,6,12` | `planned-extension` |
| `focusa_metacog_plan_adjust` | Apply selected updates into next-step policy | `reflection_id`, `selected_updates[]` | `{ adjustment_id, next_step_policy, expected_deltas }` | `ADJUST_POLICY_CONFLICT` | `11,9,5` | `planned-extension` |
| `focusa_metacog_evaluate_outcome` | Evaluate observed impact and learning promotion | `adjustment_id`, `observed_metrics` | `{ evaluation_id, delta_scorecard, promote_learning }` | `EVAL_INPUT_INVALID` | `12,6,7,11` | `planned-extension` |

## Binding realization notes

Primary API targets (planned-extension):
- `POST /v1/metacognition/capture`
- `POST /v1/metacognition/retrieve`
- `POST /v1/metacognition/reflect`
- `POST /v1/metacognition/adjust`
- `POST /v1/metacognition/evaluate`

CLI fallback targets (planned-extension):
- `focusa metacognition capture --json`
- `focusa metacognition retrieve --json`
- `focusa metacognition reflect --json`
- `focusa metacognition adjust --json`
- `focusa metacognition evaluate --json`

Current code-reality checkpoint:
- Dedicated `/v1/metacognition/*` API routes are not implemented yet.
- `/v1/reflect/*` exists as adjacent but not contract-equivalent surface.

## Evidence citations

- docs/80-pi-tree-li-metacognition-tooling-spec.md (§6.2, §14.2, §15, §19.3)
- crates/focusa-api/src/routes/reflection.rs
