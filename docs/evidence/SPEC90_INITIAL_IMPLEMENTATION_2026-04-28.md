# Spec90 Initial Implementation Evidence — 2026-04-28

## Scope

Operator requested a new spec, full granular decomposition into beads, beginning implementation, and documentation updates for making Focusa tools maximally integrated with core, CLI, API, and ontology.

## Added

- Spec: `docs/90-ontology-backed-tool-contracts-parity-spec.md`
- Contract registry: `apps/pi-extension/src/tool-contracts.ts`
- Validator: `scripts/validate-focusa-tool-contracts.mjs`
- Registry docs: `docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md`
- JSON registry projection: `docs/current/focusa-tool-contracts.json`
- Ontology API projection: `GET /v1/ontology/tool-contracts`
- Doctor integration: `focusa_tool_doctor` now reports contract totals/family coverage/scoped coverage/exemptions.
- README/docs index/changelog links for Spec90 and registry docs.

## Beads

Root epic: `focusa-9k3c`

Subtasks created:

- `focusa-9k3c.1` — Author Spec90 granular acceptance checklist
- `focusa-9k3c.2` — Implement Focusa tool contract registry foundation
- `focusa-9k3c.3` — Add Focusa tool contract validation script
- `focusa-9k3c.4` — Upgrade focusa_tool_doctor with contract coverage
- `focusa-9k3c.5` — Update docs README changelog for Spec90
- `focusa-9k3c.6` — Project contracts through ontology API follow-up
- `focusa-9k3c.7` — Enforce uniform tool_result_v1 follow-up
- `focusa-9k3c.8` — Prove full-chain live daemon doctor follow-up

## Validation

Contract validation:

```text
Spec90 tool contracts: passed
tools=43 contracts=43
by_family={"focus_state":10,"work_loop":6,"diagnostics_hygiene":4,"workpoint":5,"tree_lineage":9,"metacognition":9}
```

TypeScript validation:

```bash
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
```

Result: passed.

Rust API validation:

```bash
cargo check -p focusa-api --target-dir /tmp/focusa-cargo-target
cargo build --release -p focusa-api --bin focusa-daemon
systemctl restart focusa-daemon
curl -sS http://127.0.0.1:8787/v1/health | jq .
curl -sS http://127.0.0.1:8787/v1/ontology/tool-contracts | jq '.version, (.contracts|length)'
```

Result:

```text
active
{"ok":true,"version":"0.1.0"}
"spec90.tool_contracts.v1"
43
```

Secret scan:

```text
✅ No secrets found in /home/wirebot/focusa/docs/90-ontology-backed-tool-contracts-parity-spec.md
✅ No secrets found in /home/wirebot/focusa/docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md
✅ No secrets found in /home/wirebot/focusa/README.md
✅ No secrets found in /home/wirebot/focusa/CHANGELOG.md
```

## Result envelope proof

`node scripts/validate-focusa-tool-contracts.mjs` verifies the Pi extension installs `withToolResultEnvelope` before all 43 current `focusa_*` tool registrations, so every current registered tool is wrapped for `tool_result_v1` output.

## Workpoint proof

A canonical Workpoint checkpoint recorded this release slice after the daemon restart:

```text
WORKPOINT 019dd6f5-e973-7641-8504-491627b1815b: mission=Spec90 tool contract parity implementation; action=live_release_proof; next=record evidence, close remaining beads that passed, commit docs/evidence; canonical=true
```

## Remaining follow-ups

- Optional future hardening: add example runtime calls for each mutating tool class when safe fixtures exist.
