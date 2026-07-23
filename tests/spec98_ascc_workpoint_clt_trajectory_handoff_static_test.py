#!/usr/bin/env python3
"""Spec98 focusa-877z.9: ASCC/Workpoint/CLT/Trajectory handoff separation guard."""

from pathlib import Path
import sys
import yaml
import re

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = (
    ROOT
    / "docs/worksheets/focusa-877z.9-ascc-workpoint-clt-trajectory-handoff-contract.yaml"
)
TYPES = ROOT / "crates/focusa-core/src/types.rs"
TRAJECTORY = ROOT / "crates/focusa-api/src/routes/trajectory.rs"
WORKPOINT = ROOT / "crates/focusa-api/src/routes/workpoint.rs"
REPLAY = ROOT / "crates/focusa-core/src/replay/mod.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if (
        data.get("schema_version")
        != "focusa.ascc_workpoint_clt_trajectory_handoff_contract.v1"
    ):
        fail("unexpected .9 contract schema")
    if data.get("status") != "separate_system_handoff_route_defined":
        fail("unexpected .9 contract status")

    types = TYPES.read_text()
    trajectory = TRAJECTORY.read_text()
    workpoint = WORKPOINT.read_text()
    replay = REPLAY.read_text()

    enum_start = types.find("pub enum HandoffSystemRole")
    const_start = types.find("pub const HANDOFF_SYSTEM_ROLE_CONTRACT")
    if enum_start == -1 or const_start == -1:
        fail("types.rs must define HandoffSystemRole and HANDOFF_SYSTEM_ROLE_CONTRACT")
    enum_body = types[enum_start:const_start]
    for variant in [
        "AsccFocusStateSlots",
        "WorkpointContinuationAuthority",
        "CltLineageHistory",
        "TrajectoryRouteGuidance",
    ]:
        if variant not in enum_body:
            fail(f"HandoffSystemRole missing {variant}")
    const_body = types[
        const_start : types.find("pub const FOCUSA_STATE_PLANE_CONTRACT", const_start)
    ]
    for system in ["ascc", "workpoint", "clt", "trajectory_ladder"]:
        pattern = re.compile(rf"\(\s*\"{system}\"\s*,\s*HandoffSystemRole::")
        if not pattern.search(const_body):
            fail(f"HANDOFF_SYSTEM_ROLE_CONTRACT missing {system}")

    for phrase in [
        "Trajectory proposal is advisory; call focusa_workpoint_checkpoint before acting",
        "Trajectory does not auto-promote Workpoints",
        "Do not merge same-high-level sessions without project_root+continuity_id match",
        "canonicalization_tool",
        "focusa_workpoint_checkpoint",
    ]:
        if phrase not in trajectory:
            fail(f"trajectory route missing handoff guard phrase {phrase}")
    if (
        '"next_tools": ["focusa_workpoint_resume", "focusa_active_object_resolve"]'
        not in trajectory
    ):
        fail(
            "trajectory resume must route through Workpoint resume for canonical continuation"
        )
    if '"advisory_only": true' not in trajectory:
        fail(
            "trajectory payloads must expose advisory_only=true where candidates/checkpoints are not authority"
        )
    if '"must_not_merge_sessions"' not in trajectory:
        fail("trajectory similarity grouping must forbid authority merging")

    if "WORKPOINT {}: mission={}; action={}; next={}; canonical={}" not in workpoint:
        fail("workpoint resume summary must remain its own canonical packet surface")
    if "WorkpointResumePacket" not in workpoint and "resume packet" not in workpoint:
        fail("workpoint route must remain explicit continuation packet surface")

    if "WorkpointReplaySummary" not in replay:
        fail("CLT/replay support must remain separate replay summary surface")
    if "Workpoint event summary" not in replay:
        fail("replay must remain history/corroboration, not next-action packet")

    print(
        "✓ PASS: ASCC, Workpoint, CLT, and Trajectory Ladder handoff roles remain separate"
    )


if __name__ == "__main__":
    main()
