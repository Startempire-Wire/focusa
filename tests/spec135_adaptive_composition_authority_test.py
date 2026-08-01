#!/usr/bin/env python3
"""P01 authority gate for adaptive Pi-native Mission Canvas composition."""
from __future__ import annotations

import hashlib
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[1]
HANDOFF = ROOT / "docs/contracts/spec135/authoritative-handoff/spec135_agent_handoff_apple_principles.md"
ACTIVITY_IMAGE = ROOT / "docs/contracts/spec135/authoritative-handoff/focusa_activity_mode_recomposition.png"
VERTICAL_IMAGE = ROOT / "docs/contracts/spec135/authoritative-handoff/focusa_dynamic_vertical_recomposition.png"
HOST_PATH = ROOT / "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml"
HOST = yaml.safe_load(HOST_PATH.read_text())
VIEW = (ROOT / "apps/pi-extension/src/mission-canvas-view.ts").read_text()
SHELL = (ROOT / "apps/pi-extension/src/mission-canvas-shell.ts").read_text()
TOOL = (ROOT / "apps/pi-extension/src/mission-canvas-tool.ts").read_text()

assert HANDOFF.exists() and ACTIVITY_IMAGE.exists() and VERTICAL_IMAGE.exists()
assert hashlib.sha256(ACTIVITY_IMAGE.read_bytes()).hexdigest() == "e7a116b47a77eb8b8bae6ebd3cf048146fa5217e72f27d8f126e89e7f7faba93"
assert hashlib.sha256(VERTICAL_IMAGE.read_bytes()).hexdigest() == "a53ba95b3f411a76c75d2a46ecaa206f58700e8646c6dc85846dca39d0d18763"

authority = HOST["authority"]["adaptive_composition_authority"]
assert authority["replacement_text"].endswith("spec135_agent_handoff_apple_principles.md")
assert authority["precedence"].startswith("operator_steering")
assert HOST["authority"]["operator_clarification"]["statement"].startswith("Mission Canvas switches on and off inside the current Pi terminal")

renderer = HOST["required_host_renderer"]
assert renderer["id"] == "pi_native_mission_canvas"
assert renderer["package"] == "apps/pi-extension"
assert renderer["language"] == "TypeScript"
assert renderer["mounting_api"] == "ctx.ui.custom"
assert renderer["process_boundary"] == "current_pi_process"
assert renderer["terminal_boundary"] == "current_pi_terminal"
assert renderer["separate_window_created"] is False
assert renderer["browser_created"] is False
assert renderer["remote_host_created"] is False

for forbidden in ["launch_browser_for_canvas", "launch_webview_for_canvas", "launch_tauri_sidecar_for_canvas", "launch_remote_canvas_host"]:
    assert forbidden in HOST["forbidden_host_behavior"]

assert HOST["adaptive_composition"]["output"] == "ResolvedWorkspaceProjection"
for rule in ["omit_empty_optional_contributions_before_layout", "remove_heading_border_and_spacing_with_omitted_contribution", "reflow_remaining_contributions_deterministically", "preserve_per_profile_layout_memory"]:
    assert rule in HOST["adaptive_composition"]["occupancy_law"]

for activity in HOST["activity_modes"]:
    assert f'"{activity}"' in VIEW
for marker in ["resolveContributions", "meaningful", "renderContributionGrid", "layoutMemory", "Pi Transcript · live", "Evidence Matrix"]:
    assert marker in VIEW, marker
assert "Authoritative Pi-native Mission Canvas" in SHELL
assert "RichHostLifecycleManager" not in TOOL

print("Spec 135 adaptive-composition authority: PASS (Pi-native current-terminal renderer)")
