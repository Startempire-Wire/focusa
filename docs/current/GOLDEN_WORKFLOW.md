# Golden Workflow

Status: current canonical happy path
Sources: Spec 106 vocabulary, Specs 143/158/164 project journey, and issue #618 continuous closed-loop requirements

The Golden Workflow is a projection of [one cohesive agent project journey](../agent/02-focusa-cohesive-project-flow.md), not a separate linear script. Preserve the accepted destination, reuse valid state, and validate evidence-backed route corrections. **Verify binding → Bootstrap/Genesis when needed → linked Ladder/spec/tasks → Workpoint/frontier → Prepare/Act/Reconcile/Advance.**

## Authority boundary

See [`AUTHORITY_MODEL.md`](AUTHORITY_MODEL.md). Operator steering wins. `project_root + continuity_id` are lookup/lineage inputs; exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey establish the canonical binding. Context Cognition, Project Card, Prediction, Metacognition, and Call Stack Design are advisory unless explicitly linked through Workpoint/Trajectory/Evidence paths.

## Conditional capability crosswalk

Select only capabilities needed by the current valid frontier, recovery or relevant proof obligation; do not execute every numbered item. Bootstrap/Genesis status is inspected for missing/interrupted initialization, not repeated for every session. Context, design, prediction and transfer remain supporting operations, not compulsory ritual. Admission, applicable proof and authority requirements are unchanged.

1. **Verify ProjectIdentity**
   - Tool/API: `focusa_project_identity`, `focusa_project_verify`
   - Output: verified project scope, canonical project name/id, repo/deploy facts when available
   - Authority: project boundary / scope authority

2. **Load or define HLT / Trajectory Hierarchy**
   - Tool/API: `focusa_trajectory_view`, `focusa_trajectory_define_goal`, `focusa_hlt_history`
   - Output: HLT, MLG, STG, Waypoints, current state, active gap
   - Authority: trajectory orientation within exact scope

3. **Create or resume Workpoint**
   - Tool/API: `focusa_workpoint_resume`, `focusa_workpoint_checkpoint`
   - Output: canonical immediate continuation contract
   - Authority: immediate continuation authority when canonical and exact-scoped

4. **Generate Context Cognition packet**
   - Tool/API: `focusa_context_cognition`, `focusa_context_cognition_render`, `focusa_context_cognition_curate`
   - Output: bounded selected/excluded context, scope, freshness, evidence, route frame
   - Authority: advisory context only

5. **Create Call Stack Design**
   - Tool/API: `focusa_call_stack_design`
   - Output: typed blueprint: `entry → handlers → services → adapters → storage → output`
   - Authority: advisory/evidence-linkable implementation blueprint

6. **Run implementation**
   - Tool/API: agent/harness tools, code edits, tests
   - Guard: Context Authority preflight before risky mutation
   - Authority: operator steering + Workpoint + Context Authority verdict

7. **Capture Evidence Refs**
   - Tool/API: `focusa_evidence_capture`
   - Output: stable proof handles/refs
   - Authority: proof authority

8. **Link evidence to Workpoint**
   - Tool/API: `focusa_workpoint_link_evidence`
   - Output: Workpoint verification record
   - Authority: scoped Workpoint proof linkage

9. **Evaluate prediction/metacog outcomes**
   - Tool/API: `focusa_predict_record`, `focusa_predict_evaluate`, `focusa_metacog_capture`, `focusa_metacog_retrieve`
   - Output: calibrated forecast/learning signals
   - Authority: advisory until evaluated/promoted

10. **Save session transfer**
    - Tool/API: `focusa_session_transfer`
    - Output: save/continue packet
    - Authority: continuation support, not a replacement for Workpoint

11. **Resume after compaction/model switch**
    - Tool/API: `focusa_workpoint_resume`, `focusa_trajectory_resume`, `focusa_project_identity`
    - Output: rehydrated scope, trajectory, Workpoint
    - Authority: canonical only after exact-scope verification

12. **Reconcile, advance or settle with proof**
    - Tool/API: evidence refs, Workpoint result, trajectory assessment
    - Output: verified observed state/gap, acceptance disposition, and next dependency-valid action; a concise report does not stop unfinished admitted work
    - Authority: existing Workpoint/Trajectory/task/graph owners; whole-outcome settlement requires all accepted obligations and relevant effects reconciled

New discoveries return only affected work to Prepare/Refine. Missing frontier or stale binding requires supported recovery, not automatic mission surrender or a shell bypass. Snapshot restore/rollback/transfer remain conditional. Preserve real scope, consent, integrity and budget boundaries.

## Display requirements

Every implementation of the Golden Workflow should show:

- ProjectIdentity
- Continuity ID
- HLT / MLG / STG / Waypoints
- Workpoint
- Context Cognition advisory status
- Context Authority verdict before risky mutation
- Evidence refs
- `canonical/advisory/degraded/blocked/stale` posture
- Next safe tool/action

## Non-goals

- The Golden Workflow does not remove canonical vocabulary.
- The Golden Workflow does not make advisory surfaces canonical.
- The Golden Workflow does not permit cross-project/session merging.
- The Golden Workflow does not replace operator steering.
