#!/usr/bin/env python3
"""Spec 135 Mission Canvas host/renderer truth and drift firewall."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROOF = json.loads(
    (ROOT / "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json").read_text()
)
HOST_CONTRACT = (
    ROOT / "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml"
).read_text()
SHELL = (ROOT / "apps/pi-extension/src/mission-canvas-shell.ts").read_text()
COMMANDS = (ROOT / "apps/pi-extension/src/commands.ts").read_text()
SESSION = (ROOT / "apps/pi-extension/src/session.ts").read_text()

# Current truth: the existing full-screen Pi component is terminal projection
# scaffolding and must not close the rich GUI requirement.
assert PROOF["status"] == "invalidated_as_rich_gui_proof"
assert PROOF["accepted"] is False
assert PROOF["runtime"]["observed_host_renderer"] == "pi_terminal_projection"
assert PROOF["runtime"]["required_host_renderer"] == "focusa_pi_rich_window"
assert PROOF["reclassified_partial_evidence"]["terminal_projection_exists"] is True
assert PROOF["reclassified_partial_evidence"]["rich_graphical_host_exists"] is False

# The machine contract must preserve the original Pi light-switch intent and
# distinguish interaction mode from renderer capability.
for marker in [
    "focusa_pi_rich_window",
    "pi_terminal_projection",
    "must_not_be_inferred_from_interaction_mode_alone",
    "same_runtime_not_a_handoff: true",
    "bind_current_pi_session_as_a_pi_session_work_surface",
    "work_surface_strip",
    "focused_work_surface_with_focusa_right_inspector",
    "work_rail",
    "steering_queue",
    "follow_up_queue",
    "prompt_editor",
]:
    assert marker in HOST_CONTRACT, marker

# Existing useful lifecycle scaffolding remains present but is classified
# truthfully. A terminal component or source marker is never rich-GUI proof.
assert "@earendil-works/pi-tui" in SHELL
assert "ctx.ui.custom" in COMMANDS
assert "closeActiveMissionCanvasShell" in COMMANDS
assert 'interactionMode === "canvas-guided"' in SESSION
assert "canonical_runtime_forked" in json.dumps(PROOF)

# No evidence reference may be enough merely because a file exists.
for ref in PROOF["evidence_refs_retained_as_partial"]:
    assert (ROOT / ref).exists(), ref

print(
    "Spec 135 Mission Canvas host/renderer drift firewall: PASS "
    "(rich Pi GUI correctly remains open)"
)
