# Focusa Agent Utility Card

## v0.9.142 capability surface

The Utility Card routes 135 typed tools and 29 generated skills through project/workstream scope. Alongside Workpoint, Trajectory, evidence, prediction, metacognition, and recovery, the current surface includes Context Cognition, Project Card/Genesis, Temporal Authority, preload, Silent Sessions, UIAI/WebMCP workflows, device pairing, Mission Canvas, and progressive Tool Discovery.

This card is injected by the Pi extension at startup/reload and included in the system prompt so agents are aware of Focusa as a friendly project navigation utility without reading the repository first.

## Runtime card content requirements

The card must mention:

- Focusa availability/degraded status.
- Friendly Focusa Q: where am I, what project/architecture is this, where are we going, next useful move, proof, compounding lesson.
- Tool route hints: orient, execute, prove, learn, recover, and inspect reflexes when a result exposes `reflex_suggestions`.
- Current mission or latest operator/current repo fallback.
- Current next anchor or `focusa_workpoint_resume` fallback.
- Current project folder (`project_root`) or project-root binding rule.
- `focusa_project_identity` / `focusa_project_verify` before cross-project state trust.
- `focusa_trajectory_view` / `focusa_trajectory_assess` for goals, state, destination, and gaps.
- `focusa_tool_doctor` as the first recovery tool when uncertain; `focusa_reflex_primitives` as read-only advisory metadata for recurring recovery patterns.
- `focusa_workpoint_checkpoint` before compaction/model switch/fork/risky continuation.
- `focusa_workpoint_resume` after compaction/reload/resume.
- Workpoint project-folder and continuity rules.
- evidence capture/linking after proof.
- prediction record/evaluate around risky or uncertain actions.
- Metacognition/work-loop tool families for learning/continuous work.
- Focus State tools as note/decision slots, not the whole project workflow.
- Spec97 Reflex Primitives as bounded, read-only suggestions; they never override operator steering, Workpoint authority, or reducer-approved mutation paths.
- Missing active Pi frame fallback: `Attentive and awaiting operator direction`, with continued help from operator/repo context until scope is safe.
- Compaction fallback rule: related canonical fallback, not blank `none` fields.
- Operator steering wins.

## Source

Runtime implementation:

```text
apps/pi-extension/src/awareness.ts
apps/pi-extension/src/turns.ts
```

Companion docs:

```text
docs/current/FOCUSA_FRIENDLY_ONBOARDING.md
docs/current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md
```

Validation:

```bash
node scripts/validate-agent-awareness.mjs
```
