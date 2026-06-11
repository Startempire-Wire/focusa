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
health=ok version=0.9.14-dev
static=spec90.tool_contracts.v1 count=79
live=spec90.tool_contracts.v1 count=79
payload_equal=true
checked_endpoints=/v1/health,/v1/ontology/tool-contracts
```

## Safe fixture expected result

```text
Spec91 live tool contract proof: passed
health=ok version=0.9.14-dev
static=spec90.tool_contracts.v1 count=79
live=spec90.tool_contracts.v1 count=79
payload_equal=true
checked_endpoints=/v1/health,/v1/ontology/tool-contracts,/v1/workpoint/current,/v1/work-loop/status?summary_only=true,/v1/lineage/head,/v1/metacognition/reflections/recent,/v1/focus/frame/current
fixture_checks=workpoint:passed,work_loop:passed,tree_lineage:passed,metacognition:passed,focus_state:passed
```

## Stale daemon note

After static registry edits and before an approved daemon rebuild/restart, the proof can fail with `payload_equal=false` while all safe fixture endpoint checks pass. Treat that as daemon read-model staleness, not tool-contract invalidity; run static validation until restart is approved.

## Safety

Default proof and safe fixture mode are read-only and local. They do not mutate Focus State, Workpoints, Work-loop state, metacognition state, or user data.

## Latest Spec97/reflex proof

- Static/live tool contract registry count: 79.
- `payload_equal=true` after rebuilt daemon restart.
- Safe hot probes include `GET /v1/trajectory/view`; cold `/v1/lineage/tree` is skipped by low-memory audit unless explicitly opted in.
- Reflex proof includes `GET /v1/reflex/primitives?family=recovery&limit=2`, `surface=reflex_primitives` traversal, and degraded/full-payload recovery suggestions.
- Evidence: `docs/evidence/SPEC97_REFLEX_DIRECT_API_LIVE_PROOF_2026-05-25.md`; `tests/spec97_reflex_runtime_dogfood_test.sh`; `/tmp/spec97-live-contract-proof.json` when generated locally.
