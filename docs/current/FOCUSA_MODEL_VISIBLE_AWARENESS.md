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

## Stored, retrieved, attended memory, and action authority

Focusa now separates four states that were previously easy to conflate:

- **Stored memory** — durable Focus State, Workpoints, Trajectory, evidence handles, telemetry traces, report-summary handles, and project-switch ledger observations.
- **Retrieved memory** — a WorkpointResumePacket, Trajectory view, bounded traverse slice, or Focus Slice section selected for the next model turn.
- **Attended memory** — the non-droppable prefix in the Focus Slice/compaction prompt: `MEMORY_ANCHOR`, `ATTENTION_RECALL_VERDICT`, `CURRENT_ASK_SCOPE_VERDICT`, and visible recap lines when tool output flood risk is active.
- **Action authority** — the final gate for file/API/tool action. A packet can be `canonical_for_saved_scope=true` while `action_authority_for_current_ask=false` when the latest operator ask names a different project/root/remote.

Implemented model-visible attention surfaces:

- `MEMORY_ANCHOR` pins task, must-not-forget facts, latest report summary ref, evidence refs, next action, and action authority before verbose Workpoint/Trajectory JSON.
- `ATTENTION_RECALL_VERDICT` reports attentive/attention-risk/conflict status, recap requirement, attention risks, and required next steps.
- `CURRENT_ASK_SCOPE_VERDICT` compares current ask/project-switch ledger evidence against saved Workpoint scope and suppresses action on conflict.
- Tool-output flood detection forces a visible recap and replays `latest_report_summary_ref` instead of relying on transcript scrollback.
- `scope_conflict_detected` telemetry and the Spec97 `detect_semantic_project_scope_conflict` primitive distinguish semantic current-ask conflicts from API-level `scope_mismatch`.

Regression evidence: `tests/spec_attention_recall_anchor_static_test.sh`, `tests/spec_report_replay_static_test.sh`, `tests/spec_project_scope_override_static_test.sh`, `tests/pi_session_project_switch_ledger_static_test.sh`, `tests/spec97_semantic_scope_conflict_primitive_static_test.sh`, and `tests/scope_routing_regression_eval.sh`.

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
2. Protected attention prefix: `MEMORY_ANCHOR`, `ATTENTION_RECALL_VERDICT`, `CURRENT_ASK_SCOPE_VERDICT`, and required recap lines.
3. Hard safety + identity prior (`project_root + continuity_id`) and `action_authority_for_current_ask`.
4. ResourceMode when non-normal.
5. Project Trajectory (`PROJECT_TRAJECTORY`).
6. Workpoint continuation packet (`canonical_for_saved_scope`, not automatically action authority).
7. Tool Affordances / next-tool routing / Reflex Primitive suggestions.
8. Focus frame/current focus/intent.
9. Ontology active objects/link paths/valid next actions.
10. Constraints, decisions, evidence/results/failures/next steps/artifact handles.

Operator steering always wins, but stale transcript tail does not outrank canonical scoped Workpoint/Trajectory context. The attention/recall and current-ask scope verdicts sit above ordinary Focus Slice sections because they decide whether retrieved context is safe to use for the current action.

## Degraded / fallback posture

`PROJECT_TRAJECTORY` is now always attempted from the context hook. If Focusa is unavailable, the scoped frame is missing, or trajectory lookup fails, the model still sees a compact fallback card with `PROJECT_IDENTITY`, `PROJECT_INFRA`, `PROJECT_ARCHITECTURE`, degraded sufficiency, and the recommended recovery route. Unsafe broad roots withhold architecture facts until `focusa_project_identity` verifies an explicit project root.

## Current improvement

The Friendly Focusa Q now includes project infrastructure/architecture orientation, and the per-call trajectory slice includes both `PROJECT_INFRA` and `PROJECT_ARCHITECTURE` so the model does not infer architecture from folder names alone. Machine-readable choreography edges are available at `docs/current/focusa-tool-choreography.json` and `GET /v1/ontology/tool-choreography`; live choreography can weight edges using evaluated prediction evidence.

Spec97 Reflex Primitives are now model-visible through bounded `reflex_suggestions` in Pi/API result envelopes, `focusa_reflex_primitives`, direct `GET /v1/reflex/primitives`, and `surface=reflex_primitives` traversal. These are advisory recovery affordances only; they do not replace operator steering or canonical Workpoint/Trajectory scope gates.

## Failure analysis/reporting discipline

A model-forgetting or scope-override report is not proven by agreeing with operator wording. Reports must cite at least one evidence surface (Focus Slice/compaction block, WorkpointResumePacketV2 fields, telemetry trace, project-switch ledger observation, test, or evidence handle) and should list rejected hypotheses such as missing stored memory, daemon unavailability, API `scope_mismatch`, or operator ambiguity when the evidence rules them out.
