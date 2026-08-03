#!/usr/bin/env python3
"""Pi-native Mission Canvas hardening and dead-host absence gate."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
view = (ROOT / "apps/pi-extension/src/mission-canvas-view.ts").read_text()
shell = (ROOT / "apps/pi-extension/src/mission-canvas-shell.ts").read_text()
commands = (ROOT / "apps/pi-extension/src/commands.ts").read_text()
contract = (ROOT / "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml").read_text()
package = json.loads((ROOT / "apps/pi-extension/package.json").read_text())
performance = (ROOT / "apps/pi-extension/tests/mission-canvas-performance.test.mjs").read_text()
reference = (ROOT / "apps/pi-extension/tests/mission-canvas-reference-design.test.mjs").read_text()
fixture = (ROOT / "apps/pi-extension/tests/fixtures/spec135-pi-native-uiai/index.html").read_text()
responsive = json.loads((ROOT / "tests/fixtures/spec135-responsive-evaluations.json").read_text())

# The only interactive Mission Canvas implementation is the Pi TUI component.
assert (ROOT / "apps/pi-extension/src/mission-canvas-view.ts").exists()
assert (ROOT / "apps/pi-extension/src/mission-canvas-shell.ts").exists()
assert not (ROOT / "apps/pi-extension/src/rich-host").exists()
assert not (ROOT / "apps/pi-extension/rich-host").exists()
assert not any(ROOT.glob("apps/pi-extension/tests/rich-host-*.mjs"))
assert not (ROOT / "apps/pi-extension/tests/run-rich-host-lifecycle.mjs").exists()
assert "test:rich-host" not in package["scripts"]

# Pi-native output is bounded and hostile terminal control sequences are removed.
for rule in ["virtualWindow", "replace(/\\x1b", "handleInput", "render(width", "mode-next", "profile-next", "surface-next"]:
    assert rule in view, rule
assert "MissionCanvasView" in shell
assert "ctx.ui.custom" in commands
assert "current Pi terminal" in contract
for forbidden in ["remote host", "browser", "webview", "sidecar"]:
    assert forbidden in contract, forbidden
assert "5_000" in performance
assert "SECRETS" in reference

# UIAI fixture routes exercise canonical projection evidence only; they are not a renderer.
assert 'data-renderer="pi_native_mission_canvas"' in fixture
assert "projection" in (ROOT / "apps/pi-extension/tests/mission-canvas-uiai-server.mjs").read_text()
assert {item["viewport"]["platform"] for item in responsive} == {"macOS", "Windows", "Linux"}
assert min(item["viewport"]["css_width"] for item in responsive) == 1024

print("Spec 135 Pi-native Mission Canvas hardening and dead-host absence: PASS")
