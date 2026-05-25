# Public Docs Spec97 Refresh Evidence — 2026-05-25

## Scope

Public-facing Focusa docs were refreshed after Spec97/Spec96/low-resource uplifts so current onboarding, runtime status, API, tool contracts, troubleshooting, release proof, and agent-awareness docs describe the implemented build.

## Refreshed topics

- 59 current `focusa_*` tools, including `focusa_reflex_primitives`.
- Spec97 Reflex Primitives: read-only registry, direct `GET /v1/reflex/primitives`, `surface=reflex_primitives` traversal, API/Pi `reflex_suggestions`, ontology classes/actions, and runtime dogfood.
- Spec96 Focus current-focus ↔ Trajectory short-term-goal sync.
- Low-resource hardening: narrowed async/HTTP features, compact/capped Pi outputs, route-tier timeouts, explicit cold payload opt-in.
- Live contract proof count: static/live 59/59 with `payload_equal=true` after daemon rebuild/restart.

## Validation

```bash
node scripts/validate-focusa-tool-contracts.mjs --json
node scripts/validate-docs-runtime-parity.mjs
node scripts/validate-agent-awareness.mjs
tests/spec97_api_native_reflex_and_ontology_static_test.sh
tests/spec97_reflex_direct_route_static_test.sh
tests/spec97_reflex_runtime_dogfood_test.sh  # temporarily activates/restores LowMem for degraded-envelope proof
git diff --check
```

All commands passed during the refresh.
