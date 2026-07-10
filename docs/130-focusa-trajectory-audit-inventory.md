# Focusa Trajectory Audit Inventory

**Work item:** `focusa-h7rro.1`  
**Purpose:** inventory current Trajectory surfaces and data before judging behavior.  
**Source files:** `crates/focusa-api/src/routes/trajectory.rs`, `crates/focusa-cli/src/commands/trajectory.rs`, `crates/focusa-core/src/types.rs`, `crates/focusa-api/src/routes/workpoint.rs`, `crates/focusa-api/src/routes/focus.rs`.

## 1. Public Trajectory surfaces

| Surface | Route / CLI | Method | What it does |
|---|---|---:|---|
| View | `GET /v1/trajectory/view`; `focusa trajectory view` | read | Builds the per-project `Trajectory Intelligence view` from project scope, persisted trajectory, active Workpoint, Focus frame, Focus State, HLT history, and evidence refs. |
| Define goal | `POST /v1/trajectory/define-goal`; `focusa trajectory define-goal` | write | Validates long-term goal and desired end state, records a trajectory candidate, emits lifecycle metadata, and refreshes the view. |
| Assess | `POST /v1/trajectory/assess`; `focusa trajectory assess` | write/read | Records observed state/evidence refs as a state delta and returns the refreshed gap/next-action view. |
| Propose Workpoint | `POST /v1/trajectory/propose-workpoint`; `focusa trajectory propose-workpoint` | advisory write | Builds an advisory Workpoint candidate from the current active gap, trajectory id, target ref, and action type. |
| Checkpoint | `POST /v1/trajectory/checkpoint`; `focusa trajectory checkpoint` | write | Persists a trajectory checkpoint packet for compaction/resume continuity. |
| Resume | `POST /v1/trajectory/resume`; `focusa trajectory resume` | read | Renders a compact trajectory resume packet after compaction/model switch/session resume. |

## 2. Request shapes

| Request | Important fields | What it does |
|---|---|---|
| `TrajectoryDefineGoalRequest` | `long_term_goal`, `desired_end_state`, optional MLG/STG/waypoints/current state/source/supersession/evidence/checks/project/session/continuity | Declares or supersedes the project goal and supplies enough desired/current state to judge gaps. |
| `TrajectoryAssessRequest` | `observed_state`, `evidence_refs`, project/session/continuity | Updates current-state evidence and asks Trajectory to compare it with desired end state. |
| `TrajectoryProposeWorkpointRequest` | `trajectory_id`, `target_ref`, `action_type`, project/session/continuity | Converts active gap into an advisory Workpoint candidate. |
| `TrajectoryCheckpointRequest` | `summary`, project/session/continuity/idempotency | Saves current Trajectory orientation as a checkpoint packet. |
| `TrajectoryResumeRequest` | `mode`, project/session/continuity | Renders saved Trajectory orientation for continuation. |
| `TrajectoryViewQuery` | `project_root`, `session_id`, `continuity_id`, `mode`, `allow_prior_project_trajectory` | Selects scope and read-mode for the intelligence view. |

## 3. Internal data structures

### 3.1 `TrajectoryProjectionRecord`

| Field/group | What it does |
|---|---|
| Identity: `trajectory_id`, `session_identity`, `project_root`, `continuity_id` | Binds the trajectory to project and logical session authority. |
| Goal ladder: `root_long_term_goal`, `long_term_goal`, `desired_end_state`, `mid_level_goal`, `short_term_goal`, `waypoints`, `current_state` | Stores desired direction, current/desired state, and intermediate route markers. |
| Stability/status: `root_goal_stability`, `session_clarity_status`, `definition_status`, `confidence`, `canonical` | Reports whether the goal is stable, clear, provisional, conflicted, or canonical. |
| Gap/lifecycle: `gap_summary`, `milestones`, `active_milestone_id`, `active_workpoint_id` | Tracks current gap, milestone progress, and Workpoint alignment. |
| Evidence/source: `source_refs`, `goal_provenance`, `definition_of_done` | Records where goal wording came from and what proof/checks are required for done. |
| Drift/support: `blockers`, `open_questions`, `supersedes_trajectory_id`, timestamps | Captures missing/conflicting info and supersession lineage. |

### 3.2 `WorkpointRecord`

| Field/group | What it does |
|---|---|
| Identity/scope: `workpoint_id`, `work_item_id`, `session_identity`, `continuity_id`, `session_id`, `project_root`, `frame_id` | Binds immediate work to project/session/frame/bead authority. |
| Status/canonicality: `status`, `checkpoint_reason`, `confidence`, `canonical`, `rejection_reason` | Says whether the Workpoint is active/canonical or rejected/degraded. |
| Mission/action: `mission`, `active_object_refs`, `action_intent`, `next_slice` | Carries the current objective, target refs, action type, verification hooks, and next bounded step. |
| Proof/blockers: `verification_records`, `blockers` | Stores evidence refs/results and known reasons work cannot proceed. |
| Lineage/replay: `source_turn_id`, `idempotency_key`, `supersedes`, timestamps | Supports safe replay, handoff, and supersession. |

### 3.3 `FrameRecord`

