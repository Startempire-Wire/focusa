# `focusa_context_cognition_proof`

**Family:** `trajectory`
**Label:** Context Cognition Proof

## Purpose

Map the **Spec 100 `ContextCognitionPacket`** surfaces to a bounded set of proof commands. Returns the command list as a stable shape. Read-only; never mutates state.

Use when an operator wants a one-shot proof bundle for the context cognition packet: curl health, project identity, trajectory, workpoint; focusa CLI commands; node audit scripts.

## When to use

- Before claiming a packet is valid: copy-paste the commands into a shell and run.
- When handing off work to another agent: include the proof bundle.
- As a contract artifact: the proof_commands list is part of the Spec 100 §10 API contract.

Do not use for high-frequency polling; proof is built on demand.

## Parameters

- `project_root` — project scope. Defaults to Pi session cwd.
- `continuity_id` — optional workstream filter.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, `canonical=false`, `format=proof_commands`, the bounded `proof_commands` array, `command_count`, `workpoint_id`, and `rehydrate_id`.

The proof commands are a fixed-shape list: curl health/identity/trajectory/workpoint, focusa context-cognition view/render/proof, and the three static audit scripts.

## Example

```json
{
  "project_root": "/home/wirebot/focusa"
}
```

```text
focusa_context_cognition_proof ok | context cognition proof → 10 commands
ids: rehydrate_id=019eacb8-… workpoint_id=019eacb8-…
fields: command_count=10 format=proof_commands advisory=true
next: focusa_context_cognition → focusa_context_cognition_render → focusa_evidence_capture

1. curl 'http://127.0.0.1:8787/v1/health'
2. curl 'http://127.0.0.1:8787/v1/project/identity?project_root=/home/wirebot/focusa'
3. curl 'http://127.0.0.1:8787/v1/trajectory/view?project_root=/home/wirebot/focusa'
4. curl 'http://127.0.0.1:8787/v1/workpoint/current?project_root=/home/wirebot/focusa'
5. focusa context-cognition view --project-root /home/wirebot/focusa
6. focusa context-cognition render --project-root /home/wirebot/focusa
7. focusa context-cognition proof --project-root /home/wirebot/focusa
8. node scripts/validate-focusa-tool-contracts.mjs
9. node scripts/audit-focusa-tool-implementation-spec-gaps.mjs
10. node scripts/audit-focusa-tool-suite-safe.mjs
```

## Scope rules

- `project_root` is **required** — proof is scoped to project.
- Agent runtime paths (e.g. `/root/pi-mono`) are rejected with `failure_class=scope_mismatch`.
- Proof is read-only; no Workpoint, Trajectory, or HLT mutation occurs.

## Notes

- Per Spec 100 §10 the API contract requires a `proof` route to "map packet surfaces to proof commands".
- The `proof_commands` array is bounded (~10 commands) and stable for the same scope.
- The proof URLs use the daemon's own bind (default `http://127.0.0.1:8787`); update if you run a different bind.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `project_root_missing` — provide an explicit `project_root` and retry.
- `project_root_unverified` — call `focusa_project_verify` first.
- `scope_mismatch` — the `project_root` is an agent runtime path; pick a real project folder.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.

When `failure_class` is missing, treat the response as a successful proof bundle.

## Contract summary

- Family: `trajectory`
- Side effects: `read_state`
- Result envelope: `tool_result_v1`
- API routes: `GET /v1/context-cognition/proof`
- CLI commands: `focusa context-cognition proof`
- Core surface: `Spec100 §10 API contract + §13 tool wrapper`
- Spec: `docs/100-context-cognition-spec.md`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_context_cognition` — the full packet JSON.
- `focusa_context_cognition_render` — the compact text render.
- `focusa_evidence_capture` — link the proof bundle to the active Workpoint.
- `focusa_project_verify` — verify project identity on `project_root_unverified`.
- `focusa_tool_doctor` — diagnose daemon health when commands fail.
