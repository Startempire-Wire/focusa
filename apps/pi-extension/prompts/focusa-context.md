---
description: Start work with canonical Focusa project, Trajectory, Workpoint, and Context Cognition context
---

Before starting: load Focusa context through the current canonical route. Treat transcript tail and stale focus-stack dumps as advisory only.

1. Resolve project scope first:
   - `focusa_project_identity` for the active project root.
   - `focusa_project_verify` when an expected `project_root`, `project_id`, `canonical_name`, repo remote, or remote host is known.
2. Resume immediate work authority:
   - `focusa_workpoint_resume` with matching `project_root` + `continuity_id`.
   - Continue only when the Workpoint is `canonical=true` for the verified scope.
   - If no canonical Workpoint exists, create one with `focusa_workpoint_checkpoint` before risky changes.
3. Load north-star route context:
   - `focusa_trajectory_view` for HLT/MLG/STG, active gap, and next Workpoint candidate.
   - Trajectory is advisory; Workpoint remains immediate action authority.
4. Load bounded context cognition when planning or selecting files:
   - `focusa_context_cognition_render` for a compact Context Cognition card.
   - `focusa_context_cognition_curate` for token-budgeted file/evidence selection.
5. Preserve proof and learning:
   - Use `focusa_evidence_capture` / `focusa_workpoint_link_evidence` after tests, browser diagnostics, or endpoint proof.
   - Use predictions/metacognition only as advisory learning; never override operator steering.
6. If Focusa is degraded, stale, or scope-conflicted:
   - Run `focusa_tool_doctor`.
   - Re-verify `project_root + continuity_id` before trusting any packet.

Canonical vocabulary: ProjectIdentity, Trajectory (HLT/MLG/STG), Workpoint resume/checkpoint, evidence refs, Context Authority, Context Cognition.

Avoid stale startup guidance: do not begin with raw curl-only `/v1/focus/stack` or `/v1/ascc/state` dumps unless a troubleshooting task explicitly asks for legacy endpoint inspection.

$ARGUMENTS
