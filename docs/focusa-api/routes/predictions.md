# Focusa predictions Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.prediction.evaluate`



- Method/path: `POST /v1/predictions/evaluate`
- Family: `prediction`
- Input schema: `focusa.prediction_evaluate.request.v1`
- Output schema: `focusa.prediction_evaluate.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `prediction:write`
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

## `focusa.prediction.recent`



- Method/path: `GET /v1/predictions/recent`
- Family: `prediction`
- Input schema: `focusa.prediction_recent.request.v1`
- Output schema: `focusa.prediction_recent.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `prediction:read`
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

## `focusa.prediction.record`



- Method/path: `POST /v1/predictions/record`
- Family: `prediction`
- Input schema: `focusa.prediction_record.request.v1`
- Output schema: `focusa.prediction_record.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `prediction:write`
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
