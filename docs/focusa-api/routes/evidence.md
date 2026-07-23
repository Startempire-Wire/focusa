# Focusa evidence Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.evidence.capture`

- Method/path: `POST /v1/evidence/capture`
- Family: `evidence`
- Input schema: `focusa.evidence_capture.request.v1`
- Output schema: `focusa.evidence_capture.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `evidence:write`
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
