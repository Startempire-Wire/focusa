#!/usr/bin/env python3
"""Spec98 focusa-877z.4: Focus Gate authority-plane guard."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = (
    ROOT / "docs/worksheets/focusa-877z.4-focus-gate-authority-plane-contract.yaml"
)
TYPES = ROOT / "crates/focusa-core/src/types.rs"
EXTRA = ROOT / "crates/focusa-api/src/routes/capabilities_extra.rs"
REDUCER = ROOT / "crates/focusa-core/src/reducer.rs"
DAEMON = ROOT / "crates/focusa-core/src/runtime/daemon.rs"
GATE_ROUTE = ROOT / "crates/focusa-api/src/routes/gate.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.focus_gate_authority_plane_contract.v1":
        fail("unexpected .4 contract schema")
    if data.get("status") != "focus_gate_is_advisory_candidate_front_door":
        fail("unexpected .4 contract status")

    types = TYPES.read_text()
    extra = EXTRA.read_text()
    reducer = REDUCER.read_text()
    daemon = DAEMON.read_text()
    gate_route = GATE_ROUTE.read_text()

    for invariant in [
        "INVARIANT: Focus Gate never mutates Focus State or Focus Stack.",
        "INVARIANT: Focus Gate never triggers actions.",
        "INVARIANT: Focus Gate only surfaces candidates.",
    ]:
        if invariant not in types:
            fail(f"FocusGateState missing invariant: {invariant}")

    for route_marker in [
        "async fn gate_policy",
        "async fn gate_scores",
        "async fn gate_explain",
    ]:
        start = extra.find(route_marker)
        if start == -1:
            fail(f"missing {route_marker}")
        body = extra[
            start : extra.find("async fn", start + 1)
            if extra.find("async fn", start + 1) != -1
            else len(extra)
        ]
        if '"authority_plane": "advisory_candidate_front_door"' not in body:
            fail(f"{route_marker} must expose advisory authority_plane")
        if '"canonical": false' not in body:
            fail(f"{route_marker} must expose canonical=false")
        if (
            '"promotion_required": "explicit FocusFrame/Workpoint/Trajectory/operator action"'
            not in body
        ):
            fail(f"{route_marker} must expose explicit promotion requirement")

    explain_start = extra.find("async fn gate_explain")
    explain_body = extra[explain_start : extra.find("// ─── Intuition", explain_start)]
    for forbidden in [
        "workpoint_resume",
        "trajectory_goal",
        "attention_recall",
        "work_loop_execution",
    ]:
        if forbidden not in explain_body:
            fail(f"gate_explain must name forbidden authority {forbidden}")

    forbidden_cross_writes = [
        "state.workpoint",
        "state.trajectory",
        "state.work_loop.current_task",
        "state.focus_stack.active_id =",
        "state.focus_stack.frames.push",
    ]
    for event in [
        "IntuitionSignalObserved",
        "CandidateSurfaced",
        "CandidatePinned",
        "CandidateSuppressed",
    ]:
        if f"FocusaEvent::{event}" not in reducer and event not in reducer:
            fail(f"reducer must have Focus Gate event handling for {event}")
    gate_section_start = reducer.find("FocusaEvent::IntuitionSignalObserved")
    gate_section_end = reducer.find("// ─── Reference Store", gate_section_start)
    gate_section = reducer[gate_section_start:gate_section_end]
    for needle in forbidden_cross_writes:
        if needle in gate_section:
            fail(
                f"Focus Gate reducer section must not write other authority plane: {needle}"
            )

    if (
        "fn dispatch_gate_action" not in gate_route
        or ".try_send(action)" not in gate_route
    ):
        fail("Focus Gate mutation dispatch must use bounded try_send helper")
    if (
        ".send(Action::IngestSignal" in gate_route
        or ".send(Action::SurfaceCandidate" in gate_route
    ):
        fail("Focus Gate HTTP routes must not await command channel sends")

    candidate_translate_start = daemon.find("Action::SurfaceCandidate")
    candidate_translate_end = daemon.find(
        "Action::PinCandidate", candidate_translate_start
    )
    if candidate_translate_start == -1 or candidate_translate_end == -1:
        fail("daemon must translate SurfaceCandidate before PinCandidate")
    candidate_translate = daemon[candidate_translate_start:candidate_translate_end]
    for forbidden in ["Workpoint", "Trajectory", "Continuous", "FocusFramePushed"]:
        if forbidden in candidate_translate:
            fail(f"SurfaceCandidate translation must not promote into {forbidden}")

    print(
        "✓ PASS: Focus Gate is explicit advisory candidate front-door, not canonical authority"
    )


if __name__ == "__main__":
    main()
