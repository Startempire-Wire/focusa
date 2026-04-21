# SPEC80 Error + Permission Model (Normalized) — 2026-04-21

Purpose: implement `focusa-yro7.2.4` by normalizing error-code and permission semantics across all SPEC80 tool contracts from §14 and §15.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§14.1, §14.2, §15, §19.3)

## Normalization principles

1. Every tool MUST define at least one typed error code.
2. Permission scope MUST be explicitly declared and consistent with operation class:
   - lineage read operations -> `lineage:read`
   - state snapshot/restore write operations -> `state:write`
   - metacognitive write operations -> `metacognition:write`
   - metacognitive retrieval operations -> `metacognition:read`
3. Error code prefixes SHOULD map to capability domains for auditability:
   - `TREE_`, `CLT_`, `SNAPSHOT_`, `RESTORE_`, `DIFF_`
   - `CAPTURE_`, `RETRIEVE_`, `REFLECT_`, `ADJUST_`, `EVAL_`
4. Adjacent-but-non-equivalent surfaces MUST NOT be treated as contract closure (e.g., `/v1/reflect/*` vs `/v1/metacognition/*`).

## Canonical error catalog

### Tree/lineage bridge errors
- `TREE_HEAD_UNAVAILABLE`
- `SESSION_NOT_FOUND`
- `CLT_NODE_NOT_FOUND`
- `SNAPSHOT_WRITE_DENIED`
- `SNAPSHOT_CONFLICT`
- `SNAPSHOT_NOT_FOUND`
- `RESTORE_CONFLICT`
- `AUTHORITY_DENIED`
- `DIFF_INPUT_INVALID`

### Metacognitive compounding errors
- `CAPTURE_SCHEMA_INVALID`
- `RETRIEVE_UNAVAILABLE`
- `RETRIEVE_BUDGET_EXCEEDED`
- `REFLECT_INPUT_INVALID`
- `ADJUST_POLICY_CONFLICT`
- `EVAL_INPUT_INVALID`

## Canonical permission map by tool

| Tool | Required permission | Operation class | Label (§19.3) |
|---|---|---|---|
| `focusa_tree_head` | `lineage:read` | read | `implemented-now` |
| `focusa_tree_path` | `lineage:read` | read | `implemented-now` |
| `focusa_tree_snapshot_state` | `state:write` | mutate | `planned-extension` |
| `focusa_tree_restore_state` | `state:write` | mutate | `planned-extension` |
| `focusa_tree_diff_context` | `lineage:read` | interpret/read | `planned-extension` |
| `focusa_metacog_capture` | `metacognition:write` | mutate | `planned-extension` |
| `focusa_metacog_retrieve` | `metacognition:read` | read | `planned-extension` |
| `focusa_metacog_reflect` | `metacognition:write` | interpret | `planned-extension` |
| `focusa_metacog_plan_adjust` | `metacognition:write` | mutate | `planned-extension` |
| `focusa_metacog_evaluate_outcome` | `metacognition:write` | interpret | `planned-extension` |

## Validation gates for decomposition/closure

- FAIL if any tool contract row lacks either (a) explicit permission or (b) explicit error codes.
- FAIL if any `implemented-now` claim lacks code citation and verification test evidence.
- FAIL if metacognition contract closure is claimed solely via `/v1/reflect/*` adjacency.

## Evidence citations

- docs/80-pi-tree-li-metacognition-tooling-spec.md (§14, §15, §19.3)
- docs/evidence/SPEC80_TREE_LINEAGE_TOOL_CONTRACTS_2026-04-21.md
- docs/evidence/SPEC80_METACOG_TOOL_CONTRACTS_2026-04-21.md
- docs/evidence/SPEC80_ENDPOINT_FALLBACK_BINDING_MATRIX_2026-04-21.md
