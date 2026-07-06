# Spec 117 Help Overlay proof — 2026-07-05

Scope: focusa-117-arch.4 Help Overlay.

## Headless proof
```json
{"title":"Focusa Mission Deck","help_overlay":{"toggle":["h","?"],"topic_count":5,"topics":["Workpoint — the saved mission state: objective, current action, proof, next action.","Evidence — a test, file, screenshot, command output, or URL proving the claim.","Recall — Focusa remembering the mission after compaction, restart, or handoff.","Mission Ladder — high-level goal → current milestone → next safe action.","Authority badges — canonical means safe to act; advisory means review first; blocked means stop and rebind."]}}
```

## Tests/gates
- cargo test --release -p focusa-tui -- help_overlay: PASS (1 test)
- tests/spec_focusa_117_help_overlay_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
- cargo build --release -p focusa-tui: PASS
