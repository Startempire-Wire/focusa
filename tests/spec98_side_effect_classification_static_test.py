#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.13 side-effect classification guard."""

from pathlib import Path
import json
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
PLAN = ROOT / "docs/worksheets/focusa-877z.18-migration-side-effect-proof-plan.yaml"
TAXONOMY = ROOT / "docs/worksheets/focusa-877z.8-authority-taxonomy.yaml"
REGISTRY = ROOT / "docs/current/FOCUSA_AUTHORITY_SURFACE_REGISTRY.generated.json"
TOOL_CONTRACTS = ROOT / "docs/current/focusa-tool-contracts.json"
SHARED = ROOT / "docs/current/SHARED_TOOL_RESULT_ENVELOPE_STUBS.md"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"
PROOF_SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_proof_suite_static_test.py"

REQUIRED_CLASSES = {
    "read_only",
    "advisory_projection",
    "runtime_cache",
    "telemetry_event",
    "evidence_write",
    "reducer_event",
    "external_io",
    "destructive_or_service_control",
}
REQUIRED_ACCEPTANCE_ALIASES = {
    "read_only",
    "advisory",
    "runtime_cache",
    "telemetry",
    "evidence_write",
    "reducer_event",
    "external_io",
    "destructive",
}


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    plan = yaml.safe_load(PLAN.read_text())
    classes = plan.get("side_effect_classification") or {}
    missing = REQUIRED_CLASSES - set(classes)
    if missing:
        fail(f"side_effect_classification missing classes: {sorted(missing)}")
    for cls in REQUIRED_CLASSES:
        block = classes.get(cls) or {}
        for field in ["meaning", "examples", "proof"]:
            if block.get(field) in (None, "", []):
                fail(f"side-effect class {cls} missing {field}")
    expected = plan.get("expected_side_effects") or {}
    for key in [
        "stricter_envelopes",
        "degraded_legacy_packets",
        "scoped_evidence_handles",
        "ui_packet_capture_rendering",
        "headless_schema_first",
        "side_effect_lint",
    ]:
        block = expected.get(key) or {}
        for field in ["positive", "risk", "mitigation"]:
            if block.get(field) in (None, "", []):
                fail(f"expected_side_effects.{key} missing {field}")

    taxonomy_text = TAXONOMY.read_text()
    for alias in REQUIRED_ACCEPTANCE_ALIASES:
        if alias not in taxonomy_text:
            fail(f"authority taxonomy missing acceptance side-effect alias {alias}")

    registry = json.loads(REGISTRY.read_text())
    entries = registry.get("entries") or []
    if not entries:
        fail("generated authority registry empty")
    for entry in entries:
        if (
            not entry.get("mutation_class")
            or not entry.get("side_effects")
            or not entry.get("proof_commands")
        ):
            fail(
                f"registry entry lacks side-effect/mutation/proof declaration: {entry.get('worksheet_id')}"
            )

    contract_text = TOOL_CONTRACTS.read_text()
    for profile in [
        "read_only",
        "read_state",
        "advisory_projection",
        "evidence_link",
        "write_state",
        "process_control",
        "control_state",
        "write_prediction",
    ]:
        if profile not in contract_text:
            fail(f"tool contracts missing representative side_effect_profile {profile}")
    if (
        "side_effects" not in SHARED.read_text()
        or "external_io" not in SHARED.read_text()
    ):
        fail("shared tool-result envelope docs do not expose side_effects/external_io")

    if (
        "tests/spec98_side_effect_classification_static_test.py"
        not in SUITE.read_text()
    ):
        fail("Spec98 suite does not run side-effect classification guard")
    if (
        "tests/spec98_side_effect_classification_static_test.py"
        not in PROOF_SUITE.read_text()
    ):
        fail(
            "proof suite static contract does not include side-effect classification guard"
        )
    print("✓ PASS: Spec98 side-effect classification guard ok")


if __name__ == "__main__":
    main()
