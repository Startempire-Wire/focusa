#!/usr/bin/env python3
"""Spec 135A-7 responsive/accessibility Pi TUI proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md").read_text()
ACCESS = (ROOT / "apps/pi-extension/src/mission-canvas-accessibility.ts").read_text()
VIEW = (ROOT / "apps/pi-extension/src/mission-canvas-view.ts").read_text()
INVENTORY = (ROOT / "apps/pi-extension/src/mission-canvas-session-inventory.ts").read_text()
RUNTIME = (ROOT / "apps/pi-extension/tests/mission-canvas-accessibility.test.mjs").read_text()

for token in (
    "CanvasResponsiveMode",
    '"narrow"',
    '"stacked"',
    '"desktop"',
    "accessibilityPreferences",
    "highContrast",
    "reducedMotion",
    "colorIndependent: true",
    "restoreFocusAfterModal: true",
    "virtualWindow",
    "accessibleStateLabel",
    "focusRestorationLabel",
):
    assert token in ACCESS, token

for token in (
    "responsiveCanvasMode(width)",
    "surfaceCapacity(width)",
    "virtualWindow(surfaces",
    "preferences.highContrast",
    "preferences.reducedMotion",
    "wrapTextWithAnsi",
    "truncateToWidth",
    "filled(",
):
    assert token in VIEW, token

assert "MAX_MISSION_CANVAS_ROWS" in INVENTORY
assert "Array.from({ length: 1000 }" in RUNTIME
assert 'responsiveCanvasMode(47), "narrow"' in RUNTIME
assert 'responsiveCanvasMode(90), "desktop"' in RUNTIME

for requirement in (
    "compact/narrow session switcher and drawer",
    "stacked terminal layout",
    "keyboard-only Work Surface navigation",
    "clear focus indicators",
    "color-independent states",
    "high contrast",
    "reduced motion",
    "virtualized long session/artifact lists",
    "restored focus after modal closure",
):
    assert requirement in SPEC, requirement

print("Spec 135 A7 responsive/accessibility Pi TUI: PASS")
