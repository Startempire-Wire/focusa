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

2. **Where are we going?**  
   Current state, destination, waypoints, goal gaps.  
   Tools: `focusa_trajectory_view`, `focusa_trajectory_define_goal`, `focusa_trajectory_assess`, `focusa_trajectory_propose_workpoint`.

3. **What is the next useful move?**  
   Mission, current action, active object, next anchor.  
   Tools: `focusa_workpoint_resume`, `focusa_active_object_resolve`, `focusa_workpoint_checkpoint`.

4. **What proof changes confidence?**  
   Test/API/file/release evidence that proves state changed.  
   Tools: `focusa_evidence_capture`, `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`.

5. **What might go wrong?**  
   Risk forecast before risky edits, releases, recovery, or uncertain next action.  
   Tools: `focusa_predict_record`, then `focusa_predict_evaluate` after outcome.

6. **What should compound for the next agent?**  
   Reusable lesson, retrieved prior lessons, adjustment outcome.  
   Tools: `focusa_metacog_retrieve`, `focusa_metacog_capture`, `focusa_metacog_reflect`, `focusa_metacog_plan_adjust`, `focusa_metacog_evaluate_outcome`.

7. **Is context too big or stale?**  
   Surgical state lookup and recovery instead of transcript guessing.  
   Tools: `focusa_traverse`, tree/snapshot tools, `focusa_tool_doctor`, `focusa_resource_mode`.

8. **Is work continuous or delegated?**  
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

## Minimal friendly startup copy

```text
Quick Focusa Q: where am I, where are we going, what is the next useful move, what proof matters, and what should future agents reuse?
Suggested route: project_identity → trajectory_view → workpoint_resume/checkpoint → evidence → prediction/metacog. Operator steering wins.
```
