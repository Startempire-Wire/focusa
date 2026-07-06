# Spec 117 Lightweight Advisory Recall Tab proof — 2026-07-05

Scope: focusa-117-arch.12 Recall Tab. Full Recall spec expansion tracked separately in focusa-117-arch.29 and is not a blocker for this lightweight advisory surface.

## Headless proof
```json
{"title":"Focusa Mission Deck","recall_tab":{"hotkey":"/","source_count":8,"card_fields":14,"authority_rule":"Recall is advisory: inspect/verify first; canonical Workpoint promotion requires operator approval."},"keybindings":{"recall":["/"]}}
```

## Tests/gates
- cargo test --release -p focusa-tui -- recall: PASS (2 tests)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_recall_tab_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
