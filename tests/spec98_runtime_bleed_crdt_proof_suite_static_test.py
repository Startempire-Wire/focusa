#!/usr/bin/env python3
"""Spec98 Phase H: proof-suite manifest/runner honesty guard."""
from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.26-runtime-bleed-crdt-proof-suite.yaml"
RUNNER = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"
SYNC = ROOT / "crates/focusa-api/src/routes/sync.rs"
PERSISTENCE = ROOT / "crates/focusa-core/src/runtime/persistence_sqlite.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.runtime_bleed_crdt_proof_suite.v1":
        fail("unexpected .26 schema_version")
    if data.get("status") != "proof_suite_defined":
        fail(".26 status must be proof_suite_defined")
    groups = data.get("proof_groups") or {}
    for group in ["project_scope_and_bleed", "partition_contracts", "crdt_foundation", "build_gates"]:
        if group not in groups:
            fail(f"proof group missing {group}")
        if not groups[group].get("commands"):
            fail(f"proof group {group} has no commands")
    gaps = "\n".join(data.get("known_gaps") or [])
    if "writer-claim" in gaps or "active_writer is legacy-global" in gaps:
        fail("writer-claim migration must not remain a known gap")
    runner = RUNNER.read_text()
    for command in [
        "bun tests/spec98_pi_scope_cache_switch_handling_runtime_test.mts",
        "bun tests/pi_project_root_inference_test.mts",
        "tests/spec98_workpoint_trajectory_active_scope_static_test.py",
        "tests/spec98_focus_stack_state_scope_static_test.py",
        "tests/spec98_crdt_event_store_wiring_static_test.py",
        "tests/spec98_pi_uiai_authority_impact_static_test.py",
        "tests/spec98_uiai_packet_capture_headless_static_test.py",
        "tests/spec98_uiai_packet_capture_status_rendering_static_test.py",
        "tests/spec98_headless_diagnostics_intake_fallback_static_test.py",
        "tests/spec98_exact_handle_evidence_write_semantics_static_test.py",
        "tests/spec98_visual_workflow_exact_scope_static_test.py",
        "tests/spec98_policy_profiles_defaults_static_test.py",
        "tests/spec98_policy_profile_registry_impl_static_test.py",
        "tests/spec98_proof_bundle_map_runner_static_test.py",
        "tests/spec98_migration_side_effect_plan_static_test.py",
        "tests/spec98_authority_migration_backcompat_static_test.py",
        "tests/spec98_shared_tool_result_envelope_static_test.py",
        "tests/spec98_menubar_authority_state_contract_static_test.py",
        " test -p focusa-core sync::crdt",
        "tests/spec98_runtime_multi_daemon_crdt_sync_test.sh",
        "npm --prefix apps/pi-extension run check",
        " check",
    ]:
        if command not in runner:
            fail(f"runner missing command: {command}")
    sync = SYNC.read_text()
    persistence = PERSISTENCE.read_text()
    for needle in [
        "/v1/sync/crdt/export",
        "/v1/sync/crdt/import",
        "crdt_events_for_scope",
        "import_crdt_events_same_root",
        "project_root_key",
        "workstream_key",
    ]:
        if needle not in sync:
            fail(f"sync route missing CRDT production surface: {needle}")
    for needle in ["CREATE TABLE IF NOT EXISTS crdt_events", "idx_crdt_events_scope", "append_crdt_event"]:
        if needle not in persistence:
            fail(f"SQLite persistence missing durable CRDT support: {needle}")
    for forbidden_gap in ["active_writer is legacy-global", "remain pending", "Known gaps retained"]:
        if forbidden_gap in runner:
            fail(f"runner still prints deferred gap: {forbidden_gap}")
    print("✓ PASS: Spec98 runtime bleed/CRDT proof suite contract and runner are honest")


if __name__ == "__main__":
    main()
