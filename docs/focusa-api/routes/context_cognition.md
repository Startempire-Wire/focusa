# Focusa context cognition Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.context_cognition.curate`



- Method/path: `POST /v1/context-cognition/curate`
- Family: `context_cognition`
- Input schema: `focusa.context_cognition_curate.request.v1`
- Output schema: `focusa.context_cognition_curate.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `context:read`
- Confirmation: `none`
- Idempotency key required: `false`
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

## `focusa.context_cognition.packet`



- Method/path: `GET /v1/context-cognition/packet`
- Family: `context_cognition`
- Input schema: `focusa.context_cognition_packet.request.v1`
- Output schema: `focusa.context_cognition_packet.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `context:read`
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
