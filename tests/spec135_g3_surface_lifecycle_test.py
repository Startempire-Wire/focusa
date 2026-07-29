#!/usr/bin/env python3
"""Spec 135G-3 surface lifecycle and close semantics proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-surface-lifecycle-close-semantics.v1.json").read_text())
R=(ROOT/"crates/focusa-api/src/routes/mission_canvas_surfaces.rs").read_text()
assert C["close_view"]["session_terminated"] is False
assert C["close_view"]["provider_work_closed"] is False
assert C["close_view"]["workpoint_completed"] is False
assert C["close_view"]["reopenable"] is True
assert C["terminate_session"]["surface_action"] is False
assert C["terminate_session"]["confirmation_required"] is True
assert C["terminate_session"]["preview_required"] is True
assert C["terminate_session"]["receipt_required"] is True
surface_enum=R[R.index("pub enum SurfaceAction"):R.index("pub struct SurfaceRequest")]
for action in ("Create","Arrange","Suspend","Resume","CloseView"):
    assert action in surface_enum
assert "Terminate" not in surface_enum
assert "SurfaceAction::CloseView" in R
assert "MissionCanvasSurfaceStatus::ViewClosed" in R
assert "close_view never terminates session or provider work" in C["transition_laws"]
print("Spec 135 G3 surface lifecycle and close semantics: PASS")
