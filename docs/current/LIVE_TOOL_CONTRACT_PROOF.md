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

## Expected current result

```text
Spec91 live tool contract proof: passed
health=ok version=0.1.0
static=spec90.tool_contracts.v1 count=43
live=spec90.tool_contracts.v1 count=43
payload_equal=true
checked_endpoints=/v1/health,/v1/ontology/tool-contracts
```

## Safe fixture expected result

```text
Spec91 live tool contract proof: passed
health=ok version=0.1.0
static=spec90.tool_contracts.v1 count=43
live=spec90.tool_contracts.v1 count=43
payload_equal=true
checked_endpoints=/v1/health,/v1/ontology/tool-contracts,/v1/workpoint/current,/v1/work-loop/status,/v1/lineage/head,/v1/metacognition/reflections/recent,/v1/focus/frame/current
fixture_checks=workpoint:passed,work_loop:passed,tree_lineage:passed,metacognition:passed,focus_state:passed
```

## Safety

Default proof and safe fixture mode are read-only and local. They do not mutate Focus State, Workpoints, Work-loop state, metacognition state, or user data.
