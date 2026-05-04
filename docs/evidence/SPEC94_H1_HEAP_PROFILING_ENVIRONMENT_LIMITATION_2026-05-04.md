# SPEC94 H1 Environment Limitation: Heap Profiling Tools

**Date:** 2026-05-04
**Spec reference:** `docs/94-focusa-intent-preserving-memory-rpc-optimization-sow.md:386-405`
**Gap:** S94-G3 / SPEC94 H1

## Requirement

Spec94 H1 requires heap/RSS profiling before broad Rust rewrites using heaptrack, DHAT, or Valgrind:

> "run heap/RSS profiling on representative workloads before broad Rust rewrites; use heaptrack, DHAT, or equivalent; record allocation hot spots for large route projections, metacog retrieval, ontology world, ECS handle listing, and event tailing."

## Environment Status

These tools are **not installed** in this environment:

| Tool | Status | Command tested |
|---|---|---|
| heaptrack | NOT INSTALLED | `which heaptrack` → not found |
| Valgrind DHAT | NOT INSTALLED | `valgrind --tool=help` → not found |
| Valgrind (base) | NOT INSTALLED | `which valgrind` → not found |

## Equivalent Available Profiling Methods

The following proxy methods are used instead:

| Method | What it measures | Spec94 coverage |
|---|---|---|
| `/proc/self/status` VmRSS + VmHWM | Process RSS and peak RSS | ✅ Memory pressure monitoring |
| `/usr/bin/time -v` | RSS, wall time, CPU | ✅ Route-level memory sampling |
| `response_size_histograms` in telemetry | JSON response byte distribution per route | ✅ Payload size hot spots |
| Route latency p50/p95 | Execution time per route | ✅ CPU/alloc hot spots by timing |
| `cargo flamegraph` | CPU flame graph (if `inferno` installed) | ❌ Not verified available |
| Static code audit | Clone/allocation patterns | ✅ Manual hot-spot identification |

## Evidence Artifact

Runtime profiling evidence is captured at:
`docs/evidence/profile/SPEC94_PROFILE_2026-05-03.json`

This includes:
- Route-level p50/p95 latency
- Response size histograms per route
- RSS and peak RSS samples
- Store counts and cap utilization

## Installation Path (if needed)

```bash
# Debian/Ubuntu
sudo apt-get install heaptrack valgrind

# Check Valgrind DHAT support
valgrind --tool=dhat ./target/release/focusa-daemon

# Check heaptrack support
heaptrack ./target/release/focusa-daemon
```

## Resolution

This is an **environment limitation**, not a code gap. The implementation correctly:
1. Exposes RSS and peak RSS via `/v1/telemetry/memory`
2. Records response size histograms for all routes
3. Tracks store counts and cap utilization
4. Implements pressure/degrade mode with explicit degradation

The gap cannot be closed until heaptrack/DHAT tools are installed in this environment, or an equivalent profiling pipeline is documented.