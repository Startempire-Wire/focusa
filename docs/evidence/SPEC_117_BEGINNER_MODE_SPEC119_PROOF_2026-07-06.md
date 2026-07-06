# Spec 117 Beginner Mode Spec 119 alignment proof — 2026-07-06

Scope: focusa-117-arch.34 align Beginner Mode (.3) with Spec 119 §30 affordance reality.

## Changes
- AFFORDANCE_REALITY_BY_BEGINNER_STATE const map: disconnected→unavailable, unbound/no_workpoint/no_evidence→limited, resumable→possible
- affordance_reality_for(state) helper returning &'static str
- main.rs headless proof exposes beginner_mode_affordance_by_state as JSON array
- New unit test affordance_reality_matches_spec119 asserts all 5 mappings

## Tests/gates
- cargo test --release -p focusa-tui -- beginner_mode: PASS (5 tests, +1)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_beginner_mode_spec119_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
