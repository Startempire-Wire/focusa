# `focusa_metacog_evaluate_outcome`

Judge whether an adjustment improved results and whether the learning should be promoted. Use it when Judge whether an adjustment improved results and whether the learning should be promoted. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Judge whether an adjustment improved results and whether the learning should be promoted.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `adjustment_id` (required; string): Adjustment id.
- `observed_metrics` (optional; array): Observed metric ids.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_metacog_evaluate_outcome`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "adjustment_id": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_metacog_evaluate_outcome.md

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
- Side effects: `write_state`, `write_state`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_metacog_capture` (likely_next)
- `focusa_predict_stats` (likely_next)
- `focusa_decide` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_metacog_capture`, `focusa_predict_stats`, `focusa_decide`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:focusa-evidence-outcomes`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_metacog_evaluate_outcome`; MCP: `focusa.metacog.evaluate.outcome`; OpenAI: `focusa_metacog_evaluate_outcome`.
- CLI: `focusa metacognition evaluate`.
- REST: `POST /v1/metacognition/evaluate`, `GET /v1/metacognition/evaluations/recent`.
- Specification: contract registry.
- Descriptor digest: `sha256:53326c5bcaac06f5022e084bf48404bb22fc40da27e5706cf3a4d9495af4a3ec`.
