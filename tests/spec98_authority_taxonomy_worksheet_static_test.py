#!/usr/bin/env python3
"""Spec98 / focusa-877z.8 authority taxonomy worksheet coverage test."""
from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKSHEET = ROOT / "docs/worksheets/focusa-877z.8-authority-taxonomy.yaml"
REQUIRED_IDS = {
    "trajectory_ladder.hlt",
    "trajectory_ladder.mlg_stg_waypoints",
    "trajectory_ladder.hlt_ledger_md",
    "workpoint.resume_packet",
    "pi_bootstrap.trajectory_ladder_fallback",
    "pi_compaction.trajectory_ladder_fallback",
    "shared.tool_result_envelope",
    "focusa_state.active_turn",
    "focus_stack.active_frame",
    "focus_state.slots",
    "project_identity.scope_envelope",
    "reference_store.evidence_handles",
    "ontology.read_model",
    "uiai.research_diagnostics_packet",
    "telemetry.resource_pressure",
    "menubar.state_contract",
    "policy_profiles.registry",
    "side_effects.classification",
    "work_loop.status_and_writer",
}

def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)

def main() -> None:
    if not WORKSHEET.exists():
        fail(f"worksheet missing: {WORKSHEET}")
    data = yaml.safe_load(WORKSHEET.read_text())
    if data.get("status") != "implementation_ready_seed":
        fail("worksheet status is not implementation_ready_seed")
    items = data.get("items") or []
    ids = {item.get("id") for item in items}
    missing = sorted(REQUIRED_IDS - ids)
    if missing:
        fail(f"missing worksheet ids: {missing}")
    for item in items:
        item_id = item.get("id")
        for field in ["authority_class", "default_profile", "mutation_class", "scope_fields", "affected_surfaces", "proof_commands", "compact_render_required_text"]:
            if field not in item or item.get(field) in (None, "", []):
                fail(f"{item_id} missing required field {field}")
    if "resolved_defaults" not in data:
        fail("resolved_defaults missing")
    if "implementation_readiness_criteria" not in data:
        fail("implementation_readiness_criteria missing")
    print(f"✓ PASS: worksheet coverage ok ({len(items)} items)")

if __name__ == "__main__":
    main()
