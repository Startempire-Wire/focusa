# Spec 117 Beginner Mode state machine proof — 2026-07-05

Scope: focusa-117-arch.3 Beginner Mode state machine.

## Headless proof
```json
{"title":"Focusa Mission Deck","default_tab":"DeckHome","beginner_mode_decision_tree":["disconnected","unbound","no_workpoint","no_evidence","resumable"],"tabs":["d:DeckHome","1:FocusState","2:FocusStack","3:Gate","4:Events","5:Metrics","6:Lineage","w:WorkLoop","7:Autonomy","8:Constitution","9:Telemetry","0:Rfm","p:Proposals","s:Skills","u:Uxp","x:Training"]}
```

## Tests/gates
- cargo test --release -p focusa-tui -- beginner_mode: PASS (4 tests)
- tests/spec_focusa_117_beginner_mode_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
- cargo build --release -p focusa-tui: PASS
