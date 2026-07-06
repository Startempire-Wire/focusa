# Spec 117 Workpoint Candidate Promotion proof — 2026-07-05

Scope: focusa-117-arch.14 Recall → Workpoint candidate promotion gated by Context Authority.

## Headless proof
```json
{"recall_tab":{"workpoint_candidate_promotion_flow":["recall_search","recall_deck_card","verify_project_root_and_continuity_id","context_authority_preflight","proof_check","render_workpoint_candidate","operator_approval","canonical_workpoint_checkpoint"],"workpoint_candidate_forbidden":["recall_direct_canonical_write","promotion_without_scope_verification","promotion_without_operator_approval","promotion_without_proof_or_explicit_gap"]}}
```

## Tests/gates
- cargo test --release -p focusa-tui -- workpoint_candidate: PASS (1 test)
- cargo build --release -p focusa-tui: PASS
- tests/spec_focusa_117_workpoint_candidate_promotion_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
