# Focusa project Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.project.identity`



- Method/path: `GET /v1/project/identity`
- Family: `project`
- Input schema: `focusa.project_identity.request.v1`
- Output schema: `focusa.project_identity.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `project:read`
- Confirmation: `none`
- Idempotency key required: `false`
- Receipt required: `false`
- Reversible: `false`
- Required scope keys: `project_root`

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

## `focusa.project.verify`



- Method/path: `GET /v1/project/verify`
- Family: `project`
- Input schema: `focusa.project_verify.request.v1`
- Output schema: `focusa.project_verify.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `project:read`
- Confirmation: `none`
- Idempotency key required: `false`
- Receipt required: `false`
- Reversible: `false`
- Required scope keys: `project_root`

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
