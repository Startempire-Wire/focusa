# Strong Trajectory Spec

**Work item:** `focusa-h7rro.3`  
**Purpose:** define the target behavior for making Trajectory a strong core feature instead of a loose advisory projection.  
**Inputs:** `docs/130-focusa-trajectory-audit-inventory.md`, `docs/131-focusa-trajectory-audit-flailing-patterns.md`.

## 1. Definition

A strong Trajectory is a project-scoped mission contract that answers four questions with a sharp opinion:

1. **Where are we going?** A durable long-term goal and desired end state.
2. **Where are we now?** A verified current state with evidence or a clear missing-proof block.
3. **What is the gap?** A concrete gap description with target object, next tool, command shape, and proof needed.
4. **What may the agent do next?** Exactly one recommended next action, or a block with a repair command.

Trajectory remains advisory for execution authority: a canonical Workpoint still controls immediate action. Strong Trajectory decides whether the route is clear enough to create/resume/checkpoint a Workpoint.

## 2. Good Trajectory shape

A good Trajectory view must contain:

- `long_term_goal`: specific project outcome, not vague maintenance language.
- `desired_end_state`: externally checkable state.
- `current_state`: verified or explicitly missing.
- `active_gap`: concrete delta between current and desired state.
- `gap_description`: bounded sentence with verb + target + reason.
- `next_tool`: one Focusa tool/CLI route to call next.
- `next_command`: public-safe command shape or API route body.
- `proof_needed`: evidence/check required before calling done.
- `clarity_gate`: clear/provisional/blocked status with exact blocking reasons.
- `workpoint_relation`: whether Workpoint is canonical, missing, stale, or should be created.

## 3. Invariants

1. **No gap without a next call.** Every non-empty active gap must include `next_tool` and `next_command`.
2. **No done without proof.** Desired equals current is not canonical unless required evidence/checks exist or a Workpoint proof links it.
3. **No vague goal.** Goals like `make it better`, `continue`, `fix stuff`, or generic maintenance are rejected with a recovery hint.
4. **No hidden scope.** Project root and continuity mismatches block canonical Trajectory.
5. **No execution leap.** Trajectory may propose a Workpoint but must not imply the agent can execute without a canonical Workpoint.
6. **No placeholder authority.** Empty `learning_refs`, `prediction_refs`, or stale refs must be labeled `not_integrated` or omitted.
7. **No Workpoint laundering.** A vague Workpoint next slice cannot become a strong trajectory gap without a concrete verb, target, and proof hook.
8. **No operator ping when a tool can repair.** Missing current state should recommend `trajectory assess`; missing Workpoint should recommend `workpoint checkpoint` or `trajectory propose-workpoint` before asking the operator.

## 4. Clarity gate policy

| Status | Meaning | Required next behavior |
|---|---|---|
| `clear` | Goal/end state/current state/scope are specific and evidence-backed enough to route. | Return `next_tool` and allow Workpoint proposal/resume/checkpoint. |
| `provisional` | Goal is plausible but missing proof, current state, or Workpoint anchoring. | Return exact repair command; do not call it canonical. |
| `blocked` | Scope conflict, unsafe root, vague goal, contradictory state, or claimed done without proof. | Return `status=blocked`, `failure_class`, `recovery_hint`, and no actionable Workpoint candidate. |
| `conflicted` | Signals disagree on project/root/continuity/goal supersession. | Require verify-first route and block execution-oriented recommendations. |

## 5. What Trajectory must never let an agent do

- Treat transcript memory as trajectory truth.
- Merge sessions because high-level goals sound similar.
- Execute or mutate from a trajectory proposal without a canonical Workpoint.
- Accept a goal that cannot be externally verified.
- Mark a trajectory done because desired/current strings match without proof.
- Use unsafe broad roots or agent runtime directories as project scope.
- Hide missing evidence behind `context_sufficiency.score`.
- Return multiple equally ranked next actions when one exact repair command is possible.

## 6. Clear opinion examples

### 6.1 Well-formed mission

Input: long-term goal names the project outcome; desired state has a test/release/doc proof; current state has evidence; Workpoint exists.

Output opinion:

```json
{
  "status": "completed",
  "canonical": true,
  "clarity_gate": { "status": "clear", "recommended_action": "proceed" },
  "gap_description": "Run the release smoke proof for the scoped CLI launch hardening changes.",
  "next_tool": "focusa_workpoint_resume",
  "next_command": "focusa workpoint resume --project-root <project> --continuity-id <id> --copy-prompt",
  "proof_needed": ["test:cross_phase_smoke_e2e", "test:public_surface_guard_e2e"],
  "workpoint_relation": "canonical_workpoint_controls_immediate_action"
}
```

### 6.2 Bad mission

Input: `long_term_goal=make it better`, no desired end state proof, no current state.

Output opinion:

```json
{
  "status": "blocked",
  "failure_class": "vague_or_unverifiable_trajectory_goal",
  "clarity_gate": { "status": "blocked", "blocking_reasons": ["long_term_goal_vague", "desired_end_state_unverifiable", "current_verified_state_missing"] },
  "recovery_hint": "Define a concrete project outcome, desired end state, current state, and proof check.",
  "next_tool": "focusa_trajectory_define_goal",
  "next_command": "focusa trajectory define-goal --long-term-goal '<specific outcome>' --desired-end-state '<verifiable state>' --current-state '<observed state>' --required-check '<test or proof>'"
}
```

## 7. Implementation target for phase 4

Phase 4 should add a strong-opinion layer around existing `trajectory.rs` outputs:

- validate goal specificity and verifiability;
- produce `gap_description`, `next_tool`, `next_command`, `proof_needed`, and `workpoint_relation`;
- replace placeholder refs with populated values or explicit `not_integrated` metadata;
- sharpen clarity gate status into clear/provisional/blocked/conflicted;
- make `assess` and `propose_workpoint` return exact next call guidance.
