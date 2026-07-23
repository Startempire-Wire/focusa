# Focusa metacognition Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.metacog.capture`

- Method/path: `POST /v1/metacognition/capture`
- Family: `metacognition`
- Input schema: `focusa.metacog_capture.request.v1`
- Output schema: `focusa.metacog_capture.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `metacog:write`
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

## `focusa.metacog.doctor`

- Method/path: `POST /v1/metacognition/doctor`
- Family: `metacognition`
- Input schema: `focusa.metacog_doctor.request.v1`
- Output schema: `focusa.metacog_doctor.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `metacog:read`
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

## `focusa.metacog.reflect`

- Method/path: `POST /v1/metacognition/reflect`
- Family: `metacognition`
- Input schema: `focusa.metacog_reflect.request.v1`
- Output schema: `focusa.metacog_reflect.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `metacog:read`
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

## `focusa.metacog.retrieve`

- Method/path: `POST /v1/metacognition/retrieve`
- Family: `metacognition`
- Input schema: `focusa.metacog_retrieve.request.v1`
- Output schema: `focusa.metacog_retrieve.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `metacog:read`
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
