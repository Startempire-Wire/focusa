# SPEC87 Tool Desirability Audit Matrix

**Date:** 2026-04-22  
**Spec:** `docs/87-focusa-first-class-tool-desirability-and-pickup-spec.md`  
**Epic:** `focusa-eulk`  
**Task:** `focusa-eulk.1`

## Audit dimensions

Each existing first-class tool was judged on:
- payoff clarity
- safety clarity
- setup friction
- visible summary usefulness
- likely LLM pickup blockers

## Findings

### Strong already
- real runtime behavior exists
- strict validation and typed envelope foundation already exists
- writer-safe routing exists for write tools

### Main pickup blockers before follow-up
1. descriptions explained mechanics more than payoff
2. visible success text was often too terse
3. some tools required ids the model might not have ready
4. workflow compression was still too weak for common multi-step tasks

## Tool-by-tool actions

| Tool | Biggest blocker | Needed action |
|---|---|---|
| tree_head | low payoff wording | rewrite description around safe starting point + branch context |
| tree_path | low payoff wording | emphasize ancestry/branch reasoning payoff |
| tree_snapshot_state | write-side caution unclear | clarify mutation and why snapshot is worth creating |
| tree_restore_state | scary write tool | make restore safety and purpose explicit |
| tree_diff_context | id friction + terse output | improve summary and add compare helper |
| metacog_capture | benefit not obvious | frame as reusable learning signal storage |
| metacog_retrieve | high value but terse | expose top hit and reuse payoff clearly |
| metacog_reflect | result needs next-step value | surface updates and likely follow-up |
| metacog_plan_adjust | id friction | add recent reflection helper and better summary |
| metacog_evaluate_outcome | id friction | add recent adjustment helper and better summary |

## Required helper/composite gaps

Needed to make tools more desirable:
- recent/latest snapshot helper
- recent reflection helper
- recent adjustment helper
- tree compare helper/composite
- metacognition loop helper/composite
- metacognition doctor helper/composite

## Result

Audit complete. Remaining work belongs to:
- `focusa-eulk.2` descriptions/summaries/chaining
- `focusa-eulk.3` helper/composite tools
- `focusa-eulk.4` pickup/effectiveness proof
