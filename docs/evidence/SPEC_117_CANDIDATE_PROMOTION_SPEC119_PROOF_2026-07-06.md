# Spec 117 Candidate Promotion Spec 119 alignment proof — 2026-07-06

Scope: focusa-117-arch.31 align Workpoint Candidate Promotion (.14) with Spec 119 §7.11 preview-before-commit.

## Changes
- WorkpointCandidatePromotion gained `preview_state: &'static str` field
- New `is_preview_only()` method returns true until operator approval is recorded
- Promotion flow already enforces: render_workpoint_candidate -> operator_approval -> canonical_workpoint_checkpoint
- WorkpointCandidatePromotion::recall_default() returns preview_only_until_operator_approval
- main.rs headless proof exposes workpoint_candidate_preview_state + workpoint_candidate_preview_only
- New unit test `workpoint_candidate_preview_state_blocks_canonical_write` asserts preview invariant

## Tests/gates
- cargo test --release -p focusa-tui -- recall: PASS (5 tests, +1)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_candidate_promotion_spec119_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
