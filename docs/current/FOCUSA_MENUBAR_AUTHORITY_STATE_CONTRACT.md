# Focusa Menubar Authority State Contract

**Status:** implemented for `focusa-877z.8.7`.

Menubar is a read/display surface. It must never mint canonical authority; it mirrors API/Pi `tool_result_v1` envelopes and Workpoint packets.

## Required rendered authority fields

- `canonical`
- `advisory` / `advisory_only`
- `degraded`
- `stale`
- `scope.scope_status`
- `scope.scope_source`
- `side_effects`
- `evidence_refs`
- `next_tools`
- `failure_class`

## Chip semantics

- `canonical=true` → positive authority chip.
- `canonical=false` → non-canonical chip.
- `advisory=true` → advisory chip.
- `degraded=true` or `stale=true` → warning chip.
- `scope_status=verified` → verified scope chip.
- other `scope_status` → scope warning chip.

## Component mapping

- `apps/menubar/src/lib/api.ts` normalizes `tool_result_v1` fields.
- `WorkpointPeek.svelte` renders continuation authority, stale/degraded/advisory/scope chips.
- `ProofPeek.svelte` renders evidence refs, side effects, and snapshot history-only posture.

## Rule

Menubar buttons/cards may call scoped routes, but UI display state is not canonical state; canonicality comes from reducer-backed API/Workpoint envelopes only.
