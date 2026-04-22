# SPEC83 Runtime Evidence — Pi × Focusa RPC Efficiency

**Date:** 2026-04-21  
**Spec:** `docs/83-pi-focusa-rpc-efficiency-spec.md`  
**Epic:** `focusa-pyc1`

## Executed validation commands

```bash
# A/B/C benchmark harness
timeout 180 bash tests/spec83_perf_benchmark_runtime_test.sh

# TypeScript contract
timeout 120 apps/pi-extension/node_modules/.bin/tsc -p apps/pi-extension/tsconfig.json

# Rust contract + targeted test
timeout 240 cargo check -p focusa-api -p focusa-cli
timeout 240 cargo test -p focusa-api supervisor_driver_gate_respects_loop_status -- --nocapture

# Live status projection proof
timeout 20 curl -sS http://127.0.0.1:8787/v1/status | jq '{runtime_perf,runtime_process,runtime_memory}'
```

## Benchmark results (`/tmp/spec83_perf_metrics.json`)

- none (no extension): elapsed **2.99s**, maxrss **177252 KB**
- focusa event-driven: elapsed **5.31s**, maxrss **187840 KB**
- focusa polling: elapsed **6.85s**, maxrss **186896 KB**

Derived deltas:
- Focusa event-driven vs none: **+2.32s**
- polling vs event-driven: **+1.54s**

Focusa API latency during run:
- `/v1/status` p50 **3.19ms**, p95 **4.91ms**, max **5.51ms**

Interpretation:
- Event-driven bridge mode is measurably cheaper than polling mode.
- Polling adds measurable overhead and should remain opt-in.

## Runtime perf schema proof (`/v1/status`)

Observed runtime fields include:
- `runtime_perf.supervisor_ticks_total`
- `runtime_perf.driver_start_attempts`
- `runtime_perf.driver_stop_attempts`
- `runtime_perf.dispatch_attempts`
- `runtime_perf.dispatch_skipped_disallowed`
- `runtime_perf.dispatch_recovery_restarts`

This confirms Spec83-B observability is live in daemon status payload.

## Build/test status

- TypeScript compile: ✅ pass
- `cargo check -p focusa-api -p focusa-cli`: ✅ pass
- targeted server test `supervisor_driver_gate_respects_loop_status`: ✅ pass

## Summary

Spec83 A/B/C implementation is active and validated:
1. Event-driven bridge default with polling opt-in implemented.
2. Supervisor backpressure counters implemented and exposed.
3. Benchmark harness + evidence produced with measurable event-driven benefit over polling.
