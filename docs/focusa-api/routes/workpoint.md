# Focusa workpoint Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.workpoint.checkpoint`

- Method/path: `POST /v1/workpoint/checkpoint`
- Family: `workpoint`
- Input schema: `focusa.workpoint_checkpoint.request.v1`
- Output schema: `focusa.workpoint_checkpoint.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `workpoint:write`
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

## `focusa.workpoint.link_evidence`

- Method/path: `POST /v1/workpoint/link-evidence`
- Family: `workpoint`
- Input schema: `focusa.workpoint_link_evidence.request.v1`
- Output schema: `focusa.workpoint_link_evidence.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `workpoint:write`
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

## `focusa.workpoint.resume`

- Method/path: `GET /v1/workpoint/resume`
- Family: `workpoint`
- Input schema: `focusa.workpoint_resume.request.v1`
- Output schema: `focusa.workpoint_resume.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `workpoint:read`
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
