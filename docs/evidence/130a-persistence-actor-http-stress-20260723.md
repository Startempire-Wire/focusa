# Spec130A Persistence Actor HTTP Stress Proof — 2026-07-23

## Scope

Isolated Focusa daemon on `127.0.0.1:28787` with a temporary `FOCUSA_HOME` and temporary verified project marker. Production daemon and data were not touched.

## Workload

- 300 sequential `/v1/session/start` mutations while 250 `/v1/health` probes ran.
- Persistence queue capacity: 64.
- Process was killed after the queue drained, then restarted against the same SQLite/WAL directory.

## Result

```json
{
  "health_requests": 250,
  "p95_ms": 1.627,
  "max_ms": 2.987,
  "persistence": {
    "batches_total": 3,
    "failures_total": 0,
    "last_write_duration_ms": 1,
    "max_write_duration_ms": 11,
    "queue_depth": 0,
    "queue_depth_max": 2,
    "requests_coalesced_total": 1,
    "saturation_total": 0,
    "snapshot_bytes": 143360,
    "wal_bytes": 461472
  },
  "restart_wal_recovery": "PASS"
}
```

## Additional proof

`runtime::persistence_actor::tests` passed both tests:

1. ordinary snapshots coalesce and a checkpoint acknowledgement reloads the exact final state;
2. multi-megabyte serialization/write does not stall a Tokio timer.

Static guard: `tests/spec130a_persistence_actor_static_test.sh` rejects direct SQLite snapshot/event writes from daemon/API route hot paths and requires queue/write/WAL telemetry on `/v1/health`.
