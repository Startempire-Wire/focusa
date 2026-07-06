# Spec 117 TUI title/home proof — 2026-07-05

Scope: focusa-117-arch.2 TUI title and Deck Home.

## Headless proof
```json
{"schema":"focusa.tui_headless_self_test.v1","title":"Focusa Mission Deck","default_tab":"DeckHome","tabs":["d:DeckHome","1:FocusState","2:FocusStack","3:Gate","4:Events","5:Metrics","6:Lineage","w:WorkLoop","7:Autonomy","8:Constitution","9:Telemetry","0:Rfm","p:Proposals","s:Skills","u:Uxp","x:Training"]}
```

## Static/build gates
- tests/spec_focusa_117_tui_title_home_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
- cargo build --release -p focusa-tui: PASS
