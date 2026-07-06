# Spec 117 Proof Meter + Scope Badge proof — 2026-07-05

Scope: focusa-117-arch.11 Proof Meter And Scope Badge.

## Headless proof
```json
{"title":"Focusa Mission Deck","proof_meter_states":["none:[-----]","linked:[##---]","verified:[#####]"],"scope_badge_states":["canonical","advisory","blocked","unbound"]}
```

## Tests/gates
- cargo test --release -p focusa-tui -- proof_status: PASS (3 tests)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_proof_meter_scope_badge_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
