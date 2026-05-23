# Focusa Model-Visible Awareness Surfaces

Purpose: describe what the LLM actually sees from Focusa and the precedence of those signals.

## Surfaces visible to the model

1. **Tool definitions** — always available through the Pi tool registry.  
   The model sees each `focusa_*` tool name, description, parameter schema, and prompt snippet.

2. **Focusa Utility Card** — injected into the system prompt at agent start/reload; also shown once as a visible card.  
   Source: `apps/pi-extension/src/awareness.ts` via `buildFocusaUtilityCard()`.

3. **Focusa Focus Slice** — injected on each LLM context event when Focusa is available and an active scoped frame exists.  
   Source: `apps/pi-extension/src/turns.ts` context handler.

4. **Tool Affordances** — included inside the Focus Slice as `TOOL_AFFORDANCES`.  
   Source: `selectFocusSliceToolAffordances()` in `apps/pi-extension/src/tool-contracts.ts`.

5. **Skill descriptions** — visible before loading; full skill files become visible when loaded.  
   Source: `/root/.pi/skills/focusa*/SKILL.md` and project skill copies.

6. **Tool results** — every Focusa tool returns a visible summary plus `details.tool_result_v1` with status, canonical/degraded posture, failure class, retry posture, side effects, evidence refs, and next-tool hints.

Docs are **not** automatically visible unless injected by a card/slice/skill or read by the model.

## Continuous trajectory/project display

The continuous model-facing display is the **Focusa Focus Slice**, not a separate always-on UI widget. When the context handler runs, the model sees a `PROJECT_TRAJECTORY` section containing:

- `PROJECT_IDENTITY`: status, `project_root`, `continuity_id`, session id, confidence.
- `PROJECT_INFRA`: canonical name, project id, workspace kind, repo remote, beads prefix, and architecture-boundary reminder.
- `TRAJECTORY_GOALS`: long/mid/low goals, desired state, short-term goal.
- `TRAJECTORY_SIMILARITY_GROUP`: advisory grouping and authority boundary.
- `CURRENT_VERIFIED_STATE` and `ACTIVE_GAP`.
- `WORKPOINT_CANDIDATE`.
- `TRAJECTORY_EVIDENCE`, `TRAJECTORY_DO_NOT_USE`, and `CONTEXT_SUFFICIENCY`.

The startup/reload Utility Card also gives a compact orientation route, but it is not the only live context source.

## Precedence / priority order

Focus Slice sections are ordered by priority in `turns.ts`. The practical model precedence is:

1. Operator steering/current ask.
2. Hard safety + identity prior (`project_root + continuity_id`).
3. ResourceMode when non-normal.
4. Project Trajectory (`PROJECT_TRAJECTORY`).
5. Workpoint continuation packet.
6. Tool Affordances / next-tool routing.
7. Focus frame/current focus/intent.
8. Ontology active objects/link paths/valid next actions.
9. Constraints and decisions.
10. Evidence/results/failures/next steps/artifact handles.

Operator steering always wins, but stale transcript tail does not outrank canonical scoped Workpoint/Trajectory context.

## Known limitation

`PROJECT_TRAJECTORY` appears only when Focusa is available, the project folder is safe, and the Pi session has an active scoped Focus frame. If the frame is missing or unsafe, the model still sees the Utility Card and tool definitions, but not the full continuous Focus Slice.

## Current improvement

The Friendly Focusa Q now includes project infrastructure/architecture orientation, and the per-call trajectory slice includes `PROJECT_INFRA` so the model does not infer architecture from folder names alone.
