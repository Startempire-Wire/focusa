# Spec 111 Preload Slices 3-5 proof — 2026-07-06

## Slices

### Slice 3 — API routes dispatch
- /v1/preload/profiles returns full profile metadata (id, label, description, dynamic context flags, max items)
- build_packet_for_profile helper returns packet JSON (schema/profile_id/render_mode/static_rule_lines/dynamic_context_lines/acceptance_prompt/bounded_dynamic_items/rendered)

### Slice 4 — Safe write route
- POST /v1/preload/write with profile_id/target_path/idempotency_key/overwrite
- Safe target prefixes: /tmp/focusa-preload/, /var/cache/focusa/preload/
- Rejects missing idempotency_key (400), unsafe target (403), existing target without overwrite (409), unknown profile (400), IO errors (500)
- Returns FOCUSA_PRELOAD_FAIL error code on rejection

### Slice 5 — Receipt preview integration
- /v1/preload/receipt-preview → receipt_preview_for(profile_id) returns rendered packet as bootstrap_delivery Focusa Receipt preview
- /v1/preload/receipt-commit returns NOT_IMPLEMENTED until Spec 119 receipts ledger commit is integrated

## Tests/gates
- cargo test --release -p focusa-api -- routes::preload: PASS (10 tests, +2 slice4_5)
- cargo build --release -p focusa-api: PASS
- tests/spec_focusa_111_preload_slice3_static_test.sh: PASS
- tests/spec_focusa_111_preload_slice4_5_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
