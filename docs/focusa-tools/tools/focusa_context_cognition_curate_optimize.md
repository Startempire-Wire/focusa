# `focusa_context_cognition_curate_optimize`

Spec 100 Phase 5 — submit a Cognition Optimizer artifact and get the promote/rollback decision. Returns the decision per the §15 promotion rule (eval_score > baseline_score AND eval_score >= score_threshold). Appends to cognition-optimizer-artifacts/{hash}/artifacts.jsonl. Use it when Spec 100 Phase 5 — submit a Cognition Optimizer artifact and get the promote/rollback decision per §15 promotion rule. Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

## When to use

- Spec 100 Phase 5 — submit a Cognition Optimizer artifact and get the promote/rollback decision per §15 promotion rule.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (optional; string): Project root. Defaults to Pi session cwd.
- `continuity_id` (optional; string): Optional continuity id filter.
- `module_name` (optional; string): Module name (default: curator).
- `prompt_artifact_ref` (required; string): Path or ref id of the candidate prompt/module artifact.
- `eval_score` (required; number; min=0, max=1): Candidate artifact's eval F1 score.
- `baseline_score` (optional; number; min=0, max=1): Baseline F1 to beat. Defaults to 0.0.
- `score_threshold` (optional; number; min=0, max=1): F1 threshold for promotion. Defaults to 0.5.
- `eval_run_id` (optional; string): Optional CuratorEvalRun id that produced eval_score.
- `rollback` (optional; boolean): Explicit rollback override. Defaults to false.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_context_cognition_curate_optimize`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "prompt_artifact_ref": "example",
  "eval_score": 0
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_context_cognition_curate_optimize.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- overriding Workpoint/operator authority
- merging sessions on goal similarity alone

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_cognition_optimizer_artifact`, `write_cognition_optimizer_artifact`
- Advisory source hints (not dispatch/replay guarantees): read-only `false`; destructive `false`; idempotent `false`; open-world `false`. Verify selected action/schema/policy separately.
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

- `focusa_context_cognition_optimizer_artifacts` (likely_next)
- `focusa_predict_record` (likely_next)
- `focusa_metacog_capture` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_context_cognition_optimizer_artifacts`, `focusa_predict_record`, `focusa_metacog_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_context_cognition_curate_optimize`; MCP: `focusa.context.cognition.curate.optimize`; OpenAI: `focusa_context_cognition_curate_optimize`.
- CLI: `focusa context-cognition curate-optimize`.
- REST: `POST /v1/context-cognition/curate/optimize`.
- Specification: contract registry.
- Descriptor digest: `sha256:7974e386e634f4c5ddab533ab799f4933217c67e276b795e5280b4ff132e7292`.
