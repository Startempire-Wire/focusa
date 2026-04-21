# SPEC80 Endpoint + Fallback Binding Matrix (Operationalized) — 2026-04-21

Purpose: implement `focusa-yro7.2.3` by operationalizing Spec80 Appendix B (§15) into a rollout-oriented binding matrix with code-reality status and dependency order.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§15, §19.3)

## Binding matrix (operational)

| Tool | Primary API | API code-reality | CLI fallback | CLI code-reality | Permission | Label (§19.3) |
|---|---|---|---|---|---|---|
| `focusa_tree_head` | `GET /v1/lineage/head` | implemented | `focusa lineage head --json` | implemented | `lineage:read` | `implemented-now` |
| `focusa_tree_path` | `GET /v1/lineage/path/{clt_node_id}` | implemented | `focusa lineage path <id> --json` | implemented | `lineage:read` | `implemented-now` |
| `focusa_tree_snapshot_state` | `POST /v1/focus/snapshots` | planned | `focusa state snapshot create --json` | planned | `state:write` | `planned-extension` |
| `focusa_tree_restore_state` | `POST /v1/focus/snapshots/restore` | planned | `focusa state snapshot restore --json` | planned | `state:write` | `planned-extension` |
| `focusa_tree_diff_context` | `POST /v1/focus/snapshots/diff` | planned | `focusa state snapshot diff --json` | planned | `lineage:read` | `planned-extension` |
| `focusa_metacog_capture` | `POST /v1/metacognition/capture` | planned | `focusa metacognition capture --json` | planned | `metacognition:write` | `planned-extension` |
| `focusa_metacog_retrieve` | `POST /v1/metacognition/retrieve` | planned | `focusa metacognition retrieve --json` | planned | `metacognition:read` | `planned-extension` |
| `focusa_metacog_reflect` | `POST /v1/metacognition/reflect` | planned (`/v1/reflect/*` adjacent only) | `focusa metacognition reflect --json` | planned (`focusa reflect ...` adjacent only) | `metacognition:write` | `planned-extension` |
| `focusa_metacog_plan_adjust` | `POST /v1/metacognition/adjust` | planned | `focusa metacognition adjust --json` | planned | `metacognition:write` | `planned-extension` |
| `focusa_metacog_evaluate_outcome` | `POST /v1/metacognition/evaluate` | planned | `focusa metacognition evaluate --json` | planned | `metacognition:write` | `planned-extension` |

## Dependency ordering (rollout sequence)

1. **Read substrate first (already implemented)**
   - `lineage/head`, `lineage/path` API + `focusa lineage` CLI fallback.
2. **Branch-state write substrate**
   - `/v1/focus/snapshots*` APIs before snapshot CLI fallback commands.
3. **Metacognition API substrate**
   - `/v1/metacognition/*` APIs before `focusa metacognition ...` CLI domain.
4. **Outcome-loop closure wiring**
   - connect capture → retrieve → reflect → adjust → evaluate after both substrates are available.

## Blocking dependencies

- Snapshot/restore/diff tools are blocked by missing `/v1/focus/snapshots*` routes.
- Metacognitive toolchain is blocked by missing `/v1/metacognition/*` routes.
- Existing `/v1/reflect/*` and `focusa reflect ...` do not satisfy planned metacognition contract parity.

## Evidence citations

- docs/80-pi-tree-li-metacognition-tooling-spec.md (§15, §19.3)
- crates/focusa-api/src/routes/capabilities.rs
- crates/focusa-api/src/routes/reflection.rs
- crates/focusa-cli/src/commands/lineage.rs
- crates/focusa-cli/src/main.rs
