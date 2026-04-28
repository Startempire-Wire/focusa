# Spec88 Rollout Gate — Operator Docs and Skill Update

Date: 2026-04-28
Bead: `focusa-a2w2.11`
Spec: `docs/88-ontology-backed-workpoint-continuity.md`

## Rollout Surface

- `docs/88-ontology-backed-workpoint-continuity.md` status changed to implemented through Phase 10 rollout gate.
- `docs/44-pi-focusa-integration-spec.md` includes section 39, the Workpoint Continuity Operator Contract.
- `apps/pi-extension/skills/focusa/SKILL.md` documents Workpoint checkpoint/resume tools, degraded semantics, drift interpretation, and recovery examples.
- Installed Pi skill synced to `/root/.pi/skills/focusa/SKILL.md` for immediate local use.

## Operator Contract

Operators and agents should use this sequence:

1. Before compaction, fork, model switch, overflow retry, or risky branch: `focusa_workpoint_checkpoint`.
2. After compaction, resume, overflow, model switch, or uncertainty: `focusa_workpoint_resume`.
3. If `canonical: false`, treat packet as degraded fallback and avoid silent promotion.
4. If drift warning appears, resume the packet `next_slice` unless operator steering overrides it.
5. If Focusa is offline, use the smallest bounded local next action and checkpoint after service recovery.

## Rollout Gate Commands

```bash
./tests/spec88_workpoint_golden_eval_contract_test.sh
./tests/spec87_impl_tool_desirability_test.sh
./tests/spec81_impl_pi_extension_runtime_contract_test.sh
./tests/scope_routing_regression_eval.sh
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
cargo check -p focusa-cli -p focusa-api -p focusa-core --target-dir /tmp/focusa-cargo-target
cargo test -p focusa-core workpoint --target-dir /tmp/focusa-cargo-target
```

## Acceptance

Rollout is allowed when the docs/skill contract is present, first-class Workpoint tools are visible, degraded fallback semantics are documented, drift recovery is documented, and gate commands pass.
