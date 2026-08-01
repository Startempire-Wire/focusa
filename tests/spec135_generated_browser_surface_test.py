#!/usr/bin/env python3
"""Permanent A2UI/Lit and UIAI rich-host binding proof."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
frontend = (ROOT / "apps/pi-extension/rich-host/assets/main.js").read_text()
html = (ROOT / "apps/pi-extension/rich-host/assets/index.html").read_text()
entrypoint = (ROOT / "apps/pi-extension/rich-host/host-entrypoint.mjs").read_text()
a2ui = (ROOT / "packages/a2ui-renderer/src/rich-host.ts").read_text()
bundle = ROOT / "apps/pi-extension/rich-host/assets/a2ui-runtime.js"
manifest = json.loads((ROOT / "packages/focusa-elements/src/component-manifest.json").read_text())

assert bundle.exists() and bundle.stat().st_size > 100_000
assert "a2ui-runtime.js" in html and "a2ui-runtime" in entrypoint
assert "FocusaGeneratedSurfaceElement" in a2ui
assert "FocusaA2uiRenderer" in a2ui
assert "allowedActionNames" in a2ui
assert "focusa-operation" in a2ui
assert len(manifest) >= 30

for vertical in ["Context", "Role", "Interview", "Spec", "Task"]:
    assert any(vertical.lower() in item["name"].lower() for item in manifest), vertical
assert "focusa-generated-surface" in frontend
assert "a2ui_messages" in frontend
assert "recovery_operation_id" in frontend
assert "renderBrowserSurface" in frontend
for view in ["screenshot_url", "snapshot", "diagnostics", "artifacts"]:
    assert view in frontend
assert "UIAI Engine Cockpit" in frontend
assert "dataset.uiaiSessionId" in frontend
assert "innerHTML" not in frontend
assert "eval(" not in frontend
assert "document.write" not in frontend
assert "iframe" not in frontend.lower()
assert "https?:\\/\\/127" in frontend

print("Spec 135 generated C.R.I.S.T. and UIAI browser surfaces: PASS")
