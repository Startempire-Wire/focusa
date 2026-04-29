# Focusa Hook Coverage

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

This page tracks current Pi extension hook coverage for agent-first polish, token telemetry, and workflow observability.

## Implemented hooks

Current Focusa Pi extension registers these hooks:

```text
agent_end
agent_start
after_provider_response
before_agent_start
before_provider_request
context
input
message_end
message_start
message_update
model_select
resources_discover
session_before_compact
session_before_fork
session_before_switch
session_before_tree
session_compact
session_fork
session_shutdown
session_start
session_switch
session_tree
tool_call
tool_execution_end
tool_execution_start
tool_execution_update
tool_result
turn_end
turn_start
```

## Spec92 hooks added in first implementation slice

- `resources_discover` — contributes Focusa skill paths safely.
- `agent_start` — records bounded agent-run start metadata.
- `message_start` / `message_end` — records bounded assistant message lifecycle metadata.
- `before_provider_request` — records bounded payload hash/size/token budget/cache eligibility metadata.
- `after_provider_response` — records bounded provider response status/header-key/size metadata.
- `tool_execution_start/update/end` — records bounded tool lifecycle timing and size metadata.
- `session_tree` — records branch navigation and Workpoint resume recommendation.

## Runtime inspection

Use the Pi tool doctor to inspect current in-memory hook/token telemetry:

```text
focusa_tool_doctor scope="spec92"
```

Expected details include:

```text
spec92.hook_records
spec92.hook_counts
spec92.token_records
spec92.latest_token
```

## Validation commands

```bash
cd /home/wirebot/focusa
rg -n 'pi\.on\(' apps/pi-extension/src -g'*.ts'
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
```

## Safety

Hook telemetry is bounded and does not store raw provider payloads by default. Provider payloads are represented by hash, size, message count, token estimate, budget class, and cache eligibility metadata.
