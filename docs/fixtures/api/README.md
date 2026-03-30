# Focusa API fixture payloads

Canonical request samples used for docs and contract probes.

## Files
- `turn_start.request.json` → `POST /v1/turn/start`
- `prompt_assemble.request.json` → `POST /v1/prompt/assemble`
- `gate_signal.request.json` → `POST /v1/gate/signal`
- `ecs_store.request.json` → `POST /v1/ecs/store`
- `memory_reinforce.request.json` → `POST /v1/memory/procedural/reinforce`
- `reflect_run.request.json` → `POST /v1/reflect/run`
- `reflect_scheduler_update.request.json` → `POST /v1/reflect/scheduler`
- `reflect_scheduler_tick.request.json` → `POST /v1/reflect/scheduler/tick`

## Smoke run
```bash
BASE_URL=http://127.0.0.1:8787
for f in turn_start prompt_assemble gate_signal ecs_store memory_reinforce reflect_run reflect_scheduler_update reflect_scheduler_tick; do
  endpoint=""
  case "$f" in
    turn_start) endpoint="/v1/turn/start" ;;
    prompt_assemble) endpoint="/v1/prompt/assemble" ;;
    gate_signal) endpoint="/v1/gate/signal" ;;
    ecs_store) endpoint="/v1/ecs/store" ;;
    memory_reinforce) endpoint="/v1/memory/procedural/reinforce" ;;
    reflect_run) endpoint="/v1/reflect/run" ;;
    reflect_scheduler_update) endpoint="/v1/reflect/scheduler" ;;
    reflect_scheduler_tick) endpoint="/v1/reflect/scheduler/tick" ;;
  esac
  status=$(curl -s -o /tmp/focusa-fixture.out -w "%{http_code}" \
    -H 'content-type: application/json' \
    --data @"docs/fixtures/api/${f}.request.json" \
    "$BASE_URL$endpoint")
  echo "$f => $status"
  test "$status" != "422" || exit 1
done
```
