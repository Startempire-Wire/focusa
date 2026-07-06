# Spec 117 Blazing-Fast TUI Startup/Progressive Loading proof — 2026-07-06

Scope: focusa-117-arch.25 blazing-fast TUI startup and progressive loading.

## Headless proof
```json
{"title":"Focusa Mission Deck","startup":{"first_paint_budget_ms":200,"progressive_loading_plan":["deck_home: render from local defaults","next_safe_action: render from local defaults","mission_ladder: render unavailable/recovery state immediately","proof_meter: render none | linked | verified from cached fetch","scope_badge: render unbound | advisory | canonical from cached fetch","walkthroughs/recall: lazy on tab focus","tab_data: lazy after first paint"],"shell_render_phases":["frame_zero_local_defaults","headless_metadata_dispatched","daemon_state_progressive_fetch","secondary_panels_lazy_load","interactive_loop"]}}
```

## Tests/gates
- cargo test --release -p focusa-tui -- startup_perf: PASS (3 tests)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_tui_startup_perf_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
