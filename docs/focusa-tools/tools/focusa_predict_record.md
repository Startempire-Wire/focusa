# `focusa_predict_record`

Record a bounded, inspectable Focusa prediction. Predictions guide decisions; they never override operator steering. Use it when Record a bounded, inspectable Focusa prediction; core at task start, trajectory review, compaction review, and end-of-task reports. Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

## When to use

- Record a bounded, inspectable Focusa prediction; core at task start, trajectory review, compaction review, and end-of-task reports.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `prediction_type` (required; string): Prediction type, e.g. next_action_success|tool_choice|release_failure|stale_state|context_relevance|token_risk|cache_hit|drift_risk|workpoint_resume_success|compaction_recovery
- `predicted_outcome` (required; string): Predicted outcome.
- `confidence` (required; number): Confidence from 0.0 to 1.0.
- `recommended_action` (required; string): Recommended action if this prediction matters.
- `why` (required; string): Evidence-calibrated explanation.
- `context_refs` (optional; array): See the strict descriptor schema.
- `ontology_context` (optional; structured): Bounded ontology refs: object_refs, action_refs, tool_refs, evidence_refs, relation_refs.
- `project_root` (optional; string): Optional project root to bind prediction trajectory scope; auto-filled when omitted.
- `continuity_id` (optional; string): Optional continuity id to bind prediction trajectory scope; auto-filled when omitted.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_predict_record`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "prediction_type": "example",
  "predicted_outcome": "example",
  "confidence": 0,
  "recommended_action": "example",
  "why": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_predict_record.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- journaling raw logs
- unverified lessons without evidence

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_prediction`, `write_prediction`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `null`; preview supported: `null`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

Complete journey: `docs/agent/02-focusa-cohesive-project-flow.md` (Bootstrap/Genesis → linked Ladder/spec/tasks → Workpoint → Prepare/Act/Reconcile/Advance). Reuse valid state; select capabilities by condition rather than running a tool list as a script.
A rejected operation calls for exact-cause diagnosis and supported scoped recovery, not automatic mission abandonment; preserve genuine authority boundaries and resume only after verification.
Shared boundaries and conditional crosswalk: [AUTHORITY_MODEL.md](../../current/AUTHORITY_MODEL.md), [GOLDEN_WORKFLOW.md](../../current/GOLDEN_WORKFLOW.md), and [AGENT_ADAPTER_CONTRACT.md](../../current/AGENT_ADAPTER_CONTRACT.md).

- `focusa_evidence_capture` (likely_next)
- `focusa_predict_evaluate` (likely_next)
- `focusa_metacog_capture` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_evidence_capture`, `focusa_predict_evaluate`, `focusa_metacog_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:predictive-power`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_predict_record`; MCP: `focusa.predict.record`; OpenAI: `focusa_predict_record`.
- CLI: `focusa predict record`.
- REST: `POST /v1/predictions`.
- Specification: contract registry.
- Descriptor digest: `sha256:55b82f8d479850b886d0df5599d1b4e57a472218ab075d13398b7a4b35fcff1c`.
