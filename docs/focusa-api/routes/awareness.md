# Focusa awareness Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.awareness.packet`

- Method/path: `POST /v1/awareness/packet`
- Family: `awareness`
- Input schema: `focusa.awareness_packet.request.v1`
- Output schema: `focusa.awareness_packet.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `awareness:read`
- Confirmation: `none`
- Idempotency key required: `false`
- Receipt required: `true`
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
