# SPEC81 Tool Suite Audit Matrix

**Date:** 2026-04-22  
**Spec:** `docs/81-focusa-llm-tool-suite-and-cli-development-reset-spec.md`  
**Epic:** `focusa-p5hr`  
**Task:** `focusa-p5hr.1`

## What I checked

Required tools from Spec 81 section 3.1:

1. `focusa_tree_head`
2. `focusa_tree_path`
3. `focusa_tree_snapshot_state`
4. `focusa_tree_restore_state`
5. `focusa_tree_diff_context`
6. `focusa_metacog_capture`
7. `focusa_metacog_retrieve`
8. `focusa_metacog_reflect`
9. `focusa_metacog_plan_adjust`
10. `focusa_metacog_evaluate_outcome`

Code checked:
- tool helper layer in `apps/pi-extension/src/tools.ts`
- current CLI baseline in `crates/focusa-cli/src/commands/metacognition.rs`
- current CLI baseline in `crates/focusa-cli/src/commands/lineage.rs`

## Shared foundation already in place

Good news first:

- all 10 required tools are registered in the Pi extension
- tools already share one result envelope helper
- tools already share one error-code mapper
- write tools already request a writer id
- transient failures already get one retry for `0, 429, 502, 503, 504`

Key code:
- error mapping: `apps/pi-extension/src/tools.ts:916`
- result envelope: `apps/pi-extension/src/tools.ts:931`
- retry + writer-safe call path: `apps/pi-extension/src/tools.ts:955`

## Per-tool audit

| Tool | Present now | Writer-safe | Shared envelope | Main gaps to fix in 81.2 |
|---|---|---:|---:|---|
| `focusa_tree_head` | yes | no | yes | `session_id` has no max length; no trim/whitespace rule; needs stricter request bounds |
| `focusa_tree_path` | yes | no | yes | `clt_node_id` has min length only; needs max length + stricter id validation |
| `focusa_tree_snapshot_state` | yes | yes | yes | `clt_node_id` and `snapshot_reason` are unbounded; needs trim/max rules, reason length cap |
| `focusa_tree_restore_state` | yes | yes | yes | good enum handling for `restore_mode`, but `snapshot_id` still needs max length + shared id validator |
| `focusa_tree_diff_context` | yes | no | yes | both snapshot ids need trim/max/shared validator; current path trusts raw ids too much |
| `focusa_metacog_capture` | yes | yes | yes | `confidence` is bounded already; string fields are unbounded; needs max lengths and content quality limits |
| `focusa_metacog_retrieve` | yes | no | yes | `k` is bounded already; `current_ask` and `scope_tags` still need max length/max items rules |
| `focusa_metacog_reflect` | yes | yes | yes | `turn_range` needs shape validation; `failure_classes` needs max items + item length limits |
| `focusa_metacog_plan_adjust` | yes | yes | yes | `reflection_id` needs shared id validator; `selected_updates` needs max items + item length limits |
| `focusa_metacog_evaluate_outcome` | yes | yes | yes | `adjustment_id` needs shared id validator; `observed_metrics` needs max items + item length limits |

## Tool registration lines

- `focusa_tree_head` — `apps/pi-extension/src/tools.ts:979`
- `focusa_tree_path` — `apps/pi-extension/src/tools.ts:1002`
- `focusa_tree_snapshot_state` — `apps/pi-extension/src/tools.ts:1034`
- `focusa_tree_restore_state` — `apps/pi-extension/src/tools.ts:1057`
- `focusa_tree_diff_context` — `apps/pi-extension/src/tools.ts:1102`
- `focusa_metacog_capture` — `apps/pi-extension/src/tools.ts:1125`
- `focusa_metacog_retrieve` — `apps/pi-extension/src/tools.ts:1150`
- `focusa_metacog_reflect` — `apps/pi-extension/src/tools.ts:1175`
- `focusa_metacog_plan_adjust` — `apps/pi-extension/src/tools.ts:1198`
- `focusa_metacog_evaluate_outcome` — `apps/pi-extension/src/tools.ts:1221`

## Cross-tool delta list for 81.2

This is the actual work list for the next bead:

### 1) Add strict bounds everywhere
- add `maxLength` for all free-text strings
- add `maxItems` for all arrays
- add per-item length caps for string arrays

### 2) Add shared semantic validators
- one shared validator for ids like `snapshot_id`, `reflection_id`, `adjustment_id`, `clt_node_id`
- one shared validator for reason/ask/content text
- reject whitespace-only values after trim

### 3) Keep one result shape
Current shared envelope is good. Keep it, but make sure every tool returns the same shape on local validation failures too.

### 4) Keep writer-safe routing only on write tools
Current write routing looks right. Keep it explicit and do not widen writer headers to read-only tools.

### 5) Keep transient retry narrow
Current one-retry behavior is fine. Keep it limited to recoverable transport/server cases only.

## CLI baseline note for later beads

Current CLI already has primitive lineage and metacognition commands:
- lineage baseline: `crates/focusa-cli/src/commands/lineage.rs:6`
- metacognition baseline: `crates/focusa-cli/src/commands/metacognition.rs:7`

But Spec 81 still needs the higher-level commands later:
- `focusa metacognition loop run`
- `focusa metacognition promote`
- `focusa lineage compare`
- `focusa metacognition doctor`

Those belong to later beads, not this audit bead.

## Result

Audit complete.

There are no unknowns left for the tool-hardening phase. The next bead should implement shared validators, tighter schemas, and consistent local validation behavior across all 10 tools.
