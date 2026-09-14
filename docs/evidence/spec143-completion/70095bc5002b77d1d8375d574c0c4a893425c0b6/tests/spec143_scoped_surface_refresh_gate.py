#!/usr/bin/env python3
"""Release gate for GitHub #45 scoped Mission Canvas/advisory refresh."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REFRESH = (ROOT / "apps/pi-extension/src/scoped-surface-refresh.ts").read_text()
WIDGET = (ROOT / "apps/pi-extension/src/mission-canvas-widget.ts").read_text()
SESSION = (ROOT / "apps/pi-extension/src/session.ts").read_text()
TOOLS = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
RAIL = (ROOT / "apps/pi-extension/src/work-rail-widget.ts").read_text()
INDEX = (ROOT / "apps/pi-extension/src/index.ts").read_text()

for token in [
    "focusa.scoped_state_change_receipt.v1",
    "focusa.truthful_scoped_surface_snapshot.v1",
    "project_root",
    "continuity_id",
    "stale_age_ms",
    "last_refresh_status",
    "trajectory: \"absent\" | \"provisional\" | \"persisted\"",
    "workpoint: \"absent\" | \"present\" | \"blocked\"",
    "proof: \"missing\" | \"linked\" | \"verified\"",
]:
    assert token in REFRESH, f"missing refresh contract token: {token}"

assert "publishScopedStateChange" in TOOLS
assert "responseRoot === requestedRoot" in TOOLS
assert "subscribeScopedStateChanges" in WIDGET
assert "scopedReceiptMatchesCurrentScope" in WIDGET
assert "age >= 60_000" in WIDGET
assert 'focusaFetch("/workpoint/resume"' in WIDGET
assert "scopedRefreshEvents" in SESSION
assert "eventRoot === currentRoot" in SESSION
assert "eventContinuity === currentContinuity" in SESSION
assert "proof missing" in RAIL
assert "proof:missing" in INDEX
assert "✓ proof 0" not in RAIL

print("Spec143 scoped surface refresh gate: PASS")
