# Focusa Silent Sessions Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Refresh preferred address, timezone, local time, operator state, goals, constraints, desired pace, and confirmed timeline.
- Treat cwd and missing markers as weak evidence; inspect legacy project signals before suggesting creation or binding.
- Start wall-clock measurement and a human-readable bounded prediction for meaningful work; evaluate it against actual duration at completion.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text

```

## Minimal path

1. Call `focusa_silent_sessions` with only required bounded inputs.

## Current domain procedure

1. Verify typed project/workstream scope before durable mutation.
2. Return bounded evidence and executable recovery.

## Temporal authority contract

- Status returns bounded `focusa.silent_session_temporal_context.v1` for the exact session project and continuity scope.
- Context includes run start/end/elapsed time, configured wall-clock ceiling, remaining budget, timeout state, event-count progress, cancellation state, deadline authority, forecast range, and bounded temporal warnings.
- Start, pause, resume, interrupt, cancel, restart, text input, and key input return the same context plus the mutation-specific `focusa.silent_session_temporal_guard.v1` receipt.
- Mutations fail closed when HumanCalendarContext, TemporalPriorityFrame, or TemporalExecutionGuard authority is missing, stale, scope-mismatched, or does not authorize the action.
- Terminal lifecycle state remains `terminal_pending_receipt` until closure evidence and receipt references are durably settled; terminal state alone is not completion proof.
- Resource timeout fields do not independently prove Spec 131 parent-budget propagation, paired-clock lineage, cancellation effectiveness, or possible-effect reconciliation; those obligations remain separately gated.

## Branches

- Unknown tool/schema: `focusa_tool_search` → `focusa_tool_describe`.
- Scope conflict: `focusa_project_verify` → `focusa_workpoint_checkpoint`.
- Daemon/degraded state: `focusa_tool_doctor`; retry only with safe posture.
- Resource timeout: `focusa_resource_mode` → bounded `focusa_traverse`.
- Browser failure: UIAI diagnostics → `focusa_browser_diagnostics_intake` → evidence.
- Mutation ambiguity: inspect side effects/receipts before retry; require operator confirmation when declared.

## Evidence and closure

- Capture stable file/test/API/browser/receipt refs.
- Link proof to the active Workpoint.
- Evaluate relevant predictions and reusable learning only after outcome is known.
- Done: The scoped operation is verified, evidenced, and handed to the next owning skill.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.
