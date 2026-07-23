#!/usr/bin/env python3
"""Spec98 / focusa-877z.17 policy profile defaults and zero-friction enforcement guard."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKSHEET = ROOT / "docs/worksheets/focusa-877z.17-policy-profiles-defaults.yaml"
TAXONOMY = ROOT / "docs/worksheets/focusa-877z.8-authority-taxonomy.yaml"
SPEC98 = ROOT / "docs/98-project-root-crdt-reconciliation-foundation-spec.md"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"

REQUIRED_PROFILES = {
    "safe_default",
    "builder",
    "audit_strict",
    "lowmem",
    "browser_debug",
    "headless_ci",
    "demo_noncanonical",
}
REQUIRED_POSTURE_KEYS = {
    "missing_project_or_continuity",
    "uiai_research_packet",
    "uiai_focusa_scope",
    "workpoint_resume_scope_match",
    "trajectory_ladder",
    "ontology_read_indexes",
    "telemetry_resource_uiai_pressure",
    "raw_blobs_logs_screenshots",
    "synthetic_ids",
    "direct_api_state_write",
}
REQUIRED_OVERRIDE_FIELDS = {
    "profile_id",
    "override_source",
    "changed_field_or_route",
    "reason",
    "affected_surfaces",
    "agent_benefit",
    "handicap_risk",
    "rollback_or_default_restore_path",
    "proof_command_or_manual_acceptance_gate",
}
REQUIRED_ZERO_FRICTION = {
    "worksheet_lint",
    "docs_render",
    "route_scaffold",
    "tool_scaffold",
    "proof_bundle",
    "compact_output",
}
REQUIRED_INHERITANCE_PHRASES = [
    "New canonical route defaults to audit_strict",
    "New UIAI/browser workflow defaults to browser_debug",
    "New CI/MCP/CLI surface defaults to headless_ci",
    "LowMem/resource pressure changes inherit lowmem",
]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    if not WORKSHEET.exists():
        fail(f"worksheet missing: {WORKSHEET}")
    data = yaml.safe_load(WORKSHEET.read_text())
    if data.get("schema_version") != "focusa.policy_profiles_defaults.v1":
        fail("unexpected worksheet schema_version")
    if data.get("status") != "implementation_ready":
        fail("worksheet status must be implementation_ready")

    posture = set((data.get("default_authority_posture") or {}).keys())
    missing_posture = REQUIRED_POSTURE_KEYS - posture
    if missing_posture:
        fail(f"missing default posture keys: {sorted(missing_posture)}")

    profiles = data.get("profile_registry") or {}
    missing_profiles = REQUIRED_PROFILES - set(profiles)
    if missing_profiles:
        fail(f"missing profiles: {sorted(missing_profiles)}")
    for profile_id, profile in profiles.items():
        for field in [
            "intended_use",
            "default_classification",
            "automatic_enforcement",
            "override_allowed",
        ]:
            if field not in profile or profile.get(field) in (None, "", []):
                fail(f"profile {profile_id} missing {field}")
        if profile.get("override_allowed") is True and not profile.get(
            "override_requirement"
        ):
            fail(f"profile {profile_id} allows override without override_requirement")

    override_fields = set(
        (data.get("advanced_override_schema") or {}).get("required_fields") or []
    )
    missing_override = REQUIRED_OVERRIDE_FIELDS - override_fields
    if missing_override:
        fail(f"missing override audit fields: {sorted(missing_override)}")
    forbidden = set(
        (data.get("advanced_override_schema") or {}).get("forbidden_override_effects")
        or []
    )
    for required in [
        "erase_scope_validation",
        "hide_canonical_advisory_degraded_labels",
        "remove_evidence_handle_requirements",
        "drop_proof_requirements",
    ]:
        if required not in forbidden:
            fail(f"missing forbidden override effect: {required}")

    zero = set((data.get("zero_friction_enforcement_model") or {}).keys())
    missing_zero = REQUIRED_ZERO_FRICTION - zero
    if missing_zero:
        fail(f"missing zero-friction mechanisms: {sorted(missing_zero)}")
    zero_text = yaml.safe_dump(data.get("zero_friction_enforcement_model") or {})
    for phrase in [
        "focusa taxonomy lint",
        "focusa taxonomy render-docs",
        "focusa new-route",
        "focusa new-tool",
        "focusa proof surface",
        "traffic_light_summary_plus_expandable_detail",
    ]:
        if phrase not in zero_text:
            fail(f"zero-friction model missing target: {phrase}")

    worksheet_text = WORKSHEET.read_text()
    for phrase in REQUIRED_INHERITANCE_PHRASES:
        if phrase not in worksheet_text:
            fail(f"inheritance rule missing phrase: {phrase}")

    proof = yaml.safe_dump(data.get("proof_matrix") or {})
    for expected in [
        "tests/spec98_policy_profiles_defaults_static_test.py",
        "tests/spec98_authority_taxonomy_worksheet_static_test.py",
        "npm --prefix apps/pi-extension run check",
    ]:
        if expected not in proof:
            fail(f"proof matrix missing {expected}")

    taxonomy = yaml.safe_load(TAXONOMY.read_text())
    taxonomy_profiles = set((taxonomy.get("default_profiles") or {}).keys())
    missing_taxonomy_profiles = REQUIRED_PROFILES - taxonomy_profiles
    if missing_taxonomy_profiles:
        fail(
            f"authority taxonomy missing profiles: {sorted(missing_taxonomy_profiles)}"
        )
    items = taxonomy.get("items") or []
    if not any(item.get("id") == "policy_profiles.registry" for item in items):
        fail("authority taxonomy missing policy_profiles.registry item")

    spec98_text = SPEC98.read_text()
    for phrase in [
        "Opinionated defaults",
        "Named policy profiles",
        "Advanced overrides",
        "Zero-friction implementation model",
    ]:
        if phrase not in spec98_text:
            fail(f"Spec98 missing supporting phrase: {phrase}")

    if "tests/spec98_policy_profiles_defaults_static_test.py" not in SUITE.read_text():
        fail("Spec98 regression suite does not run policy profiles guard")

    print("✓ PASS: Spec98 policy profiles/defaults contract ok")


if __name__ == "__main__":
    main()
