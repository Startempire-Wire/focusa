# Spec 111 Preload Slice 2 proof — 2026-07-06

Scope: focusa-wkzg.2 Spec 111 Slice 2 — core packet types and renderers.

## Added
- RenderMode enum (StaticRule | DynamicContext | AcceptancePrompt)
- AgentBootstrapProfile struct with id/label/description/includes_dynamic_context/includes_acceptance_prompt/max_dynamic_items
- Four profile constants matching slice 1 ids
- profile_by_id lookup
- AgentBootstrapPacket struct (schema/profile_id/render_mode/static_rule_lines/dynamic_context_lines/acceptance_prompt/bounded_dynamic_items)
- build_packet(profile_id) returning Result with FOCUSA_PRELOAD_FAIL on unknown profile
- render_packet(packet) producing markdown output with Rules/Context/Acceptance sections

## Tests/gates
- cargo test --release -p focusa-api -- routes::preload: PASS (6 tests, +3 slice2)
- cargo build --release -p focusa-api: PASS
- tests/spec_focusa_111_preload_slice2_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
