# `focusa_project_card_outcome`

Attach a final outcome/result to a specific project-card algorithm_run_id and update learned project-card weights. Use it when Attach a verified result to a project-card algorithm_run_id so project-card learning weights and future bootstrap/sequence planning can improve. Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

## When to use

- Attach a verified result to a project-card algorithm_run_id so project-card learning weights and future bootstrap/sequence planning can improve.
- Capability family: `project_identity`; namespace: `focusa.project_identity`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `algorithm_run_id` (required; string): Project-card algorithm_run_id returned by focusa_project_card.
- `actual_outcome` (required; string): Observed final outcome/result for that algorithm run.
- `score` (optional; number): Optional outcome score from 0.0 to 1.0; defaults to 1.0.
- `evidence_refs` (optional; array): Evidence refs proving the outcome.
- `project_root` (optional; string): Optional project root associated with the run.
- `notes` (optional; string): Optional bounded note about the result.
- `task_timing` (optional; structured): Optional override timing object; Pi auto-populates elapsed task timing when omitted.
- `token_usage` (optional; structured): Optional override token usage object; Pi auto-populates provider/estimated token counts when omitted.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_project_card_outcome`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "algorithm_run_id": "example",
  "actual_outcome": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_project_card_outcome.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- assuming unsafe broad cwd is canonical
- skipping verify after scope mismatch

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_project_card_outcome`, `write_project_card_outcome`
- Advisory source hints (not dispatch/replay guarantees): read-only `false`; destructive `false`; idempotent `false`; open-world `true`. Verify selected action/schema/policy separately.
- Confirmation required: `unknown (null)`; preview supported: `unknown (null)`. Inspect the selected action, strict installed schema and policy; unknown is not permission.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

Complete journey: `docs/agent/02-focusa-cohesive-project-flow.md` (Bootstrap/Genesis → linked Ladder/spec/tasks → Workpoint → Prepare/Act/Reconcile/Advance). Reuse valid state; select capabilities by condition rather than running a tool list as a script.
A rejected operation calls for exact-cause diagnosis and supported scoped recovery, not automatic mission abandonment; preserve genuine authority boundaries and resume only after verification.
Shared boundaries and conditional crosswalk: [AUTHORITY_MODEL.md](../../current/AUTHORITY_MODEL.md), [GOLDEN_WORKFLOW.md](../../current/GOLDEN_WORKFLOW.md), and [AGENT_ADAPTER_CONTRACT.md](../../current/AGENT_ADAPTER_CONTRACT.md).

- `focusa_project_card` (likely_next)
- `focusa_predict_record` (likely_next)
- `focusa_metacog_capture` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_project_card`, `focusa_predict_record`, `focusa_metacog_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-project-scope`, `skill:focusa-evidence-outcomes`
- Runbooks: `runbook:project_identity`
- Pi: `focusa_project_card_outcome`; MCP: `focusa.project.card.outcome`; OpenAI: `focusa_project_card_outcome`.
- CLI: `focusa project card-outcome`.
- REST: `POST /v1/project/card/outcome`.
- Specification: contract registry.
- Descriptor digest: `sha256:c8a3c913aa5e9ca963b8c43c93c7955f6c7370a866aced7e97e4952809b224d3`.
