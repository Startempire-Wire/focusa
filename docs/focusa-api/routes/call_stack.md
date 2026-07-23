# Focusa call stack Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.call_stack.design`



- Method/path: `POST /v1/call-stack/design`
- Family: `call_stack`
- Input schema: `focusa.call_stack_design.request.v1`
- Output schema: `focusa.call_stack_design.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `call_stack:write`
- Confirmation: `none`
- Idempotency key required: `true`
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

## `focusa.call_stack.verify`



- Method/path: `POST /v1/call-stack/verify`
- Family: `call_stack`
- Input schema: `focusa.call_stack_verify.request.v1`
- Output schema: `focusa.call_stack_verify.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `call_stack:read`
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
