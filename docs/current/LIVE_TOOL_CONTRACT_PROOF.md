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

## What it verifies

- Static Spec90 registry validation passes.
- Local daemon `/v1/health` is reachable and healthy.
- Live `GET /v1/ontology/tool-contracts` returns `spec90.tool_contracts.v1`.
- Live contract count equals static registry count.
- Live contract names exactly match static registry names.
- Live payload canonically equals `docs/current/focusa-tool-contracts.json`.
- API reference includes `/v1/ontology/tool-contracts`.

## Expected current result

```text
Spec91 live tool contract proof: passed
health=ok version=0.1.0
static=spec90.tool_contracts.v1 count=43
live=spec90.tool_contracts.v1 count=43
payload_equal=true
checked_endpoints=/v1/health,/v1/ontology/tool-contracts
```

## Safety

Default proof is read-only and local. It does not mutate Focus State, Workpoints, Work-loop state, metacognition state, or user data.
