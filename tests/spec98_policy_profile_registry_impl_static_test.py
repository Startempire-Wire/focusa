#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.5 policy profile registry + override audit guard."""
from pathlib import Path
import json
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs/current/FOCUSA_POLICY_PROFILE_REGISTRY.json"
WORKSHEET = ROOT / "docs/worksheets/focusa-877z.17-policy-profiles-defaults.yaml"
TAXONOMY = ROOT / "docs/worksheets/focusa-877z.8-authority-taxonomy.yaml"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"

REQUIRED_PROFILES = {"safe_default", "builder", "audit_strict", "lowmem", "browser_debug", "headless_ci", "demo_noncanonical"}
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
REQUIRED_FORBIDDEN = {
    "erase_scope_validation",
    "hide_canonical_advisory_degraded_labels",
    "remove_evidence_handle_requirements",
    "omit_mutation_class",
    "weaken_privacy_or_redaction_boundaries",
    "drop_proof_requirements",
}


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    registry = json.loads(REGISTRY.read_text())
    worksheet = yaml.safe_load(WORKSHEET.read_text())
    taxonomy = yaml.safe_load(TAXONOMY.read_text())

    if registry.get("schema_version") != "focusa.policy_profile_registry.v1":
        fail("unexpected registry schema_version")
    if registry.get("default_profile") != "safe_default":
        fail("registry default_profile must be safe_default")

    profiles = registry.get("profiles") or {}
    missing_profiles = REQUIRED_PROFILES - set(profiles)
    if missing_profiles:
        fail(f"registry missing profiles: {sorted(missing_profiles)}")

    worksheet_profiles = set((worksheet.get("profile_registry") or {}).keys())
    if set(profiles) != worksheet_profiles:
        fail("registry profile ids drift from worksheet")

    for profile_id, profile in profiles.items():
        for field in ["intended_use", "default_classification", "override_allowed", "enforcement"]:
            if profile.get(field) in (None, "", []):
                fail(f"profile {profile_id} missing {field}")
        if profile.get("override_allowed") is True and not profile.get("override_requirement"):
            fail(f"profile {profile_id} allows override without requirement")
        if profile.get("override_allowed") is False and "override_requirement" in profile:
            fail(f"profile {profile_id} should not advertise override requirement when overrides are forbidden")

    audit = registry.get("override_audit_schema") or {}
    fields = set(audit.get("required_fields") or [])
    missing_fields = REQUIRED_OVERRIDE_FIELDS - fields
    if missing_fields:
        fail(f"override audit schema missing fields: {sorted(missing_fields)}")
    forbidden = set(audit.get("forbidden_override_effects") or [])
    missing_forbidden = REQUIRED_FORBIDDEN - forbidden
    if missing_forbidden:
        fail(f"override audit schema missing forbidden effects: {sorted(missing_forbidden)}")

    template = audit.get("audit_record_template") or {}
    template_missing = REQUIRED_OVERRIDE_FIELDS - set(template)
    if template_missing:
        fail(f"audit record template missing fields: {sorted(template_missing)}")
    if not template.get("rollback_or_default_restore_path") or "safe_default" not in template.get("rollback_or_default_restore_path", ""):
        fail("audit template rollback path must name safe_default")
    if not template.get("proof_command_or_manual_acceptance_gate"):
        fail("audit template must include proof command or manual acceptance gate")

    registry_text = REGISTRY.read_text()
    for phrase in [
        "New canonical route defaults to audit_strict",
        "New UIAI/browser workflow defaults to browser_debug",
        "New CI/MCP/CLI surface defaults to headless_ci",
        "New demo/example defaults to demo_noncanonical",
        "LowMem/resource pressure changes inherit lowmem",
    ]:
        if phrase not in registry_text:
            fail(f"registry missing inheritance rule: {phrase}")

    taxonomy_profiles = set((taxonomy.get("default_profiles") or {}).keys())
    if REQUIRED_PROFILES - taxonomy_profiles:
        fail("taxonomy default_profiles missing registry ids")
    if not any((item.get("id") == "policy_profiles.registry" and item.get("proof_commands")) for item in taxonomy.get("items") or []):
        fail("taxonomy policy_profiles.registry item lacks proof_commands")

    for command in [
        "tests/spec98_policy_profiles_defaults_static_test.py",
        "tests/spec98_policy_profile_registry_impl_static_test.py",
        "npm --prefix apps/pi-extension run check",
    ]:
        if command not in registry_text:
            fail(f"registry proof_commands missing {command}")

    if "tests/spec98_policy_profile_registry_impl_static_test.py" not in SUITE.read_text():
        fail("Spec98 suite does not run policy profile registry implementation guard")

    print("✓ PASS: Spec98 policy profile registry and override audit ok")


if __name__ == "__main__":
    main()
