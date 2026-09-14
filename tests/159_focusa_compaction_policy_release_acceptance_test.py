#!/usr/bin/env python3
"""GH #112 structural release-acceptance gate; behavioral proof lives in Rust/Node tests."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
core = ROOT / "crates/focusa-core/src/compaction_policy"
auto = (ROOT / "apps/pi-extension/src/auto-compaction.ts").read_text()
adapter = (ROOT / "apps/pi-extension/src/compaction-policy-adapter.ts").read_text()
compaction = (ROOT / "apps/pi-extension/src/compaction.ts").read_text()
config = (ROOT / "apps/pi-extension/src/config.ts").read_text()
projection = (ROOT / "apps/pi-extension/src/compaction-resume-projection.ts").read_text()
controller = (ROOT / "crates/focusa-api/src/routes/compaction_policy_resolution.rs").read_text()

required_core = [
    "identity.rs",
    "capabilities.rs",
    "candidate.rs",
    "pressure.rs",
    "semantic_pressure.rs",
    "selector.rs",
    "provider_strategies.rs",
    "registry.rs",
    "lifecycle.rs",
    "differential.rs",
]
for file in required_core:
    assert (core / file).is_file(), f"missing Rust authority module {file}"

assert auto.count("ctx.compact(") == 1, "exactly one direct Pi compact call required"
assert "selectFrozenCompactionPolicy(ctx" in auto
assert "prewarmCompactionPolicy(ctx, getConfig())" in auto
assert "provider_overflow" in auto and '["manual", "provider_overflow"].includes' in auto
assert "successful_compaction_hysteresis" in auto
assert "focusaPost(\"/compaction/policy/observe\"" in adapter
assert "selectCompactionPolicy(telemetry, capabilities)" in adapter, "exact local fallback required"
assert "await focusaFetch" not in auto[auto.index('pi.on("input"'):auto.index('pi.on("session_compact"')]

assert not any(f"microCompactEveryNTurns: {n}" in config for n in range(1, 10))
assert "automatic fixed-cadence compaction is disabled" in compaction
for field in [
    "compactionPolicyMode",
    "compactionCanaryEnrollment",
    "compactionAdaptiveMinSamples",
    "compactionAdaptiveConfidence",
]:
    assert field in config
assert 'compactionPolicyMode: "shadow"' in config

for budget in ['normal: 900', 'pressure: 600', 'critical: 400', 'blocked: 250']:
    assert budget in projection
assert "renderCompactionMissionPacket" not in compaction
assert "renderCompactionResumeProjection" in compaction
assert "return undefined" in compaction, "Pi native tactical summary must remain authoritative"

for route in [
    "resolve",
    "observe",
    "status",
    "candidates",
    "evidence",
    "canary/enroll",
    "canary/pause",
    "rollback",
]:
    assert f'/v1/compaction/policy/{route}' in controller

provider = (core / "provider_strategies.rs").read_text()
for contract in [
    "openai_opaque_compaction_item_round_trip",
    "anthropic_compaction_block_round_trip",
    "anthropic_usage_iterations",
    "thought_signature_round_trip",
    "gemini_request_scoped_config_replay",
    "PiStructuredFallback",
]:
    assert contract in provider

lifecycle = (core / "lifecycle.rs").read_text()
for state in ["minimum_samples_not_met", "operator_input_regression", "transport_fallback", "legacy_current_v1"]:
    assert state in lifecycle

differential = (core / "differential.rs").read_text()
for invariant in [
    "operator_turn_loss_or_reorder",
    "duplicate_model_turn",
    "foreign_scope",
    "opaque_state_loss",
    "recovery_handle_loss",
    "adaptive_task_success_inferior",
    "performance_budget_exceeded",
    "rollback_drill_failed",
]:
    assert invariant in differential

print("GH#112 compaction policy release acceptance: PASS")
