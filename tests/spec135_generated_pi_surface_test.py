#!/usr/bin/env python3
"""Generated C.R.I.S.T. bindings are surfaced through the Pi-native Canvas."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
view = (ROOT / "apps/pi-extension/src/mission-canvas-view.ts").read_text()
shell = (ROOT / "apps/pi-extension/src/mission-canvas-shell.ts").read_text()
crist = (ROOT / "apps/pi-extension/src/crist-canvas.ts").read_text()
manifest = json.loads((ROOT / "packages/focusa-elements/src/component-manifest.json").read_text())

assert len(manifest) >= 30
for vertical in ["Context", "Role", "Interview", "Spec", "Task"]:
    assert any(vertical.lower() in item["name"].lower() for item in manifest), vertical

for action in ["/focusa-context", "/focusa-role", "/focusa-interview", "/focusa-crist", "/focusa-rail"]:
    assert action in view, action
assert "Generated action" in view
assert "Authority: generated presentation only" in crist
assert "canonical reducers remain authoritative" in crist
assert "MissionCanvasView" in shell
assert "render(" in shell

# Generated UI is semantic Pi text, never a browser runtime or hidden sidecar.
for forbidden in ["rich-host", "a2ui-runtime", "renderBrowserSurface", "iframe", "document.write"]:
    assert forbidden not in view.lower()
assert not (ROOT / "apps/pi-extension/rich-host").exists()
assert not (ROOT / "apps/pi-extension/src/rich-host").exists()

print("Spec 135 generated C.R.I.S.T. and Pi-native Mission Canvas surfaces: PASS")
