# `focusa_metacog_recent_adjustments`

Best safe helper for finding recent adjustment ids before evaluation or promotion decisions. Use it when Best safe helper for finding recent adjustment ids before evaluation or promotion decisions. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Best safe helper for finding recent adjustment ids before evaluation or promotion decisions.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `limit` (optional; integer; min=1, max=20): How many recent adjustments to return (default 5).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_metacog_recent_adjustments`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_metacog_recent_adjustments.md

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
- Side effects: `read_only`, `read_only`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_metacog_evaluate_outcome` (likely_next)
- `focusa_metacog_doctor` (likely_next)
- `focusa_metacog_reflect` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_metacog_evaluate_outcome`, `focusa_metacog_doctor`, `focusa_metacog_reflect`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_metacog_recent_adjustments`; MCP: `focusa.metacog.recent.adjustments`; OpenAI: `focusa_metacog_recent_adjustments`.
- CLI: `focusa metacognition recent-adjustments`.
- REST: `GET /v1/metacognition/adjustments/recent`.
- Specification: contract registry.
- Descriptor digest: `sha256:f773b9fd4a31be3fb237a008464bf52e984285527b37f4e973edaed9e733ff23`.
