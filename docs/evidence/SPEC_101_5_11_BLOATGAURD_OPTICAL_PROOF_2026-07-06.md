# Spec 101 §5.11 Bloatgaurd Optical Context Gateway scaffold proof — 2026-07-06

Scope: focusa-29ew.1 Spec 101 §5.11 scaffold + provider-policy-ledger model.

## Added
- `focusa.bloatgaurd_optical.v1` + `focusa.provider_policy_ledger.v1` schemas
- POSTURE constants: allowed | blocked | unknown | stale | needs_review
- Default-on safe_auto posture: min_net_savings=0.30, max_quality_regression=0, full_payload_policy=cold_opt_in, default_fallback=text_passthrough
- Image kinds allowed (old_dense_tool_output, old_command_logs, old_collapsed_history_after_checkpoint, large_non_current_tool_docs, large_structured_json_behind_rehydrate_ref, diagnostic_dumps_gist_only)
- Never-imaged kinds (workpoint_action_authority, evidence_refs_themselves, exact_diffs, secrets, hashes, uuids, recent_live_turns, etc.)
- Compatibility probe checklist (7 items)
- `decide(action, status)` helper enforces `if status != allowed then fallback=text_passthrough`
- Read-only routes:
  - /v1/bloatgaurd/optical/policy
  - /v1/bloatgaurd/optical/ledger
  - /v1/bloatgaurd/optical/probe
  - /v1/bloatgaurd/optical/imaged-kinds
  - /v1/bloatgaurd/optical/never-imaged

## Tests/gates
- cargo test --release -p focusa-api -- routes::bloatgaurd_optical: PASS (4 tests)
- cargo build --release -p focusa-api: PASS
- tests/spec_focusa_101_5_11_bloatgaurd_optical_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
