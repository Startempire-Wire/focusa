# Focusa Awareness Plugin

Standalone OpenClaw-compatible plugin that injects a Focusa Utility Card before agent start.

## Build

```bash
cd apps/focusa-awareness
bun run check
bun run build
```

The plugin intentionally declares a minimal local host API interface instead of importing `openclaw/plugin-sdk`; this keeps the package buildable in this repository while preserving the expected runtime hook shape.

## Runtime config

See `openclaw.plugin.json` for supported config fields:

- `focusaUrl` — Focusa daemon URL, default `http://127.0.0.1:8787`.
- `adapterId`, `workspaceId`, `agentId`, `operatorId` — adapter identity echoed into the awareness card.
- `projectRoot` — verified project root passed as `project_root`.
- `continuityId` — stable logical workstream id passed as `continuity_id`.
- `timeoutMs` — awareness request timeout.
- `enabled` — disable injection when false.

## Scope and authority contract

Adapter surfaces must preserve Focusa scope semantics:

1. Pass `project_root` and `continuity_id` whenever known.
2. Treat `project_root + continuity_id` as the Workpoint authority boundary.
3. Treat Workpoint resume/checkpoint as immediate action authority.
4. Treat Trajectory and awareness cards as advisory unless the response explicitly says `canonical=true` for the verified scope.
5. Preserve labels such as `canonical`, `advisory`, `degraded`, `stale`, `scope_status`, and `scope_conflict_reason` in any UI or log surface.

If `continuityId` is not configured, the card is allowed to be advisory/unbound. Do not promote it to canonical task authority.

## Tool-result pass-through expectations

The `/v1/awareness/card` response may include structured status fields and `details.tool_result_v1`-style payloads. OpenClaw adapters should pass these through without flattening away:

- `status`, `failure_class`, `canonical`, `degraded`, `stale`
- `scope.project_root`, `scope.continuity_id`, `scope.scope_status`
- `evidence_refs`, `side_effects`, `next_tools`
- `retry.safe` and `retry.posture`

This aligns the plugin with `docs/current/AGENT_ADAPTER_CONTRACT.md`: UI copy may be concise, but machine-readable scope/authority labels must remain inspectable.

## Degraded fallback

If the daemon is unreachable, the plugin injects a degraded fallback card. The fallback includes configured `project_root` and `continuity_id` but must be treated as `cognition_degraded=true`; fetch a canonical Workpoint resume before risky continuation.
