#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.12 generated authority docs and lint path guard."""
from pathlib import Path
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/focusa-authority-taxonomy"
MD = ROOT / "docs/current/FOCUSA_AUTHORITY_TAXONOMY_GENERATED.md"
REG = ROOT / "docs/current/FOCUSA_AUTHORITY_SURFACE_REGISTRY.generated.json"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"
PROOF_SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_proof_suite_static_test.py"


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def run(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(SCRIPT), *args], cwd=ROOT, capture_output=True, text=True)


def main() -> None:
    text = SCRIPT.read_text()
    for term in ["REQUIRED_ITEM_FIELDS", "authority_class", "mutation_class", "scope_fields", "affected_surfaces", "proof_commands", "side_effects_or_render_semantics", "--lint-changed", "ROUTE_TOOL_PREFIXES"]:
        if term not in text:
            fail(f"generator/lint missing {term}")

    lint = run(["--lint"])
    if lint.returncode != 0 or "authority taxonomy lint ok" not in lint.stdout:
        fail(f"taxonomy lint failed: {lint.stdout}\n{lint.stderr}")

    changed_missing = run(["--lint-changed", "crates/focusa-api/src/routes/new_route.rs"])
    if changed_missing.returncode == 0 or "missing --worksheet-id" not in changed_missing.stderr:
        fail(f"route/tool lint did not fail without worksheet id: {changed_missing.stdout}\n{changed_missing.stderr}")

    changed_valid = run(["--lint-changed", "crates/focusa-api/src/routes/new_route.rs", "--worksheet-id", "shared.tool_result_envelope"])
    if changed_valid.returncode != 0 or "route/tool authority lint ok" not in changed_valid.stdout:
        fail(f"route/tool lint did not pass with valid worksheet id: {changed_valid.stdout}\n{changed_valid.stderr}")

    if not MD.exists() or not REG.exists():
        fail("generated authority docs/registry missing")
    md = MD.read_text()
    for term in ["Focusa Authority Taxonomy", "Route/tool additions must cite", "Worksheet ID", "shared.tool_result_envelope"]:
        if term not in md:
            fail(f"generated markdown missing {term}")
    reg = json.loads(REG.read_text())
    if reg.get("schema_version") != "focusa.authority_surface_registry.generated.v1":
        fail("generated registry schema mismatch")
    entries = reg.get("entries") or []
    if not entries or not any(e.get("worksheet_id") == "shared.tool_result_envelope" and e.get("authority_class") for e in entries):
        fail("generated registry lacks shared envelope authority entry")
    for entry in entries:
        for field in ["worksheet_id", "authority_class", "mutation_class", "scope_fields", "affected_surfaces", "side_effects", "proof_commands"]:
            if entry.get(field) in (None, "", []):
                fail(f"generated registry entry missing {field}: {entry.get('worksheet_id')}")

    if "tests/spec98_authority_taxonomy_generated_lint_static_test.py" not in SUITE.read_text():
        fail("Spec98 suite does not run authority taxonomy generated lint guard")
    if "tests/spec98_authority_taxonomy_generated_lint_static_test.py" not in PROOF_SUITE.read_text():
        fail("proof suite static contract does not include authority taxonomy generated lint guard")
    print("✓ PASS: Spec98 authority taxonomy generated docs/lint ok")


if __name__ == "__main__":
    main()
