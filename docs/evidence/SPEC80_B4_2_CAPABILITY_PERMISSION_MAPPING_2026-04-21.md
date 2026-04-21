# SPEC80 B4.2 — Capability Permission Mapping

Date: 2026-04-21
Bead: `focusa-yro7.2.4.2`
Label: `documented-authority`

Purpose: bind each SPEC80 tool to required capability scopes and denial behavior.

## Permission map

| Tool | Permission | Denial code |
|---|---|---|
| `focusa_tree_head` | `lineage:read` | `AUTHORITY_DENIED` |
| `focusa_tree_path` | `lineage:read` | `AUTHORITY_DENIED` |
| `focusa_tree_snapshot_state` | `state:write` | `SNAPSHOT_WRITE_DENIED` |
| `focusa_tree_restore_state` | `state:write` | `AUTHORITY_DENIED` |
| `focusa_tree_diff_context` | `lineage:read` | `AUTHORITY_DENIED` |
| `focusa_metacog_capture` | `metacognition:write` | `AUTHORITY_DENIED` |
| `focusa_metacog_retrieve` | `metacognition:read` | `AUTHORITY_DENIED` |
| `focusa_metacog_reflect` | `metacognition:write` | `AUTHORITY_DENIED` |
| `focusa_metacog_plan_adjust` | `metacognition:write` | `AUTHORITY_DENIED` |
| `focusa_metacog_evaluate_outcome` | `metacognition:write` | `AUTHORITY_DENIED` |

## Enforcement rules

1. Denial must be explicit and typed; no silent noop.
2. Permission checks happen before mutation.
3. CLI fallback must preserve same permission semantics and denial code family.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§15)
- docs/24-capabilities-cli.md
