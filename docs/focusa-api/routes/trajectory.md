# Focusa trajectory Agent Operations

Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.

These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.

## `focusa.trajectory.assess`

- Method/path: `POST /v1/trajectory/assess`
- Family: `trajectory`
- Input schema: `focusa.trajectory_assess.request.v1`
- Output schema: `focusa.trajectory_assess.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `trajectory:read`
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

## `focusa.trajectory.checkpoint`

- Method/path: `POST /v1/trajectory/checkpoint`
- Family: `trajectory`
- Input schema: `focusa.trajectory_checkpoint.request.v1`
- Output schema: `focusa.trajectory_checkpoint.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `trajectory:write`
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

## `focusa.trajectory.define_goal`

- Method/path: `POST /v1/trajectory/define-goal`
- Family: `trajectory`
- Input schema: `focusa.trajectory_define_goal.request.v1`
- Output schema: `focusa.trajectory_define_goal.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `trajectory:write`
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

## `focusa.trajectory.propose_workpoint`

- Method/path: `POST /v1/trajectory/propose-workpoint`
- Family: `trajectory`
- Input schema: `focusa.trajectory_propose_workpoint.request.v1`
- Output schema: `focusa.trajectory_propose_workpoint.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `trajectory:read`
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

## `focusa.trajectory.resume`

- Method/path: `POST /v1/trajectory/resume`
- Family: `trajectory`
- Input schema: `focusa.trajectory_resume.request.v1`
- Output schema: `focusa.trajectory_resume.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `trajectory:read`
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

## `focusa.trajectory.view`

- Method/path: `GET /v1/trajectory/view`
- Family: `trajectory`
- Input schema: `focusa.trajectory_view.request.v1`
- Output schema: `focusa.trajectory_view.response.v1`
- Error schema: `focusa.tool_result.v1`
- Permission scopes: `trajectory:read`
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
