# Agent Awareness Quickstart

Focusa is an agent utility layer: working memory, continuation contracts, evidence links, prediction records, recovery guidance, Spec97 reflex affordances, and governance for long-running AI sessions.

## Friendly Focusa Q

Use this as internal orientation, not a blocker:

1. **Where am I?** `project_root + continuity_id` → `focusa_project_identity` / `focusa_project_verify`.
2. **What kind of project is this?** canonical name, repo, workspace kind, infra/architecture boundaries → `focusa_project_identity` / `focusa_traverse`.
3. **Where are we going?** current state, destination, waypoints → `focusa_trajectory_view` / `focusa_trajectory_assess`.
4. **What is the next useful move?** mission, active object, next anchor → `focusa_workpoint_resume` / `focusa_workpoint_checkpoint`.
5. **What proof changes confidence?** tests/API/file handles → `focusa_evidence_capture` / `focusa_workpoint_link_evidence`.
6. **What compounds?** prediction outcome + reusable lesson → `focusa_predict_record`, `focusa_predict_evaluate`, `focusa_metacog_*`.
7. **What routine recovery applies?** bounded primitive ids from `reflex_suggestions` → `focusa_reflex_primitives` / `surface=reflex_primitives`.

## What agents must know first

1. **Focusa is not chat memory.** It stores bounded state, Workpoints, evidence refs, predictions, lineage, and recovery hints.
2. **Use the route, not only the note tools.** `focusa_scratch` / `focusa_decide` are useful slots, but project work should usually route through project identity → trajectory → Workpoint → evidence → learning.
3. **Workpoint beats transcript tail.** After compaction/reload/model switch/fork, call `focusa_workpoint_resume` and follow the canonical packet unless the operator steers otherwise.
4. **Checkpoint before risky boundaries.** Before compaction, model switch, fork, context overflow, or risky continuation, call `focusa_workpoint_checkpoint`.
5. **No deadends on tool failure.** Read `failure_class`, `retry.posture`, `recovery_hint` / `misuse_hint`, `next_tools`, and optional `reflex_suggestions`; fix ordering/scope before retrying.
6. **Doctor first when uncertain.** If Focusa seems stale/offline/blocked/degraded, call `focusa_tool_doctor` before guessing.
7. **Missing-frame fallback stays helpful.** If no active Pi frame is available, use `Attentive and awaiting operator direction`: continue from operator/repo context, then checkpoint/resume once scope is safe.
8. **Evidence is first-class.** After tests, release proof, API proof, or file proof, call `focusa_evidence_capture` or `focusa_workpoint_link_evidence`.
9. **Predictions are measurable and regular.** Before risky, uncertain, or high-leverage next action, call `focusa_predict_record` with bounded ontology context; after proof/test/CI/evidence, call `focusa_predict_evaluate` or capture outcome.
10. **Metacognition compounds regularly.** Retrieve prior lessons before similar work, and after meaningful outcomes evaluate/promote learning so it can feed the next prediction.
10. **Compaction must be useful.** Sparse Focusa slots should use related Workpoint/current-ask/frame/local-shadow/session fallbacks, never random filler or bare `none`.
11. **Identity has axes.** Project scope is `project_root`; logical session/workstream identity is `continuity_id`; Pi `session_id` is temporal metadata; trajectory/goals are corroborating evidence.
12. **Context pressure is Focusa-aware.** Focusa checkpoints and resumes scoped anchors under pressure; warnings say anchors are unconfirmed, not degraded, and `/fork`, `/new`, or handoff are optional UI-isolation paths only when anchors are unconfirmed.

## Minimal runtime loop

```text
Start/reload:      project_identity → trajectory_view → workpoint_resume if resuming.
Before boundary:  focusa_workpoint_checkpoint.
During work:      active_object_resolve → do work → evidence_capture/link.
Before risk:      focusa_predict_record.
After outcome:    focusa_predict_evaluate → metacog_capture/retrieve if reusable.
After proof:      trajectory_assess → recent_result/decision if durable.
After compaction: focusa_workpoint_resume; continue from canonical packet only when project_root and continuity_id match.
When uncertain:   focusa_tool_doctor → resource_mode/traverse/workpoint_resume.
Recurring recovery: inspect reflex_suggestions → focusa_reflex_primitives (advisory only).
```

More: [`FOCUSA_FRIENDLY_ONBOARDING.md`](FOCUSA_FRIENDLY_ONBOARDING.md) and [`FOCUSA_TOOL_CHOREOGRAPHY_MAP.md`](FOCUSA_TOOL_CHOREOGRAPHY_MAP.md).

## Operator steering

Operator steering always wins. Focusa guides, preserves, and audits; it does not overrule a fresh operator instruction.
