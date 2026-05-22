# Live Tool Contract Proof

**Spec:** [`docs/91-live-tool-contract-proof-harness-spec.md`](../91-live-tool-contract-proof-harness-spec.md)

Spec91 proves the running local Focusa daemon is serving the same tool contract registry that the repository defines.

## Command

```bash
node scripts/prove-focusa-tool-contracts-live.mjs
```

Machine-readable mode:

```bash
node scripts/prove-focusa-tool-contracts-live.mjs --json
```

Read-only safe fixture mode:

```bash
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures --json
```

## What it verifies

- Static Spec90 registry validation passes.
- Local daemon `/v1/health` is reachable and healthy.
- Live `GET /v1/ontology/tool-contracts` returns `spec90.tool_contracts.v1`.
- Live contract count equals static registry count.
- Live contract names exactly match static registry names.
- Live payload canonically equals `docs/current/focusa-tool-contracts.json`.
- API reference includes `/v1/ontology/tool-contracts`.
- With `--safe-fixtures`, representative read-only family probes pass for Workpoint, Work-loop, tree/lineage, metacognition, and Focus State.

## Expected current result after daemon reload

```text
Spec91 live tool contract proof: passed
health=ok version=0.1.0
static=spec90.tool_contracts.v1 count=53
live=spec90.tool_contracts.v1 count=53
payload_equal=true
checked_endpoints=/v1/health,/v1/ontology/tool-contracts
```

## Safe fixture expected result

```text
Spec91 live tool contract proof: passed
health=ok version=0.1.0
static=spec90.tool_contracts.v1 count=53
live=spec90.tool_contracts.v1 count=53
payload_equal=true
checked_endpoints=/v1/health,/v1/ontology/tool-contracts,/v1/workpoint/current,/v1/work-loop/status?summary_only=true,/v1/lineage/head,/v1/metacognition/reflections/recent,/v1/focus/frame/current
fixture_checks=workpoint:passed,work_loop:passed,tree_lineage:passed,metacognition:passed,focus_state:passed
```

## Stale daemon note

After static registry edits and before an approved daemon rebuild/restart, the proof can fail with `payload_equal=true` while all safe fixture endpoint checks pass. Treat that as daemon read-model staleness, not tool-contract invalidity; run static validation until restart is approved.

## Safety

Default proof and safe fixture mode are read-only and local. They do not mutate Focus State, Workpoints, Work-loop state, metacognition state, or user data.

## Latest trajectory-family proof

- Static/live tool contract registry count: 53.
- `payload_equal=true` after rebuilt daemon restart.
- Safe hot probes include `GET /v1/trajectory/view`; cold `/v1/lineage/tree` is skipped by low-memory audit unless explicitly opted in.
- Evidence: `/tmp/focusa-tool-audit-trajectory-family-final3.json`.
