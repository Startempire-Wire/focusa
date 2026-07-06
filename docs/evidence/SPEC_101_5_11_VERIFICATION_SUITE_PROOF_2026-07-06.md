# Spec 101 §5.11.9 Verification Suite proof — 2026-07-06

Scope: focusa-29ew.5 verification suite tests per Spec 101 §5.11.9.

## Added
- 12 new tests covering §5.11.9 verification list:
  - defaults_safe_auto_with_text_passthrough_fallback
  - provider_policy_gate_blocks_unauthorized_provider (4 statuses)
  - provider_terms_hash_change_triggers_text_passthrough
  - image_input_rejected_falls_back
  - model_allowlist_required
  - verbatim_guard_protects_action_authority
  - active_blocker_kept_as_text
  - profitability_gate_required (min_net_savings >= 0.30)
  - recoverable_ref_required
  - canary_failed_text_passthrough
  - context_cognition_no_canonical_mutation (no commit/mutate/write sentinels)
  - focus_slice_no_raw_blob_default (full_payload_policy = cold_opt_in)

## Fix
- choose_fallback now checks all 4 gates (was missing profitability + recoverable_store)
- FALLBACK_CHAIN[6] set to FALLBACK_TEXT_PASSTHROUGH constant so all 4 gates resolve to the same fallback

## Tests/gates
- cargo test --release -p focusa-api -- routes::bloatgaurd_optical: PASS (25 tests)
- cargo build --release -p focusa-api: PASS
- tests/release_deploy_automation_static_test.sh: PASS
