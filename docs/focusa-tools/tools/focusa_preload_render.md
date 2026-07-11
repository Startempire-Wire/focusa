# focusa_preload_render

Render an AgentBootstrapPacket for a selected profile without writing to disk.

## CLI

```sh
focusa preload render --profile rules_and_context
```

## API

`POST /v1/preload/render`

## Arguments

- `profile` (optional)
- bounded project/session scope fields (optional)

## Output

A `tool_result_v1` envelope containing the rendered bootstrap packet.

## Safety

Read-state only. Failures use `FOCUSA_PRELOAD_FAIL`.

## Evidence

Spec 111 §§9–11.
