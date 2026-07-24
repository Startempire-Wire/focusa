# Friendly Focusa Onboarding Q

The **Friendly Focusa Q** should feel like navigation help, not a nag. The goal is a short orientation that helps the model choose the right tool route before it falls back to only `focusa_scratch` / `focusa_decide`.

## Tone contract

- Use **suggested route**, **quick orientation**, and **next useful move** language.
- Avoid scoldy startup banners like “REQUIRED FIRST” unless an operation would be unsafe.
- Ask internally first; only ask the operator when the answer cannot be inferred safely.
- Keep operator steering above Focusa guidance.

## The friendly Focusa Q

1. **Where am I?**
   Project folder/container (`project_root`) and continuity identity.
   Tools: `focusa_project_identity`, `focusa_project_verify`, then scoped Workpoint calls.

2. **What kind of project is this?**
   Canonical name, repo, workspace kind, service/deploy boundaries, and architecture assumptions.
   Tools: `focusa_project_identity`, `focusa_traverse`, docs/evidence lookup before architectural claims.

3. **Where are we going?**
   Current state, destination, waypoints, goal gaps.
   Tools: `focusa_trajectory_view`, `focusa_trajectory_define_goal`, `focusa_trajectory_assess`, `focusa_trajectory_propose_workpoint`.

4. **What is the next useful move?**
   Mission, current action, active object, next anchor.
   Tools: `focusa_workpoint_resume`, `focusa_active_object_resolve`, `focusa_workpoint_checkpoint`.

5. **What proof changes confidence?**
   Test/API/file/release evidence that proves state changed.
   Tools: `focusa_evidence_capture`, `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`.

6. **What might go wrong?**
   Risk forecast before risky edits, releases, recovery, or uncertain next action.
   Tools: `focusa_predict_record`, then `focusa_predict_evaluate` after outcome.

7. **What should compound for the next agent?**
   Reusable lesson, retrieved prior lessons, adjustment outcome.
   Tools: `focusa_metacog_retrieve`, `focusa_metacog_capture`, `focusa_metacog_reflect`, `focusa_metacog_plan_adjust`, `focusa_metacog_evaluate_outcome`.

8. **Is context too big or stale?**
   Surgical state lookup and recovery instead of transcript guessing.
   Tools: `focusa_traverse`, tree/snapshot tools, `focusa_tool_doctor`, `focusa_resource_mode`.

9. **Is work continuous or delegated?**
   Writer ownership, preflight, checkpoint, next ready work, background sessions.
   Tools: `focusa_work_loop_writer_status`, `focusa_work_loop_status`, `focusa_work_loop_context`, `focusa_work_loop_checkpoint`, `focusa_work_loop_select_next`, `focusa_silent_sessions`.

## Anti-pattern this fixes

Bad route:

```text
scratch note → decide note → continue from transcript memory
```

Better route:

```text
project_identity → trajectory_view → workpoint_resume/checkpoint → evidence → prediction/metacog → Focus State summary
```

`focusa_scratch`, `focusa_decide`, `focusa_constraint`, `focusa_failure`, and sibling Focus State tools are still useful. They are slots in the route, not the route itself.

## Model-visible fallback

The context hook now injects a compact Project/Trajectory/Architecture fallback card even when the scoped Focus frame is missing or trajectory lookup is degraded. The architecture digest includes confidence and evidence refs from manifests/docs/tests/service files. This keeps the friendly Q helpful without blocking work. Unsafe broad roots still withhold architecture facts until project identity is verified.

## Minimal friendly startup copy

```text
Quick Focusa Q: where am I, what kind of project/architecture is this, where are we going, what is the next useful move, what proof matters, and what should future agents reuse?
Suggested route: project_identity → trajectory_view → workpoint_resume/checkpoint → evidence → prediction/metacog. Operator steering wins.
```

## Current first-agent walkthrough

1. Read `AGENTS.md`, then this bounded index: `docs/agent/01-focusa-agent-docs-index.md`.
2. Call `focusa_agent_card`; confirm its version, registry digest, Pi tool count, complete skill inventory, and runbook count.
3. Verify `project_root + continuity_id` with `focusa_project_identity` and `focusa_project_verify`. Treat worktrees as typed working subpaths.
4. Resume `focusa_trajectory_view` and `focusa_workpoint_resume`; checkpoint when no canonical Workpoint exists.
5. Use `focusa_tool_search` → `focusa_tool_describe` for the narrowest of all Focusa Pi tools. Do not hot-load or invent schemas.
6. Load the matching `.pi/skills/<skill>/SKILL.md`, then its numbered runbook only for the selected workflow.
7. For Mission Canvas, use the Work Rail/Work Surface bindings and UIAI session/origin boundaries; do not create a parallel hand-coded authority path.
8. For autonomous background work, use daemon-native Silent Sessions with exact session/run/generation and mutation approval/idempotency.
9. Before compaction or model/session change, checkpoint Workpoint and Trajectory; governed rollover is the recovery path after bounded transport exhaustion.
10. Close work with stable Evidence, prediction evaluation when available, reusable metacognition when evidence-backed, and an exact next action.

## Customer lifecycle walkthrough

```bash
# Inspect without mutation.
bash scripts/install-focusa.sh --dry-run --eval

# Install or idempotently repair/rerun.
curl -fsS https://install.focusa.dev/focusa | bash -s -- --eval

# Discover trusted updater and rollback controls.
focusa update --help

# Remove managed binaries/integration while preserving user data by default.
curl -fsS https://install.focusa.dev/focusa | bash -s -- --uninstall
```

Use `--purge-data` only for explicit destructive removal. Verify daemon health, version, Pi tool discovery, Mission Canvas, and Workpoint resume after install/update/repair.
