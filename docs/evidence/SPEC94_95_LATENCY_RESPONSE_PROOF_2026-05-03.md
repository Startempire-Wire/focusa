# Spec94/95 Latency, Response Size, and RSS Proof — 2026-05-03

> Superseded/extended by `docs/evidence/SPEC94_95_RUNTIME_GATES_2026-05-03.md`, which adds duplicate-daemon, pressure-mode, plateau, Spec90/91, and live intelligence gate coverage.

## Scope

Ephemeral daemon built from current workspace changes with isolated data dir:

```bash
CARGO_TARGET_DIR=/tmp/focusa-target cargo build -p focusa-api
FOCUSA_BIND=127.0.0.1:18787 FOCUSA_DATA_DIR=/tmp/focusa-spec94-* /tmp/focusa-target/debug/focusa-daemon
```

Each route was sampled 8 times using Python `urllib`; p95 uses nearest-rank over the sorted sample set. This proof covers the default bounded read path and Spec95 low-latency ontology intelligence routes.

## Results

| Route | OK | p50 ms | p95 ms | max bytes |
|---|---:|---:|---:|---:|
| `/v1/ontology/world` | true | 9.11 | 10.20 | 7,290 |
| `/v1/ecs/handles` | true | 2.16 | 2.43 | 429 |
| `/v1/memory/semantic` | true | 2.84 | 3.05 | 430 |
| `/v1/work-loop/status?summary_only=true` | true | 2.67 | 5.60 | 2,016 |
| `/v1/telemetry/events` | true | 2.92 | 3.12 | 204 |
| `/v1/telemetry/productivity` | true | 2.43 | 3.26 | 276 |
| `/v1/telemetry/autonomy` | true | 2.24 | 3.57 | 354 |
| `/v1/ontology/context` | true | 11.93 | 14.18 | 8,003 |
| `/v1/ontology/working-set` | true | 3.52 | 3.65 | 3,999 |
| `/v1/ontology/affordances` | true | 5.77 | 7.73 | 3,012 |

Process sample during run:

```text
RSS: 14,208 KiB
VSZ: 735,656 KiB
COMMAND: focusa-daemon
```

## Gate interpretation

- Spec94 default large-read payloads are bounded under 10 KB in the isolated proof workload.
- Spec95 p95 budgets are met after warm-up: context <50 ms, working-set <50 ms, affordances <75 ms.
- Full-world/export behavior remains explicit through `include_full_payload=true` and route rehydrate metadata.
- Plateau and deeper runtime gate evidence is recorded in `SPEC94_95_RUNTIME_GATES_2026-05-03.md`.
