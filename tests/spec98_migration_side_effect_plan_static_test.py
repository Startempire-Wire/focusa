#!/usr/bin/env python3
"""Spec98 / focusa-877z.18 migration backcompat + side-effect proof plan guard."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKSHEET = (
    ROOT / "docs/worksheets/focusa-877z.18-migration-side-effect-proof-plan.yaml"
)
SPEC98 = ROOT / "docs/98-project-root-crdt-reconciliation-foundation-spec.md"
TAXONOMY = ROOT / "docs/worksheets/focusa-877z.8-authority-taxonomy.yaml"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"

REQUIRED_MIGRATION_ITEMS = {
    "old_workpoint_packets",
    "old_trajectory_packets",
    "old_focus_state_records",
    "old_focus_stack_frames",
    "old_uiai_packets",
    "old_evidence_handles",
    "old_reference_store_records",
    "old_snapshots_and_clt_nodes",
}
REQUIRED_SIDE_EFFECTS = {
    "read_only",
    "advisory_projection",
    "runtime_cache",
    "telemetry_event",
    "evidence_write",
    "reducer_event",
    "external_io",
    "destructive_or_service_control",
}
REQUIRED_PROOF_SURFACES = {
    "daemon_core",
    "api_routes",
    "cli",
    "pi_extension",
    "uiai_external",
    "menubar",
    "proof_suite",
}
REQUIRED_EXPECTED_SIDE_EFFECTS = {
    "stricter_envelopes",
    "degraded_legacy_packets",
    "scoped_evidence_handles",
    "ui_packet_capture_rendering",
    "headless_schema_first",
    "side_effect_lint",
}


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    if not WORKSHEET.exists():
        fail(f"worksheet missing: {WORKSHEET}")
    data = yaml.safe_load(WORKSHEET.read_text())
    if data.get("schema_version") != "focusa.migration_side_effect_proof_plan.v1":
        fail("unexpected worksheet schema_version")
    if data.get("status") != "implementation_ready":
        fail("worksheet status must be implementation_ready")

    migration = data.get("migration_backcompat_matrix") or {}
    missing_migration = REQUIRED_MIGRATION_ITEMS - set(migration)
    if missing_migration:
        fail(f"missing migration items: {sorted(missing_migration)}")
    for item_id, item in migration.items():
        for field in [
            "legacy_shape",
            "read_behavior",
            "authority_status",
            "migration_warning",
            "promotion_path",
            "proof_requirements",
        ]:
            if field not in item or item.get(field) in (None, "", []):
                fail(f"migration item {item_id} missing {field}")

    side_effects = data.get("side_effect_classification") or {}
    missing_side_effects = REQUIRED_SIDE_EFFECTS - set(side_effects)
    if missing_side_effects:
        fail(f"missing side-effect classes: {sorted(missing_side_effects)}")
    for class_id, item in side_effects.items():
        for field in ["meaning", "examples", "proof"]:
            if field not in item or item.get(field) in (None, "", []):
                fail(f"side-effect class {class_id} missing {field}")

    expected = data.get("expected_side_effects") or {}
    missing_expected = REQUIRED_EXPECTED_SIDE_EFFECTS - set(expected)
    if missing_expected:
        fail(f"missing expected side effects: {sorted(missing_expected)}")
    for effect_id, item in expected.items():
        for field in ["positive", "risk", "mitigation"]:
            if field not in item or item.get(field) in (None, "", []):
                fail(f"expected side effect {effect_id} missing {field}")

    proof_map = data.get("proof_bundle_map") or {}
    missing_surfaces = REQUIRED_PROOF_SURFACES - set(proof_map)
    if missing_surfaces:
        fail(f"missing proof bundle surfaces: {sorted(missing_surfaces)}")
    proof_text = yaml.safe_dump(proof_map)
    for command in [
        "cargo test -p focusa-core sync::crdt",
        "cargo check -p focusa-api",
        "tests/focusa_cli_parity_smoke_test.sh",
        "npm --prefix apps/pi-extension run check",
        "scripts/check-focusa-packet-drift.sh",
        "npm --prefix apps/menubar run typecheck",
        "tests/spec98_runtime_bleed_crdt_regression_suite.sh",
    ]:
        if command not in proof_text:
            fail(f"proof bundle map missing command: {command}")

    worksheet_text = WORKSHEET.read_text()
    for phrase in [
        "readable_as_degraded_advisory_recovery_packet",
        "proposal_only_capture_unknown",
        "legacy_scope_missing",
        "lineage_not_current_action_authority",
        "no FocusaState cognition version bump",
        "operator approval",
    ]:
        if phrase not in worksheet_text:
            fail(f"worksheet missing required phrase: {phrase}")

    spec98_text = SPEC98.read_text()
    for phrase in [
        "Expected side effect",
        "Migration of old packets/snapshots",
        "Side-effect classification tests",
        "Proof bundle map runner",
    ]:
        if phrase not in spec98_text:
            fail(f"Spec98 missing supporting phrase: {phrase}")

    taxonomy = yaml.safe_load(TAXONOMY.read_text())
    ids = {item.get("id") for item in taxonomy.get("items") or []}
    for required in [
        "side_effects.classification",
        "reference_store.evidence_handles",
        "uiai.research_diagnostics_packet",
    ]:
        if required not in ids:
            fail(f"authority taxonomy missing related item: {required}")

    if (
        "tests/spec98_migration_side_effect_plan_static_test.py"
        not in SUITE.read_text()
    ):
        fail("Spec98 regression suite does not run migration/side-effect plan guard")

    print("✓ PASS: Spec98 migration/backcompat and side-effect proof plan ok")


if __name__ == "__main__":
    main()
