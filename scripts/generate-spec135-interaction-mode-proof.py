#!/usr/bin/env python3
"""Generate the Spec 135 interaction-mode/host-renderer truth contract."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/contracts/spec135-interaction-mode-toggle.v1.json"
GUI_PROOF_PATH = ROOT / "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json"
GUI_PROOF = json.loads(GUI_PROOF_PATH.read_text())
rich_host_accepted = bool(GUI_PROOF.get("accepted"))

contract = {
    "schema": "focusa.spec135.interaction_mode_toggle.v1",
    "status": "verified" if rich_host_accepted else "partial_foundation",
    "acceptance_criteria": (
        "Interaction mode and host renderer resolve independently, and Canvas ON/OFF "
        "from Pi preserves the exact live runtime while launching or closing the truthful host renderer."
    ),
    "authority_ref": "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml",
    "interaction_modes": ["canvas-guided", "terminal-guided", "headless"],
    "host_renderers": [
        "focusa_pi_rich_window",
        "uiai_engine_cockpit",
        "mission_deck_web",
        "pi_terminal_projection",
        "native_tui",
        "menubar_peek",
        "headless_none",
    ],
    "precedence": [
        "temporary-session override",
        "project preference",
        "user preference",
        "environment",
        "default canvas-guided",
    ],
    "foundations_proven": {
        "durable_mode": True,
        "source_displayed": True,
        "scope_exact": True,
        "refresh_immediate": True,
        "resume_survives": True,
        "reconnect_survives": True,
        "headless_no_ui_calls": True,
        "state_loss_observed": False,
        "nag_on_headless": False,
    },
    "rich_host_proof": {
        "evidence_ref": "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json",
        "accepted": rich_host_accepted,
    },
    "not_yet_proven": {
        "host_renderer_is_independently_resolved": not rich_host_accepted,
        "focusa_pi_rich_window_launches_from_pi": not rich_host_accepted,
        "focusa_pi_rich_window_closes_back_to_same_pi_session": not rich_host_accepted,
        "unsent_pi_and_canvas_drafts_survive": not rich_host_accepted,
        "live_transcript_and_tool_stream_continue_across_toggle": not rich_host_accepted,
        "rich_work_surfaces_rehydrate": not rich_host_accepted,
        "uiai_engine_eval_proof_exists": not rich_host_accepted,
    },
    "implementation_refs": [
        "apps/pi-extension/src/config.ts",
        "apps/pi-extension/src/commands.ts",
        "apps/pi-extension/src/turns.ts",
    ],
    "existing_proof_refs_reclassified_as_terminal_foundation": [
        "apps/pi-extension/tests/mission-canvas-mode-precedence.test.mjs",
        "tests/spec135_m2_pi_work_rail_test.py",
    ],
    "accepted": rich_host_accepted,
}
OUT.write_text(json.dumps(contract, indent=2) + "\n")
print(
    "Spec 135 interaction mode/host renderer proof generated: "
    + ("VERIFIED" if rich_host_accepted else "PARTIAL")
)
