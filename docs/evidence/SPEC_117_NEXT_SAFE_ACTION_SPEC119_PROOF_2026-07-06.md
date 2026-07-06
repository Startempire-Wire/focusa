# Spec 117 Next Safe Action Spec 119 alignment proof — 2026-07-06

Scope: focusa-117-arch.30 align Next Safe Action (.5) with Spec 119 §7.6 + §19.

## Changes
- RecoveryTool struct added (id, label, command)
- NextSafeAction gained `recovery_tools: &'static [RecoveryTool]` field
- All 6 next_safe_action branches (start_daemon, bind_project, create_workpoint, attach_evidence, resume_mission, review_scope_before_acting) supply 3 bounded recovery tools each
- HEADLESS_PROOF_RECOVERY_TOOL_CAP = 3 constant exposes cap to headless proof
- main.rs headless proof exposes next_safe_action_recovery_tool_cap
- New unit test `recovery_tools_are_bounded_to_three` enforces the cap

## Tests/gates
- cargo test --release -p focusa-tui -- next_safe_action: PASS (3 tests)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_next_safe_action_spec119_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
