#!/usr/bin/env python3
"""Spec98/99 Phase A: unscoped canonical/action-affecting path inventory coverage."""

from pathlib import Path
import re
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "docs/worksheets/focusa-877z.19-unscoped-canonical-paths.yaml"
SCAN_GLOBS = [
    "crates/focusa-api/src/server.rs",
    "crates/focusa-api/src/routes/*.rs",
    "crates/focusa-cli/src/main.rs",
    "crates/focusa-cli/src/api_client.rs",
    "apps/pi-extension/src/*.ts",
]
PATTERNS = {
    "state_write_lock": re.compile(r"focusa\.write\(\)|state\.focusa\.write\(\)"),
    "reducer_dispatch": re.compile(
        r"\.send\(\s*(?:focusa_core::types::)?Action::|Action::[A-Z]"
    ),
    "singleton_active_pointer": re.compile(
        r"active_frame_id|focus_stack\.active_id|active_workpoint|current_task|active_writer|lastTrajectoryClarity|lastWorkpoint"
    ),
    "pi_focusa_mutating_call": re.compile(
        r"focusaFetch\(\"/(?:focus|ascc|workpoint|trajectory|work-loop|evidence|visual-workflow|telemetry|project|metacog|predict)"
    ),
}
REQUIRED_ENTRY_FIELDS = {
    "path",
    "surface",
    "risk",
    "status",
    "matched_patterns",
    "required_scope_fields",
    "required_classification",
    "mutation_class",
    "scope_status",
    "hard_fail_strategy",
    "required_envelope_fields",
    "agent_handicap_risk",
    "proof_status",
    "implementation_beads",
    "notes",
}


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def candidate_paths() -> set[str]:
    paths: set[str] = set()
    for glob in SCAN_GLOBS:
        for path in ROOT.glob(glob):
            text = path.read_text(errors="ignore")
            if any(pattern.search(text) for pattern in PATTERNS.values()):
                paths.add(str(path.relative_to(ROOT)))
    return paths


def main() -> None:
    if not INVENTORY.exists():
        fail(f"inventory missing: {INVENTORY}")
    data = yaml.safe_load(INVENTORY.read_text())
    if data.get("schema_version") != "focusa.unscoped_canonical_path_inventory.v1":
        fail("unexpected inventory schema_version")
    if data.get("work_item_id") != "focusa-877z.19":
        fail("inventory is not linked to focusa-877z.19")
    if data.get("status") != "phase_1_inventory_covered_static":
        fail("inventory status is not phase_1_inventory_covered_static")
    entries = data.get("entries") or []
    if not entries:
        fail("inventory entries are empty")
    indexed = {entry.get("path"): entry for entry in entries}
    missing = sorted(candidate_paths() - set(indexed))
    if missing:
        fail(f"static candidates missing inventory entries: {missing[:20]}")
    for entry in entries:
        missing_fields = sorted(REQUIRED_ENTRY_FIELDS - set(entry))
        if missing_fields:
            fail(f"{entry.get('path')} missing fields {missing_fields}")
        scope_fields = set(entry.get("required_scope_fields") or [])
        if {"project_root", "continuity_id"} - scope_fields:
            fail(
                f"{entry.get('path')} missing required project_root+continuity_id scope fields"
            )
        if "focusa-877z.19" not in (entry.get("implementation_beads") or []):
            fail(f"{entry.get('path')} missing focusa-877z.19 link")
        envelope = set(entry.get("required_envelope_fields") or [])
        for field in [
            "status",
            "canonical",
            "advisory",
            "degraded",
            "stale",
            "scope_status",
            "failure_class",
            "next_tools",
        ]:
            if field not in envelope:
                fail(f"{entry.get('path')} missing envelope field {field}")
    policy = data.get("next_hard_fail_policy") or {}
    for key in ["phase_1", "phase_2", "phase_3"]:
        if not policy.get(key):
            fail(f"next_hard_fail_policy missing {key}")
    print(
        f"✓ PASS: unscoped canonical/action-affecting inventory covers {len(entries)} files"
    )


if __name__ == "__main__":
    main()
