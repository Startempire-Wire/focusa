# Focusa license Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.license.validate`

- Method/path: `POST /v1/license/validate`
- Family: `license`
- Input schema: `focusa.license_validate.request.v1`
- Output schema: `focusa.license_validate.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `license:write`
- Confirmation: `none`
- Idempotency key required: `true`
- Receipt required: `true`
- Reversible: `false`
- Required scope keys: none

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
