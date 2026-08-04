#!/usr/bin/env python3
"""Static contract gate for the scoped compaction-policy surfaces."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
api = (ROOT / "crates/focusa-api/src/routes/compaction_policy.rs").read_text()
cli = (ROOT / "crates/focusa-cli/src/commands/compaction.rs").read_text()
pi = (ROOT / "apps/pi-extension/src/auto-compaction.ts").read_text()
pi_adapter = (ROOT / "apps/pi-extension/src/compaction-policy-adapter.ts").read_text()
api_resolution = (ROOT / "crates/focusa-api/src/routes/compaction_policy_resolution.rs").read_text()
tui_app = (ROOT / "crates/focusa-tui/src/app.rs").read_text()
tui_view = (ROOT / "crates/focusa-tui/src/views/telemetry.rs").read_text()

for field in [
    "pressure_percent",
    "selected_route",
    "reason",
    "rollback_route",
    "operator_override",
]:
    assert field in api, f"API omits {field}"
    assert field in cli, f"CLI omits {field}"
    assert field in pi, f"Pi omits {field}"
    assert field in tui_view, f"TUI omits {field}"

assert '"/v1/compaction/policy"' in api
assert '"/v1/compaction/policy/override"' in api
assert '"/v1/compaction/policy"' in cli
assert '"/v1/compaction/policy/override"' in cli
assert '"/compaction/policy"' in pi
assert '"/compaction/policy/override"' in pi
assert '"/v1/compaction/policy?project_root=' in tui_app
assert '"/v1/compaction/policy/resolve"' in api_resolution
assert 'resolve_runtime_fingerprint' in api_resolution
assert 'legal_action_mask' in api_resolution
assert 'resolve_policy' in api_resolution
assert '"/compaction/policy/resolve"' in pi_adapter
assert 'selectFrozenCompactionPolicy' in pi
assert 'focusa.compaction_policy_override_receipt.v1' in api
assert '"reversible":true' in api
assert "require_workstream_key" in api
assert "policy_store_write_failed" in api
assert "applyOperatorOverride" in pi
assert 'registerCommand("focusa-compaction-policy"' in pi
print("compaction policy API/CLI/Pi/TUI parity passed")
