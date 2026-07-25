# Focusa CLI and API Runbook

## Preconditions

- Confirm daemon health and exact project scope.
- Prefer `focusa_*` Pi tools for daemon/state operations; use CLI/REST for parity proof or non-Pi clients.
- Discover exact schemas in `docs/contracts/spec141/generated-capability-v2/cli-commands.json` and `rest-agent-operations.json`.

## Workflow

1. Read `docs/current/CLI_REFERENCE_CURRENT.md` and `docs/current/API_REFERENCE_CURRENT.md`.
2. Compare the requested operation with its Pi descriptor in `pi-tools.json`.
3. Preserve `project_root`, `continuity_id`, idempotency, approval, and receipt fields across interfaces.
4. Verify response posture and side effects before retrying mutations.
5. Capture API/CLI proof and link it to the active Workpoint.

## Recovery

- Health/scope failure: `focusa_tool_doctor` → `focusa_project_identity` → `focusa_project_verify`.
- Contract uncertainty: `focusa_tool_search` → `focusa_tool_describe` → `focusa_tool_graph`.
- Silent Session mutation: use the daemon-issued run/generation/approval/idempotency tuple.

## Done condition

CLI, REST, MCP, OpenAI, and Pi behavior preserve the same authority and output semantics for the operation.
