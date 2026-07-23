# Focusa device Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.device_pair.start`



- Method/path: `POST /v1/device/pair/start`
- Family: `device`
- Input schema: `focusa.device_pair_start.request.v1`
- Output schema: `focusa.device_pair_start.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `device:write`
- Confirmation: `none`
- Idempotency key required: `false`
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

## `focusa.device_pair.status`



- Method/path: `GET /v1/device/pair/status`
- Family: `device`
- Input schema: `focusa.device_pair_status.request.v1`
- Output schema: `focusa.device_pair_status.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `device:read`
- Confirmation: `none`
- Idempotency key required: `false`
- Receipt required: `false`
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
