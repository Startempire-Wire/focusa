#!/usr/bin/env python3
"""SPEC135-M7 accessibility, recovery clarity, and nontechnical operation proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
canvas = (ROOT / "apps/menubar/src/lib/components/MissionCanvasView.svelte").read_text()
runtime = (ROOT / "apps/menubar/src/lib/components/RuntimeView.svelte").read_text()
rail = (ROOT / "apps/pi-extension/src/work-rail-widget.ts").read_text()

for marker in (
    'aria-label="Focusa Mission Canvas"',
    'aria-describedby="mission-canvas-help"',
    'role="status"',
    'aria-live="polite"',
    "Use Tab to reach controls and Enter or Space",
    ":focus-visible",
    "prefers-reduced-motion: reduce",
):
    assert marker in canvas
assert 'aria-label="Copy the exact Mission Canvas resume command"' in runtime
assert "JSON.stringify" not in canvas
assert "JSON.stringify" not in runtime
for marker in ("ASCII", "nextAction", "proofCount"):
    assert marker.lower() in rail.lower()
m2_proof = (ROOT / "tests/spec135_m2_pi_work_rail_test.py").read_text().lower()
assert "keyboard" in m2_proof and "mouse" in m2_proof

print("Spec 135 M7 accessible, recoverable, nontechnical Mission Canvas: PASS")
