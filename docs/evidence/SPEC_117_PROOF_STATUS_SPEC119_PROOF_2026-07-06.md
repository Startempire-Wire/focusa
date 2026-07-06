# Spec 117 Proof Status Spec 119 alignment proof — 2026-07-06

Scope: focusa-117-arch.32 align Proof Meter/Scope Badge (.11) with Spec 119 §30 (Affordance Reality) + §31 (Governing Priors/Precedence).

## Changes
- ProofMeter gained `affordance_reality: &'static str` field (possible/limited/unavailable)
- ScopeBadge gained `precedence_frame: &'static str` field (project/authority/operator)
- All 5 scope badge + 3 proof meter literals now populate new fields
- main.rs headless proof exposes affordance_reality_states + precedence_frames
- 2 new unit tests: affordance_reality_matches_status, scope_badge_carries_precedence_frame

## Tests/gates
- cargo test --release -p focusa-tui -- proof_status: PASS (5 tests, +2)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_proof_status_spec119_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
