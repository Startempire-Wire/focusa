# focusa_preload_verify

Verify a scoped AgentBootstrapPacket before delivery or receipt creation.

## CLI

```sh
focusa preload verify --profile rules_and_context
```

## API

`POST /v1/preload/verify`

## Arguments

- `profile` (optional)
- bounded project/session scope fields (optional)

## Output

A `tool_result_v1` envelope with profile, integrity, and scope checks.

## Safety

Read-state only. Failures use `FOCUSA_PRELOAD_FAIL`.

## Evidence

Spec 111 §§9–11.
