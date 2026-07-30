#!/usr/bin/env python3
"""Spec 135 interaction-mode and host-renderer truth gate."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
C = json.loads(
    (ROOT / "docs/contracts/spec135-interaction-mode-toggle.v1.json").read_text()
)
GUI = json.loads(
    (ROOT / "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json").read_text()
)
HOST = (
    ROOT / "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml"
).read_text()
CFG = (ROOT / "apps/pi-extension/src/config.ts").read_text()
CMD = (ROOT / "apps/pi-extension/src/commands.ts").read_text()

assert C["interaction_modes"] == ["canvas-guided", "terminal-guided", "headless"]
assert "focusa_pi_rich_window" in C["host_renderers"]
assert "pi_terminal_projection" in C["host_renderers"]
assert "must_not_be_inferred_from_interaction_mode_alone" in HOST

for mode in C["interaction_modes"]:
    assert mode in CFG and mode in CMD
for key, value in C["foundations_proven"].items():
    expected = False if key in {"state_loss_observed", "nag_on_headless"} else True
    assert value is expected, key

assert "resolveInteractionMode" in CFG
assert 'registerCommand("mission-canvas-mode"' in CMD
assert "sessionInteractionModes.set" in CMD
assert "saveConfigOverrides" in CMD
for ref in (
    C["existing_proof_refs_reclassified_as_terminal_foundation"]
    + C["implementation_refs"]
):
    assert (ROOT / ref).exists(), ref

# Verification is derived from the actual rich-host proof, not from the mode
# enum or the existence of terminal controls.
assert C["accepted"] is bool(GUI.get("accepted"))
assert C["rich_host_proof"]["accepted"] is bool(GUI.get("accepted"))
if not GUI.get("accepted"):
    assert C["status"] == "partial_foundation"
    assert all(C["not_yet_proven"].values())
else:
    assert C["status"] == "verified"
    assert not any(C["not_yet_proven"].values())

print(
    "Spec 135 interaction mode/host renderer truth: PASS "
    f"({C['status']})"
)
