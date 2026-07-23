# Focusa work loop Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.work_loop.control`



- Method/path: `POST /v1/work-loop/control`
- Family: `work_loop`
- Input schema: `focusa.work_loop_control.request.v1`
- Output schema: `focusa.work_loop_control.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `work_loop:write`
- Confirmation: `none`
- Idempotency key required: `true`
- Receipt required: `true`
- Reversible: `false`
- Required scope keys: `project_root`, `continuity_id`

### Example request

```json
{}
```

### Example result

```json
{}
```

### Failure and recovery

Failure classes: `focusa.operation_error.v1`.

Use the structured error recovery field, preserve the original scope and idempotency key when retry-safe, and run the indicated doctor/verify capability before any authority-sensitive retry.

## `focusa.work_loop.status`



- Method/path: `GET /v1/work-loop/status`
- Family: `work_loop`
- Input schema: `focusa.work_loop_status.request.v1`
- Output schema: `focusa.work_loop_status.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `work_loop:read`
- Confirmation: `none`
- Idempotency key required: `false`
- Receipt required: `false`
- Reversible: `false`
- Required scope keys: `project_root`, `continuity_id`

### Example request

```json
{}
```

### Example result

```json
{}
```

### Failure and recovery

Failure classes: `focusa.operation_error.v1`.

Use the structured error recovery field, preserve the original scope and idempotency key when retry-safe, and run the indicated doctor/verify capability before any authority-sensitive retry.
