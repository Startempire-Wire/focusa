# `focusa_context_cognition_curate_optimize`

Spec 100 Phase 5 — submit a Cognition Optimizer artifact and get the promote/rollback decision. Returns the decision per the §15 promotion rule (eval_score > baseline_score AND eval_score >= score_threshold). Appends to cognition-optimizer-artifacts/{hash}/artifacts.jsonl. Use it when Spec 100 Phase 5 — submit a Cognition Optimizer artifact and get the promote/rollback decision per §15 promotion rule. It returns a typed Focusa result with bounded recovery and likely next capabilities.

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

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

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
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_context_cognition_optimizer_artifacts` (likely_next)
- `focusa_predict_record` (likely_next)
- `focusa_metacog_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_context_cognition_optimizer_artifacts`, `focusa_predict_record`, `focusa_metacog_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_context_cognition_curate_optimize`; MCP: `focusa.context.cognition.curate.optimize`; OpenAI: `focusa_context_cognition_curate_optimize`.
- CLI: `focusa context-cognition curate-optimize`.
- REST: `POST /v1/context-cognition/curate/optimize`.
- Assignable: `true`; parity: `full`.
- Specification: contract registry.
- Descriptor digest: `sha256:4ac5a9d0d0bc42a0e2b9d0fa8a073a4095f2160b92e5ce19c59632b9df64acce`.
