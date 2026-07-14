# focusa_preload_doctor

Diagnose AgentBootstrapPacket delivery readiness and recovery steps.

## CLI

```sh
focusa preload doctor --profile rules_and_context
```

## API

`POST /v1/preload/doctor`

## Arguments

- `profile` (optional)
- bounded project/session scope fields (optional)

## Output

A `tool_result_v1` diagnostic envelope with bounded checks and next tools.

## Safety

Read-state only. Failures expose `failure_class` and use `FOCUSA_PRELOAD_FAIL`.

## Evidence

Spec 111 §§9–11.
