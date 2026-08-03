#!/usr/bin/env python3
"""Spec 135 Pi-native interaction-mode and renderer truth gate."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
C = json.loads((ROOT / "docs/contracts/spec135-interaction-mode-toggle.v1.json").read_text())
GUI = json.loads((ROOT / "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json").read_text())
HOST = (ROOT / "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml").read_text()
CFG = (ROOT / "apps/pi-extension/src/config.ts").read_text()
CMD = (ROOT / "apps/pi-extension/src/commands.ts").read_text()
TOOL = (ROOT / "apps/pi-extension/src/mission-canvas-tool.ts").read_text()
SHELL = (ROOT / "apps/pi-extension/src/mission-canvas-shell.ts").read_text()

assert C["interaction_modes"] == ["canvas-guided", "terminal-guided", "headless"]
assert C["host_renderers"] == ["pi_native_mission_canvas", "stock_pi", "headless_none"]
assert "current Pi terminal" in HOST
assert "ctx.ui.custom" in HOST
assert "launch_browser_for_canvas" in HOST
assert C["accepted"] is True and C["status"] == "verified"
assert GUI["accepted"] is True
assert GUI["runtime"]["observed_host_renderer"] == "pi_native_mission_canvas"
assert GUI["runtime"]["same_pi_process"] is True
assert GUI["runtime"]["same_pi_terminal"] is True
assert GUI["runtime"]["browser_or_remote_host_launched"] is False

for mode in C["interaction_modes"]:
    assert mode in CFG and mode in CMD
for key, value in C["foundations_proven"].items():
    expected = False if key in {"state_loss_observed", "nag_on_headless", "browser_or_remote_host_launched"} else True
    assert value is expected, key

assert "resolveInteractionMode" in CFG
assert 'registerCommand("mission-canvas-mode"' in CMD
assert "sessionInteractionModes.set" in CMD
assert "saveConfigOverrides" in CMD
assert "executeMissionCanvasAction" in TOOL
assert 'gui: "pi_tui"' in TOOL
assert "RichHostLifecycleManager" not in TOOL
assert "Authoritative Pi-native Mission Canvas" in SHELL
for ref in C["proof_refs"] + C["implementation_refs"]:
    assert (ROOT / ref).exists(), ref

print("Spec 135 Pi-native interaction mode/renderer truth: PASS")
