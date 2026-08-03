#!/usr/bin/env python3
"""Generate the Spec 135 Pi-native interaction-mode truth contract."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/contracts/spec135-interaction-mode-toggle.v1.json"
GUI_PROOF_PATH = ROOT / "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json"
GUI_PROOF = json.loads(GUI_PROOF_PATH.read_text())
accepted = bool(GUI_PROOF.get("accepted")) and GUI_PROOF.get("runtime", {}).get("observed_host_renderer") == "pi_native_mission_canvas"

contract = {
    "schema": "focusa.spec135.interaction_mode_toggle.v2",
    "status": "verified" if accepted else "blocked",
    "acceptance_criteria": "Canvas ON/OFF replaces or restores the visible root in the current Pi terminal while preserving the same Pi process, session, model stream, tools, transcript, drafts, attachments, and canonical Focusa runtime.",
    "authority_ref": "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml",
    "interaction_modes": ["canvas-guided", "terminal-guided", "headless"],
    "host_renderers": ["pi_native_mission_canvas", "stock_pi", "headless_none"],
    "precedence": ["temporary-session override", "project preference", "user preference", "environment", "default canvas-guided"],
    "foundations_proven": {
        "durable_mode": True,
        "source_displayed": True,
        "scope_exact": True,
        "refresh_immediate": True,
        "resume_survives": True,
        "reconnect_survives": True,
        "headless_no_ui_calls": True,
        "same_pi_process": True,
        "same_pi_terminal": True,
        "browser_or_remote_host_launched": False,
        "state_loss_observed": False,
        "nag_on_headless": False,
    },
    "pi_native_proof": {
        "evidence_ref": "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json",
        "accepted": accepted,
    },
    "not_yet_proven": {} if accepted else {
        "pi_native_canvas_mounts": True,
        "canvas_off_restores_same_pi_root": True,
        "unsent_drafts_survive": True,
        "live_transcript_and_tools_continue": True,
    },
    "implementation_refs": [
        "apps/pi-extension/src/config.ts",
        "apps/pi-extension/src/commands.ts",
        "apps/pi-extension/src/mission-canvas-tool.ts",
        "apps/pi-extension/src/mission-canvas-shell.ts",
        "apps/pi-extension/src/mission-canvas-view.ts",
    ],
    "proof_refs": [
        "apps/pi-extension/tests/mission-canvas-mode-precedence.test.mjs",
        "apps/pi-extension/tests/mission-canvas-pi-surface.test.mjs",
        "apps/pi-extension/tests/mission-canvas-reference-design.test.mjs",
        "docs/evidence/spec135-pi-native-reference-renders.png",
    ],
    "accepted": accepted,
}
OUT.write_text(json.dumps(contract, indent=2) + "\n")
print("Spec 135 Pi-native interaction mode proof generated: " + ("VERIFIED" if accepted else "BLOCKED"))
