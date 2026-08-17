# `focusa_metacog_loop_run`

Run capture -> retrieve -> reflect -> adjust -> evaluate in one move. Best composite tool when you want learning workflow compression instead of manual chaining. Use it when Run capture -> retrieve -> reflect -> adjust -> evaluate in one move. Best composite tool when you want learning workflow compression instead of manual chaining. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Run capture -> retrieve -> reflect -> adjust -> evaluate in one move. Best composite tool when you want learning workflow compression instead of manual chaining.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `current_ask` (required; string): Current ask driving retrieval and reuse.
- `turn_range` (required; string): Turn range expression for reflection.
- `kind` (optional; string): Optional capture kind (default workflow_signal).
- `content` (optional; string): Optional capture content; defaults to current_ask.
- `rationale` (optional; string): Optional capture rationale.
- `confidence` (optional; number; min=0, max=1): Optional confidence 0..1.
- `strategy_class` (optional; string): Optional strategy class.
- `scope_tags` (optional; array): See the strict descriptor schema.
- `k` (optional; integer; min=1, max=50): Top-k retrieval size.
- `failure_classes` (optional; array): See the strict descriptor schema.
- `selected_updates` (optional; array): See the strict descriptor schema.
- `observed_metrics` (optional; array): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_metacog_loop_run`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "current_ask": "example",
  "turn_range": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_metacog_loop_run.md

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

- `focusa_predict_stats` (likely_next)
- `focusa_workpoint_checkpoint` (likely_next)
- `focusa_metacog_doctor` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_predict_stats`, `focusa_workpoint_checkpoint`, `focusa_metacog_doctor`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_metacog_loop_run`; MCP: `focusa.metacog.loop.run`; OpenAI: `focusa_metacog_loop_run`.
- CLI: `focusa metacognition loop run`.
- REST: `POST /v1/metacognition/capture`, `POST /v1/metacognition/retrieve`, `POST /v1/metacognition/reflect`, `POST /v1/metacognition/adjust`, `POST /v1/metacognition/evaluate`.
- Specification: contract registry.
- Descriptor digest: `sha256:c8abe94c4dad63eb612cbc22229b56b6676775c211f371cadfc887dfce0959af`.
