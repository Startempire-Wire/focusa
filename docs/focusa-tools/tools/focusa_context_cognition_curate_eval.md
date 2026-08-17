# `focusa_context_cognition_curate_eval`

Spec 100 Phase 4 — run a curator eval case. Computes precision/recall/F1 vs. expected_selected_paths. Appends to curator-eval-ledger/{hash}/eval-runs.jsonl. Returns run_id, eval_ref, scores, and promoted flag (F1 > baseline_f1 AND F1 >= score_threshold). Use it when Spec 100 Phase 4 — run a curator eval case, compute precision/recall/F1, append to curator-eval-ledger JSONL. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Spec 100 Phase 4 — run a curator eval case, compute precision/recall/F1, append to curator-eval-ledger JSONL.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (optional; string): Project root. Defaults to Pi session cwd.
- `continuity_id` (optional; string): Optional continuity id filter.
- `case_id` (optional; string): Optional case id; defaults to a generated UUID.
- `target` (optional; string): Curator target string.
- `token_budget` (optional; integer; min=1, max=1000000): Token budget for the selection. Defaults to 2000.
- `candidates` (optional; array): See the strict descriptor schema.
- `expected_selected_paths` (optional; array): Operator-supplied expected selected paths for precision/recall/F1.
- `score_threshold` (optional; number; min=0, max=1): F1 threshold for promotion. Defaults to 0.5.
- `baseline_f1` (optional; number; min=0, max=1): Baseline F1 to beat. Defaults to 0.0.
- `evidence_refs` (optional; array): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_context_cognition_curate_eval`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_context_cognition_curate_eval.md

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
- Side effects: `write_curator_eval`, `write_curator_eval`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_context_cognition_curate_optimize` (likely_next)
- `focusa_metacog_capture` (likely_next)
- `focusa_predict_record` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_context_cognition_curate_optimize`, `focusa_metacog_capture`, `focusa_predict_record`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_context_cognition_curate_eval`; MCP: `focusa.context.cognition.curate.eval`; OpenAI: `focusa_context_cognition_curate_eval`.
- CLI: `focusa context-cognition curate-eval`, `focusa context-cognition curate-eval-runs`.
- REST: `POST /v1/context-cognition/curate/eval`, `GET /v1/context-cognition/curate/eval/runs`.
- Specification: contract registry.
- Descriptor digest: `sha256:d1f0f0fba3e29fd87b94f6c3ccf26debaab6d1b2d7846bcd2092060a1e563ee3`.
