# `focusa_epistemic_operation`

Invoke one exact generated Spec 138/138A operation through durable typed API authority; the client never settles authority locally. Use it when Invoke one exact generated Spec 138/138A operation through durable typed API authority. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Invoke one exact generated Spec 138/138A operation through durable typed API authority.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `operation_id` (required; string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string): See the strict descriptor schema.
- `id` (optional; string): Value for canonical {id} path segments.
- `event` (optional; structured): Typed ScopedAuthorityEvent required for mutations.
- `project_root` (optional; string): Explicit or current verified project root.
- `continuity_id` (optional; string): Explicit or current continuity id.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_epistemic_operation`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{
  "operation_id": "prediction.question.create"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_epistemic_operation.md

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

- Scope: `{"kind":"write","route_family":"prediction-authority:operation"}`
- Authority: `{"kind":"canonical_write"}`
- Side effects: `typed_read_or_canonical_epistemic_mutation`, `typed_read_or_canonical_epistemic_mutation`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_prediction_authority` (likely_next)
- `focusa_metacog_retrieve` (likely_next)
- `focusa_trajectory_view` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_prediction_authority`, `focusa_metacog_retrieve`, `focusa_trajectory_view`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_epistemic_operation`; MCP: `focusa.epistemic.operation`; OpenAI: `focusa_epistemic_operation`.
- CLI: `focusa predict operation --operation <operation-id>`.
- REST: `POST /v1/prediction-questions`, `POST /v1/information-sets`, `POST /v1/predictions/commit`, `POST /v1/predictions/{id}/supersede`, `GET /v1/predictions/{id}`, `GET /v1/predictions/recent`, `POST /v1/outcomes/claim`, `POST /v1/outcomes/{id}/dispute`, `POST /v1/outcomes/resolve`, `POST /v1/outcomes/{id}/correct`, `POST /v1/evaluations/predictions`, `GET /v1/calibration/reports`, `POST /v1/metacognition/signals`, `POST /v1/metacognition/reflections`, `POST /v1/metacognition/adjustments`, `POST /v1/metacognition/evaluations`, `POST /v1/learning/candidates/{id}/decide`, `POST /v1/learning/{id}/apply`, `POST /v1/learning/transfers/resolve`, `GET /v1/learning/retrieve`, `GET /v1/learning/conflicts`, `POST /v1/learning/{id}/expire`, `POST /v1/learning/{id}/supersede`, `POST /v1/learning/{id}/revoke`, `POST /v1/learning/{id}/rollback`, `POST /v1/learning/consolidate`, `GET /v1/self-model`.
- Assignable: `true`; parity: `full`.
- Specification: contract registry.
- Descriptor digest: `sha256:43b7a5020a061307c82cc301ab1739fa7d27390ff91a2753c2a6650e4f01bc5e`.
