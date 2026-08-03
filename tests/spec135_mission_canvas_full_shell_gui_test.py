#!/usr/bin/env python3
"""Spec 135 authoritative Pi-native Mission Canvas drift firewall."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROOF = json.loads((ROOT / "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json").read_text())
HOST = (ROOT / "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml").read_text()
SHELL = (ROOT / "apps/pi-extension/src/mission-canvas-shell.ts").read_text()
VIEW = (ROOT / "apps/pi-extension/src/mission-canvas-view.ts").read_text()
TOOL = (ROOT / "apps/pi-extension/src/mission-canvas-tool.ts").read_text()
COMMANDS = (ROOT / "apps/pi-extension/src/commands.ts").read_text()
WIDGET = (ROOT / "apps/pi-extension/src/mission-canvas-widget.ts").read_text()

assert PROOF["status"] == "pi_native_reference_design_implemented"
assert PROOF["accepted"] is True
assert PROOF["runtime"]["observed_host_renderer"] == "pi_native_mission_canvas"
assert PROOF["runtime"]["required_host_renderer"] == "pi_native_mission_canvas"
assert PROOF["runtime"]["same_pi_process"] is True
assert PROOF["runtime"]["same_pi_terminal"] is True
assert PROOF["runtime"]["browser_or_remote_host_launched"] is False

for marker in [
    "pi_native_mission_canvas",
    "ctx.ui.custom",
    "current_pi_process",
    "current_pi_terminal",
    "same_runtime_not_a_handoff: true",
    "actual_work_surface_strip",
    "activity_mode_rail_or_compact_tabs",
    "resolved_contribution_grid",
    "steering_queue_when_populated",
    "follow_up_queue_when_populated",
    "prompt_editor_targeting_current_pi_session",
    "launch_browser_for_canvas",
]:
    assert marker in HOST, marker

assert "@earendil-works/pi-tui" in SHELL
assert "Authoritative Pi-native Mission Canvas" in SHELL
assert "ctx.ui.custom" in COMMANDS
assert "closeActiveMissionCanvasShell" in COMMANDS
assert 'mode !== "canvas-guided"' in COMMANDS
assert 'interactionMode.mode !== "canvas-guided"' in WIDGET
assert "Pi-native authoritative Mission Canvas" in VIEW
assert "resolveContributions" in VIEW
assert "RichHostLifecycleManager" not in TOOL
assert 'gui: "pi_tui"' in TOOL

for ref in PROOF["evidence_refs"]:
    assert (ROOT / ref).exists(), ref

print("Spec 135 authoritative Pi-native Mission Canvas host/renderer: PASS")
