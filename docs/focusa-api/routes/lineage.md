# Focusa lineage Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.lineage.head`



- Method/path: `GET /v1/lineage/head`
- Family: `lineage`
- Input schema: `focusa.lineage_head.request.v1`
- Output schema: `focusa.lineage_head.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `lineage:read`
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

## `focusa.lineage.tree`



- Method/path: `GET /v1/lineage/tree`
- Family: `lineage`
- Input schema: `focusa.lineage_tree.request.v1`
- Output schema: `focusa.lineage_tree.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `lineage:read`
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
