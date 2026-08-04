#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
api = (ROOT / "crates/focusa-api/src/routes/letta.rs").read_text()
cli = (ROOT / "crates/focusa-cli/src/commands/letta.rs").read_text()
pi = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
tui = (ROOT / "crates/focusa-tui/src/api.rs").read_text()
contract = "focusa.letta_surface_status.v1"
assert contract in api and contract in pi
for field in ["availability", "identity", "active_operation", "evidence_refs", "recovery", "controls"]:
    for name, text in [("api", api), ("cli", cli), ("pi", pi), ("tui", tui)]:
        assert field in text, f"{name} missing {field}"
assert 'name: "focusa_letta_status"' in pi
assert "fetch_letta_status" in tui and "pub fn display" in tui
assert '"mutation": true, "supported": false' in api
assert "Mutation controls remain disabled" in cli
print("GH#107 Letta surface parity: PASS (API/CLI/Pi/TUI; unsupported mutation authority explicit)")
