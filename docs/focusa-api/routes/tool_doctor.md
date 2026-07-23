# Focusa tool doctor Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.tool_doctor`



- Method/path: `GET /v1/tool-doctor`
- Family: `diagnostics`
- Input schema: `focusa.tool_doctor.request.v1`
- Output schema: `focusa.tool_doctor.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `diagnostics:read`
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
