#!/usr/bin/env python3
"""GH#89 canonical daemon-routing authority parity gate."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
files = {
    "core": ROOT / "crates/focusa-core/src/daemon_multiplex.rs",
    "api": ROOT / "crates/focusa-api/src/routes/daemon_routing.rs",
    "cli": ROOT / "crates/focusa-cli/src/commands/daemon_routing.rs",
    "pi": ROOT / "apps/pi-extension/src/tools.ts",
    "tui": ROOT / "crates/focusa-tui/src/api.rs",
}
text = {name: path.read_text() for name, path in files.items()}

contract = "focusa.daemon_routing_authority.v1"
request = "focusa.daemon_routing_resolve.v1"
assert contract in text["core"]
for surface in ["api", "cli", "pi", "tui"]:
    assert request in text[surface], f"{surface} missing canonical request contract"
for field in [
    "project_root",
    "continuity_id",
    "working_subpath_id",
    "native_session_id",
    "selected_daemon_id",
    "recovery_required",
    "failure_class",
]:
    for surface in ["core", "cli", "pi", "tui"]:
        assert field in text[surface], f"{surface} missing {field}"
assert 'name: "focusa_daemon_routing_status"' in text["pi"]
assert 'name = "daemon-routing"' in (ROOT / "crates/focusa-cli/src/main.rs").read_text()
assert "resolve_daemon_routing" in text["tui"] and "pub fn display" in text["tui"]
assert 'selected_daemon_id: null' in text["pi"]
assert 'failure_class: "daemon_unavailable"' in text["pi"]
print("GH#89 routing parity: PASS (API/CLI/Pi/TUI share explicit fail-closed authority)")
