# focusa_awareness_packet

**Family:** awareness  
**Schema:** focusa.awareness_packet.v1  
**Surface:** Spec108 awareness rendering substrate  
**Parity:** API only (`GET /v1/awareness/packet`)

## Purpose

Render a surface-aware `AwarenessPacket` with DVS-scored visible lines, suppressed lines, metadata, next_tools, and recovery_tools. The packet distills Focusa cognitive state into a compact, ranked view appropriate for the current session context.

## When to use

- On session reload: surface=`reload`
- After compaction: surface=`post_compaction`
- During tool guidance: surface=`tool_guidance`
- On warning/error: surface=`warning`
- During UIAI bridge ops: surface=`uiai_bridge`

## Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `surface` | string | `reload` | Awareness surface: `reload`, `post_compaction`, `warning`, `tool_guidance`, `uiai_bridge` |

## Result envelope

Returns `tool_result_v1` with:

- `schema`: `"focusa.awareness_packet.v1"`
- `mode`: `"minimal" | "standard" | "rich" | "onboarding"`
- `surface`: the requested surface
- `visible_lines`: top DVS-scored lines (shown to operator)
- `suppressed_lines`: lower-priority lines (hidden but available)
- `metadata`: DVS cutoff, counts, confidence, freshness score, authority score
- `next_tools`: recommended next tools
- `recovery_tools`: recovery options if degraded

## Guardrails

- AwarenessPacket is advisory; it does not override Workpoint authority.
- Suppressed lines are available but not shown by default.
- Confidence is derived from authority score (high ≥80, medium ≥50, low <50).

## Next tools

- `focusa_workpoint_resume` — canonical continuation
- `focusa_trajectory_view` — goal/state orientation
- `focusa_tool_doctor` — if packet is degraded

## Evidence

- Rust implementation: `crates/focusa-core/src/awareness.rs`
- TypeScript port: `apps/pi-extension/src/awareness-substrate.ts`
- API routes: `GET /v1/awareness/packet`, `GET /v1/awareness/packet/{surface}`
- Static test: `tests/spec108_awareness_substrate_static_test.py`
