#!/usr/bin/env python3
"""Spec 135A-5 Pi tabs, splits, comparison, grouping, and inspector proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md").read_text()
LAYOUT = (ROOT / "apps/pi-extension/src/mission-canvas-layout.ts").read_text()
INDEX = (ROOT / "apps/pi-extension/src/index.ts").read_text()
MODEL = (ROOT / "apps/pi-extension/src/mission-canvas-model.ts").read_text()
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()

for action in (
    "Switch tab",
    "Horizontal split",
    "Vertical split",
    "Side-by-side comparison",
    "Pin or unpin surface",
    "Mark read or unread",
    "Group by project",
    "Group by workstream",
    "Group by session",
    "Inspect active surface",
    "Clear split",
):
    assert action in LAYOUT, action

for token in (
    'registerCommand("focusa-surfaces"',
    'registerShortcut("ctrl+shift+]"',
    'registerShortcut("ctrl+shift+["',
    "SplitOrientation",
    "secondaryIndex",
    "canonical_state_refs",
    "Presentation controls never mutate",
):
    assert token in LAYOUT, token

assert "/mission-canvas/surfaces/mutate" not in LAYOUT
assert "focusaFetch(`/mission-canvas/surfaces?" in LAYOUT
assert "registerMissionCanvasLayout(pi)" in INDEX
assert 'registerMessageRenderer("focusa-canvas-layout"' in INDEX
for token in ("pinned", "splitGroupId", "unreadEventCount"):
    assert token in MODEL, token
for token in ("MissionCanvasWorkSurfaceRecord", "tab_index", "pinned", "unread"):
    assert token in TYPES, token

for requirement in (
    "tab strip and keyboard switcher",
    "horizontal and vertical splits",
    "side-by-side comparison",
    "pinned and unread surfaces",
    "per-surface inspector",
):
    assert requirement in SPEC, requirement

print("Spec 135 A5 Pi surface layout/navigation: PASS")
