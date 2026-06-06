#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.4 authority migration/backcompat implementation guard."""
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
WORKPOINT = ROOT / "crates/focusa-api/src/routes/workpoint.rs"
ECS = ROOT / "crates/focusa-api/src/routes/ecs.rs"
SNAPSHOTS = ROOT / "crates/focusa-api/src/routes/snapshots.rs"
FOCUS = ROOT / "crates/focusa-api/src/routes/focus.rs"
UIAI = ROOT / "docs/worksheets/focusa-877z.15-uiai-packet-capture-headless.yaml"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def require(path: Path, terms: list[str], label: str) -> None:
    text = path.read_text()
    for term in terms:
        if term not in text:
            fail(f"{label} missing {term}")


def main() -> None:
    require(WORKPOINT, [
        "workpoint_legacy_migration_warnings",
        "old_workpoint_packet_missing_scope",
        "readable_as_degraded_advisory_recovery_packet",
        "canonical_false_until_project_root_plus_continuity_id_rebound",
        "migration_posture",
        "advisory",
        "stale",
        "scope_status",
        "legacy_or_request_scope",
    ], "Workpoint resume")

    require(ECS, [
        "handle_legacy_migration_warnings",
        "evidence_handle_scope_missing",
        "legacy_scope_missing",
        "evidence_handle_only_not_object_truth",
        "readable_via_handle_id_with legacy_scope_missing warning",
        "legacy_handle_metadata",
        "tool_result_v1",
    ], "ECS handles")

    require(SNAPSHOTS, [
        "snapshot_authority_posture",
        "old_snapshots_and_clt_nodes",
        "readable_history_only",
        "lineage_not_current_action_authority",
        "clt_snapshot_authority_unscoped",
        "legacy_snapshot_or_lineage_record",
    ], "Snapshots/CLT")

    require(FOCUS, [
        "focus_frame_legacy_migration_warnings",
        "focus_state_legacy_scope_inferred",
        "focus_frame_missing_beads_source",
        "canonical_false_for_synthetic_or_missing_beads",
        "old_focus_state_records_and_focus_stack_frames",
        "legacy_migration_policy",
    ], "Focus State/frames")

    require(UIAI, [
        "proposal_only",
        "degraded_unknown",
        "capture_status",
        "scope_source",
        "Focusa verification required before canonical capture",
    ], "UIAI packet worksheet")

    if "tests/spec98_authority_migration_backcompat_static_test.py" not in SUITE.read_text():
        fail("Spec98 regression suite does not run authority migration/backcompat guard")

    print("✓ PASS: Spec98 authority migration/backcompat labels are wired")


if __name__ == "__main__":
    main()
