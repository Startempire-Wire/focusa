#!/usr/bin/env python3
"""Mission Canvas must work on any supported Focusa setup without SaaS."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
contract = json.loads(
    (ROOT / "docs/contracts/spec135-mission-canvas-portability.v1.yaml").read_text()
)
assert contract["schema"] == "focusa.spec135.mission_canvas_portability.v1"
boundary = contract["product_boundary"]
assert boundary["optional_service"] == "focus.work"
assert boundary["optional_service_required"] is False
assert boundary["vendor_cloud_required"] is False
assert boundary["operator_vps_required"] is False

defaults = contract["out_of_box_defaults"]
assert defaults["daemon_discovery"] == "http://127.0.0.1:8787"
assert defaults["daemon_base_url_overridable"] is True
assert defaults["local_identity_generated"] is True
assert defaults["local_sqlite_persistence"] is True
assert defaults["network_connection_required"] is False
assert defaults["external_oauth_required"] is False
assert defaults["external_database_required"] is False
assert defaults["generated_sdk_required_at_runtime"] is False
assert {
    "local_single_user",
    "self_hosted_lan",
    "self_hosted_remote",
    "optional_managed_saas",
} <= set(contract["supported_topologies"])

wizard = (
    ROOT / "apps/menubar/src/lib/components/FirstRunWizard.svelte"
).read_text()
local_probe = wizard.index("discoveryAttempts.push(`local: ${url}`)")
tailscale_probe = wizard.index("const tailscaleHosts")
bonjour_probe = wizard.index("focusa_discover_via_bonjour")
assert local_probe < tailscale_probe < bonjour_probe
assert "A local daemon works out of the box" in wizard
assert "Could not find a Focusa daemon" in wizard
assert "focus.work" not in wizard

runtime_sources = [
    ROOT / "crates/focusa-api/src/routes/mission_canvas_surfaces.rs",
    ROOT / "crates/focusa-core/src/types.rs",
    ROOT / "crates/focusa-core/src/reducer.rs",
]
for source in runtime_sources:
    text = source.read_text()
    assert "focus.work" not in text
    assert "/home/focusadev" not in text

print("Spec 135 Mission Canvas portable out-of-box/no-SaaS contract: PASS")
