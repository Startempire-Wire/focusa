# Workpoint Project Folder + Continuity Guard

Focusa separates identity axes instead of collapsing everything into a single active session.

## Identity axes

- `project_root` — project folder/container holding related files; broad roots such as `/`, `/root`, `/home`, `/tmp`, `/var`, `/usr`, and `/opt` are unsafe authority roots.
- `continuity_id` — stable logical session/workstream identity; distinct same-root sessions receive distinct IDs.
- `session_id` — temporal Pi/process metadata across compaction, model switch, fork, and restart.
- `trajectory_id`, goals, work-item IDs, frame tags — corroborating alignment signals only.
- `workpoint_id` — checkpoint/action packet identity within a continuity stream.

## Current behavior

- Workpoint checkpoints and resumes carry `project_root`, `continuity_id`, and `session_id`.
- Focus frames carry `project_root`, `continuity_id`, and continuity tags so same-root sessions can remain active without cross-pausing.
- `/v1/workpoint/resume` rejects cross-project packets with `status: rejected_scope_mismatch`.
- `/v1/workpoint/resume` rejects unsafe broad-root packets with `status: rejected_unsafe_project_root`; compaction and Focus Slice omit them.
- Canonical Workpoint checkpoints require a safe project folder `project_root` and `continuity_id`; `/root` is never a canonical project folder.
- Focus frame reads/pushes reject unsafe broad-root scopes; scoped `project_root` queries never fall back to global active frames.
- Pi remembers the last verified safe project folder in a durable per-user cache (`~/.pi/agent/focusa-project-root.json`) so a later Pi session launched from a broad cwd can rebind without asking for `project_root` again.
- Focus State write recovery may adopt the current safe active Workpoint scope before creating a frame; it must not create `/root` frames.
- `/v1/workpoint/resume` rejects same-project/different-continuity packets with `status: rejected_continuity_mismatch`.
- Same-project/same-continuity post-compaction `session_id` changes are recorded as `session_continuity` metadata.
- **Model-switch project preservation (emergency fix 4, 2026-05-27):** After `model_select` fires `checkpointDiscontinuity`, if the session already has a verified `lastProjectIdentity` (confidence: high|medium), the subsequent `focusa_project_identity` tool must NOT overwrite it with a different project's identity. The Pi extension preserves the existing verified identity and returns it instead. This prevents cross-project overwrite during model-switch bootstrap when the daemon or project_root_cache returns a different project's root.
- `identity_confidence_percent` explains corroborating alignment; it never overrides hard gate failures.

## Canonical saved scope vs current-action authority

A Workpoint packet can be canonical for its saved `project_root + continuity_id` and still be the wrong action anchor for the latest operator ask. Canonicality answers "is this a valid stored packet for this logical workstream?" It does not answer "should the next tool/file action use this project now?"

Required anti-forgetting fields for Workpoint resume/rendering:

- `canonical_for_saved_scope` — saved Workpoint identity gates passed.
- `matches_current_ask_scope` — latest operator ask does not name, negate, or imply a different project/root/remote.
- `action_authority_for_current_ask` — file/API/tool action may proceed under this Workpoint.
- `scope_conflict_reason` — bounded evidence when action authority is suppressed.

Current scope guards reject unsafe or mismatched packets at resume/checkpoint boundaries. Pi now also emits `CURRENT_ASK_SCOPE_VERDICT` before Workpoint instructions in Focus Slice/compaction output; when it detects a semantic project conflict, it preserves the old packet as canonical for saved scope, sets `action_authority_for_current_ask=false`, emits `scope_conflict_detected` telemetry, and routes to `focusa_project_verify` / `focusa_project_identity` / `focusa_workpoint_checkpoint` before file or API action.

The semantic conflict guard is supported by the Spec97 `detect_semantic_project_scope_conflict` primitive. That primitive outputs `CurrentScopeVerdict` metadata and remains read-only/advisory; Workpoint/project identity gates and operator steering retain authority.

## Scenario matrix

| Scenario | Expected isolation result |
|---|---|
| Same project, different continuity IDs, similar long-term goal | Distinct sessions; no cross-contamination. |
| Same project, same long-term goal, different short-term goals | Distinct sessions unless continuity_id matches. |
| Same project, different long/short goals | Distinct sessions by continuity_id. |
| Different projects | Distinct by project_root before continuity is considered. |
| Operator declares a different project after a canonical packet | Saved packet remains canonical for its old scope, but current-action authority is suppressed until project verify/rebind. |
| Broad/root home scopes like `/root` | Not canonical; packet is quarantined and latest operator instruction wins. |
| Same project + same continuity + changed session_id after compaction | Same logical session; temporal session drift only. |
| Model switch / new model after session has verified project identity | Same logical session; preserve existing `lastProjectIdentity` even if `focusa_project_identity` returns a different project. Do not overwrite with project_root_cache or re-bootstrap identity. |

## Recovery

For `project_root` mismatch or unsafe broad-root project folder, follow the current project and checkpoint a new project-bound Workpoint from the exact repo/project root. For `continuity_id` mismatch, list/reopen the matching SilentSession or checkpoint a fresh Workpoint with the current continuity_id.

## Tests

```bash
cargo test -p focusa-core reducer::tests::same_project_distinct_continuity_frames_remain_active_without_cross_pause
cargo test -p focusa-api workpoint
tests/spec96_workpoint_post_compaction_resume_static_test.sh
tests/spec_project_scope_override_static_test.sh
tests/spec_scope_arbitration_block_static_test.sh
tests/spec97_semantic_scope_conflict_primitive_static_test.sh
tests/scope_routing_regression_eval.sh
```
