# SPEC80 Tree/Lineage Bridge Tool Contracts — 2026-04-21

Purpose: implement `focusa-yro7.2.1` by extracting a machine-reviewable contract pack for SPEC80 tree/lineage bridge tools from §6.1 and Appendix A §14.1, with binding references to Appendix B §15.

## Contract baseline

Authority source:
- `docs/80-pi-tree-li-metacognition-tooling-spec.md` §6.1, §14.1, §15

Mandatory contract invariants:
1. Stable JSON input/output surface per tool.
2. Explicit typed error-code envelope.
3. Declared ontology layer coverage per tool.
4. Binding path to API (`/v1/*`) and CLI parity target where applicable.

## Tree/lineage bridge tool contract set

| Tool | Contract intent | Required input | Success output (shape) | Required error codes | Layers | Label (§19.3) |
|---|---|---|---|---|---|---|
| `focusa_tree_head` | Resolve active tree/CLT head mapping | `session_id?` | `{ session_id, pi_tree_node, clt_head, branch_id }` | `TREE_HEAD_UNAVAILABLE`, `SESSION_NOT_FOUND` | `8,12` | `documented-authority` |
| `focusa_tree_path` | Provide branch-aware lineage path | `clt_node_id?`, `to_root` | `{ head, path:[...], branch_point, depth }` | `CLT_NODE_NOT_FOUND` | `8,7,12` | `documented-authority` |
| `focusa_tree_snapshot_state` | Snapshot Focus State at CLT node | `clt_node_id`, `snapshot_reason` | `{ snapshot_id, clt_node_id, created_at, checksum }` | `SNAPSHOT_WRITE_DENIED`, `SNAPSHOT_CONFLICT` | `8,5,11` | `planned-extension` |
| `focusa_tree_restore_state` | Restore branch-correct Focus State | `clt_node_id`, `restore_mode` | `{ restored, snapshot_id, clt_node_id, conflicts:[...] }` | `SNAPSHOT_NOT_FOUND`, `RESTORE_CONFLICT`, `AUTHORITY_DENIED` | `8,5,11,9` | `planned-extension` |
| `focusa_tree_diff_context` | Diff context state across branches | `left_clt_node_id`, `right_clt_node_id`, `include` | `{ decision_diff, constraint_diff, failure_diff, risk_diff }` | `CLT_NODE_NOT_FOUND`, `DIFF_SCOPE_INVALID` | `8,11,12` | `planned-extension` |

## Current code-reality binding notes

Implemented-now API lineage substrate (read surface):
- `/v1/lineage/head`
- `/v1/lineage/tree`
- `/v1/lineage/node/{clt_node_id}`
- `/v1/lineage/path/{clt_node_id}`
- `/v1/lineage/children/{clt_node_id}`
- `/v1/lineage/summaries`

CLI parity domain now present for lineage read surface:
- `focusa lineage head|tree|node|path|children|summaries`

Planned-extension gap remains for snapshot/restore/diff write+compare operations:
- `/v1/focus/snapshots*` family not yet implemented in code reality checkpoints.

## Evidence citations

- docs/80-pi-tree-li-metacognition-tooling-spec.md (§6.1, §14.1, §15, §19.3)
- crates/focusa-api/src/routes/capabilities.rs
- crates/focusa-cli/src/commands/lineage.rs
- crates/focusa-cli/src/main.rs
