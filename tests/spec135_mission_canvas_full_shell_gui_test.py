#!/usr/bin/env python3
"""Spec 135 Mission Canvas full-shell replacement and restoration proof."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROOF = json.loads((ROOT / "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json").read_text())
SHELL = (ROOT / "apps/pi-extension/src/mission-canvas-shell.ts").read_text()
COMMANDS = (ROOT / "apps/pi-extension/src/commands.ts").read_text()
SESSION = (ROOT / "apps/pi-extension/src/session.ts").read_text()

assert PROOF["full_shell"]["stock_tui_replaced_when_on"] is True
assert PROOF["full_shell"]["stock_tui_restored_when_off"] is True
assert PROOF["full_shell"]["on_again_replaces_stock_tui"] is True
assert PROOF["full_shell"]["canvas_owns_agent_stream"] is True
assert PROOF["full_shell"]["canvas_owns_user_input"] is True
assert PROOF["full_shell"]["same_session_manager"] is True
assert PROOF["programmatic_control"]["canonical_runtime_forked"] is False
assert PROOF["empty_state_behavior"]["trajectory_gate"] is False
assert PROOF["empty_state_behavior"]["workpoint_gate"] is False
assert PROOF["empty_state_behavior"]["project_verify_modal_gate"] is False
for ref in PROOF["evidence_refs"]:
    assert (ROOT / ref).exists(), ref
assert "Complete alternate Pi shell" in SHELL
assert "AGENT STREAM · SAME SESSION" in SHELL
assert "setFooter(undefined)" in SHELL
assert "closeActiveMissionCanvasShell" in COMMANDS
assert 'interactionMode === "canvas-guided"' in SESSION
print("Spec 135 Mission Canvas full-shell GUI replacement/restoration: PASS")
