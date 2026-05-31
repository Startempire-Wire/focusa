# Focusa Model-Visible Awareness Surfaces

Purpose: describe what the LLM actually sees from Focusa and the precedence of those signals.

## Surfaces visible to the model

1. **Tool definitions** — always available through the Pi tool registry.
   The model sees each `focusa_*` tool name, description, parameter schema, and prompt snippet.

2. **Focusa Utility Card** — injected into the system prompt at agent start/reload; also shown once as a visible card.
   Source: `apps/pi-extension/src/awareness.ts` via `buildFocusaUtilityCard()`.

3. **Focusa Focus Slice** — injected on each LLM context event. When Focusa has a scoped frame it includes live Focus State; when not, it still injects a compact local Project/Trajectory/Architecture fallback card.
   Source: `apps/pi-extension/src/turns.ts` context handler.

4. **Tool Affordances** — included inside the Focus Slice as `TOOL_AFFORDANCES`.
   Source: `selectFocusSliceToolAffordances()` in `apps/pi-extension/src/tool-contracts.ts`.

5. **Skill descriptions** — visible before loading; full skill files become visible when loaded.
   Source: `/root/.pi/skills/focusa*/SKILL.md` and project skill copies.

6. **Tool results** — every Focusa tool returns a visible summary plus `details.tool_result_v1` with status, canonical/degraded posture, failure class, retry posture, recovery/misuse hints, side effects, evidence refs, next-tool hints, and optional `reflex_suggestions`.

Docs are **not** automatically visible unless injected by a card/slice/skill or read by the model.

## Attention / recall gap (current design boundary)

A model seeing a Focusa surface is not the same as the model attending to it, applying it, or treating it as current-action authority. Workpoint, Trajectory, Focus Slice, and tool results are preserved/retrieved inputs; they still need a small action gate that proves critical facts were pinned or recapped before project-scoped tools run.

Current audit finding: the Pi Focus Slice hot path can truncate from the bottom under pressure, and the existing truncation keeps only the first four lines. Therefore any anti-forgetting guard must be non-droppable and must appear before verbose Workpoint/Trajectory JSON. The planned surface is a tiny `MEMORY_ANCHOR` plus `AttentionRecallVerdict` / current-ask scope verdict that states task, must-not-forget facts, latest report summary ref, action authority, visible recap requirement, and required next step.

Until that verdict is implemented, this document describes model-visible inputs and intended precedence, not a hard proof that the model attended to every critical input.

## Continuous trajectory/project display

The continuous model-facing display is the **Focusa Focus Slice**, not a separate always-on UI widget. When the context handler runs, the model sees a `PROJECT_TRAJECTORY` section containing:

- `PROJECT_IDENTITY`: status, `project_root`, `continuity_id`, session id, confidence.
- `PROJECT_INFRA`: canonical name, project id, workspace kind, repo remote, beads prefix, and architecture-boundary reminder.
- `PROJECT_ENVIRONMENT`: marker-backed and repo-scanned root/live/local URLs, live-vs-local environment, deploy target/location/command, and a reminder not to assume `.local` is active.
- `PROJECT_ARCHITECTURE`: evidence-backed local architecture digest (`stack`, manifest name, key dirs, deploy surfaces, docs, tests, confidence, source refs) with a reminder to verify via docs/ontology/evidence.
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
6. Tool Affordances / next-tool routing / Reflex Primitive suggestions.
7. Focus frame/current focus/intent.
8. Ontology active objects/link paths/valid next actions.
9. Constraints and decisions.
10. Evidence/results/failures/next steps/artifact handles.

Operator steering always wins, but stale transcript tail does not outrank canonical scoped Workpoint/Trajectory context. A future attention/recall verdict should sit above ordinary Focus Slice sections because it decides whether retrieved context is safe to use for the current action.

## Degraded / fallback posture

`PROJECT_TRAJECTORY` is now always attempted from the context hook. If Focusa is unavailable, the scoped frame is missing, or trajectory lookup fails, the model still sees a compact fallback card with `PROJECT_IDENTITY`, `PROJECT_INFRA`, `PROJECT_ARCHITECTURE`, degraded sufficiency, and the recommended recovery route. Unsafe broad roots withhold architecture facts until `focusa_project_identity` verifies an explicit project root.

## Current improvement

The Friendly Focusa Q now includes project infrastructure/architecture orientation, and the per-call trajectory slice includes both `PROJECT_INFRA` and `PROJECT_ARCHITECTURE` so the model does not infer architecture from folder names alone. Machine-readable choreography edges are available at `docs/current/focusa-tool-choreography.json` and `GET /v1/ontology/tool-choreography`; live choreography can weight edges using evaluated prediction evidence.

Spec97 Reflex Primitives are now model-visible through bounded `reflex_suggestions` in Pi/API result envelopes, `focusa_reflex_primitives`, direct `GET /v1/reflex/primitives`, and `surface=reflex_primitives` traversal. These are advisory recovery affordances only; they do not replace operator steering or canonical Workpoint/Trajectory scope gates.
