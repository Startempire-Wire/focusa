# Spec 117 RecallDeckCard Schema proof — 2026-07-05

Scope: focusa-117-arch.13 Recall Card Schema. Full Recall expansion remains focusa-117-arch.29.

## Headless proof
```json
{"recall_tab":{"memory_status_values":["active","stale","superseded","contradicted","noise","quarantined"],"scope_status_values":["current","same_project_other_continuity","other_project","global_advisory"],"proof_status_values":["none","linked","verified"],"allowed_use_values":["include","inspect_only","verify_first","exclude"]}}
```

## Tests/gates
- cargo test --release -p focusa-tui -- recall: PASS (3 tests)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_recall_card_schema_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