| Field/group | What it does |
|---|---|
| Stack identity: `id`, `parent_id`, `status`, timestamps | Places a focus frame in the active Focus Stack. |
| Goal/linkage: `title`, `goal`, `beads_issue_id`, `project_root`, `continuity_id`, `tags` | Describes focused work and links it to bead/project scope. |
| Execution metadata: `priority_hint`, `ascc_checkpoint_id`, `stats`, `constraints` | Adds scheduling hints, checkpoints, counters, and boundaries. |
| Focus State: `focus_state` | Stores decisions, constraints, failures, current focus, next steps, questions, results, and notes for the frame. |
| Completion: `completed_at`, `completion_reason` | Records why and when a frame left active work. |

## 4. intelligence_view fields

| Field | What it does |
|---|---|
| `context_sufficiency.score` | Numeric projection of whether enough facts exist to proceed. |
| `context_sufficiency.status` | Mirrors trajectory definition status (`clear`, `provisional`, `unclear`, `conflicted`). |
| `context_sufficiency.proceed_posture` | Reduces clarity recommendation to `proceed`, `verify_first`, or `operator_required`. |
| `context_sufficiency.missing_facts` | Names absent inputs like goal, desired state, verified state, or Workpoint. |
| `context_sufficiency.stale_refs` | Placeholder list for stale proof/context refs; currently empty. |
| `context_sufficiency.conflicting_signals` | Human-readable scope mismatches from query/workpoint/project identity. |
| `context_sufficiency.recommended_action` | Action from clarity gate (`proceed`, `verify_first`, `operator_input`). |
| `similarity_group` | Advisory grouping keys for high/mid/low-level trajectory similarity; not authority. |
| `clarity_gate` | Gate payload deciding clear/provisional/unclear/conflicted and why. |
| `relevance_rationale` | Explains why project identity, Workpoint, Focus frame, or evidence were included. |
| `current_state_delta` | Recent trajectory state-delta refs and evidence since last checkpoint. |
| `trajectory_workpoint_reconciliation` | Compares Workpoint and Trajectory status and says which surface controls next action. |
| `focus_trajectory_sync` | Projection-only bridge between Focus State current focus and Trajectory STG. |
| `learning_refs` | Placeholder list for relevant learning refs; currently empty. |
| `prediction_refs` | Placeholder list for relevant prediction refs; currently empty. |
| `ask_operator_if` | Operator questions needed for unclear/conflicted scope or goal. |
| `do_not_use` | Surfaces or assumptions agents must avoid in the current trajectory state. |
| `next_workpoint_candidate` | Advisory active Workpoint candidate when a Workpoint exists. |
| `tool_affordances` | Suggested next tools: view, resume Workpoint, resolve objects, capture evidence. |
| `recent_results` | Bounded Focus State recent results copied into trajectory view. |
| `decisions` | Bounded Focus State decisions copied into trajectory view. |
| `constraints` | Bounded Focus State constraints copied into trajectory view. |

## 5. Active gap / gap reasons

| Gap reason/value | When it appears | Meaning |
|---|---|---|
| `None` | Desired end state equals current state. | Trajectory sees no active state gap. |
| Workpoint next slice or action | Desired/current both exist and differ, and Workpoint has a next/action. | The Workpoint provides the concrete immediate gap text. |
| `Current verified state differs from desired end state` | Desired/current differ but Workpoint has no next/action. | A real gap exists, but it lacks a concrete Workpoint-shaped next step. |
| `Trajectory gap unclear until desired end state and current verified state are both present` | Desired or current verified state is missing. | The view cannot compare end state to current state. |
| `Trajectory definition required before ladder projection` | HLT is a bootstrap degraded placeholder. | A real operator-defined goal is missing; ladder projection is not trustworthy. |
| `agent_runtime_directory` | Project root is classified as an agent/runtime directory. | Scope is unsafe; use an actual project root. |
| `missing_verified_state` | Define-goal state gate lacks current ask, current state, or supersession evidence. | Goal text may exist, but current state is not verified enough for canonical completion. |

## 6. `clarity_gate.blocking_reasons`

| Blocking reason | What it means |
|---|---|
| `conflicting_project_or_continuity_scope` | Query/project/workpoint identity signals disagree or project identity is mismatch. |
| `project_identity` | Project root/identity is not bound and matching. |
| `long_term_goal` | A durable long-term goal is missing or only bootstrap placeholder exists. |
| `desired_end_state` | Desired end state is missing, so done/gap cannot be judged. |
| `current_verified_state` | Current state is missing, so the active gap cannot be verified. |
| `next_workpoint` | No active Workpoint is present to supply immediate execution context. |
| `stale_or_missing_evidence_refs` | Trajectory is not unclear, but no evidence refs support current/desired state claims. |
| `agent_runtime_directory` | Scope points at agent/runtime infrastructure instead of a project folder. |

## 7. Current inventory observations

- Trajectory already has the six public surfaces needed for goal definition, view, assessment, proposal, checkpoint, and resume.
- The view is rich, but several fields are placeholders or projections (`learning_refs`, `prediction_refs`, `stale_refs`, generic `tool_affordances`).
- The active gap can fall back to generic text when no Workpoint next/action exists.
- The clarity gate has useful blocking reasons, but some are broad facts rather than sharp recovery instructions.
- `trajectory_workpoint_reconciliation` correctly says Workpoint controls immediate next action when canonical.
- Phase 2 should judge which of these fields are actionable versus merely descriptive.
