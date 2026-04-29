# Spec91 Live Tool Contract Proof — 2026-04-28

## Scope

Operator requested a new granular spec, bead decomposition, implementation start, and documentation updates. Spec91 adds live runtime proof that the current daemon serves the same tool contracts as the repository registry.

## Added

- Spec: `docs/91-live-tool-contract-proof-harness-spec.md`
- Live proof script: `scripts/prove-focusa-tool-contracts-live.mjs`
- Read-only safe fixture mode: `node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures`
- User docs: `docs/current/LIVE_TOOL_CONTRACT_PROOF.md`
- README/docs index/changelog links

## Beads

Root epic: `focusa-8e34`

Subtasks created:

- `focusa-8e34.1` — Author Spec91 live proof harness spec
- `focusa-8e34.2` — Implement live tool contract proof script
- `focusa-8e34.3` — Document live proof usage
- `focusa-8e34.4` — Update README docs changelog for Spec91
- `focusa-8e34.5` — Record Spec91 proof evidence and secret scan
- `focusa-8e34.6` — Add safe fixture mode for representative Pi tool probes

## Validation

Static Spec90 validation:

```text
Spec90 tool contracts: passed
tools=43 contracts=43
by_family={"focus_state":10,"work_loop":6,"diagnostics_hygiene":4,"workpoint":5,"tree_lineage":9,"metacognition":9}
```

Live Spec91 proof:

```text
Spec91 live tool contract proof: passed
health=ok version=0.1.0
static=spec90.tool_contracts.v1 count=43
live=spec90.tool_contracts.v1 count=43
payload_equal=true
checked_endpoints=/v1/health,/v1/ontology/tool-contracts
```

Live Spec91 safe fixture proof:

```text
Spec91 live tool contract proof: passed
health=ok version=0.1.0
static=spec90.tool_contracts.v1 count=43
live=spec90.tool_contracts.v1 count=43
payload_equal=true
checked_endpoints=/v1/health,/v1/ontology/tool-contracts,/v1/workpoint/current,/v1/work-loop/status,/v1/lineage/head,/v1/metacognition/reflections/recent,/v1/focus/frame/current
fixture_checks=workpoint:passed,work_loop:passed,tree_lineage:passed,metacognition:passed,focus_state:passed
```

TypeScript:

```bash
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
```

Result: passed.

Rust API check:

```bash
cargo check -p focusa-api --target-dir /tmp/focusa-cargo-target
```

Result: passed.

Secret scan:

```text
✅ No secrets found in docs/91-live-tool-contract-proof-harness-spec.md
✅ No secrets found in docs/current/LIVE_TOOL_CONTRACT_PROOF.md
✅ No secrets found in scripts/prove-focusa-tool-contracts-live.mjs
✅ No secrets found in README.md
✅ No secrets found in docs/README.md
✅ No secrets found in CHANGELOG.md
```

## Remaining follow-up

No Spec91 bead remains open. Future work may add isolated write fixtures, but the current safe fixture mode is read-only and complete for representative live family probes.
