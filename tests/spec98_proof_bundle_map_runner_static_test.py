#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.6 proof bundle map runner guard."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts/focusa-proof-bundle"
DOC = ROOT / "docs/current/FOCUSA_PROOF_BUNDLE_MAP_RUNNER.md"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"
PROOF_SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_proof_suite_static_test.py"


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    text = RUNNER.read_text()
    for term in [
        "docs/worksheets/focusa-877z.8-authority-taxonomy.yaml",
        "docs/worksheets/focusa-877z.18-migration-side-effect-proof-plan.yaml",
        "FOCUSA_POLICY_PROFILE_REGISTRY.json",
        "SURFACE_ALIASES",
        "resolve_changed_targets",
        "missing_proof_mapping",
        "--changed-path",
        "--run",
        "as-user wpuiai",
    ]:
        if term not in text:
            fail(f"runner missing term: {term}")

    doc = DOC.read_text()
    for term in ["scripts/focusa-proof-bundle", "api_routes", "policy_profiles.registry", "--changed-path", "Failure rule"]:
        if term not in doc:
            fail(f"doc missing term: {term}")

    for command in [
        [str(RUNNER), "api_routes", "--json"],
        [str(RUNNER), "policy_profiles.registry", "--json"],
        [str(RUNNER), "--changed-path", "crates/focusa-api/src/routes/workpoint.rs", "--json"],
    ]:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        if result.returncode != 0 or '"status": "ok"' not in result.stdout:
            fail(f"runner command failed: {' '.join(command)}\n{result.stderr}")

    missing = subprocess.run([str(RUNNER), "missing.surface"], cwd=ROOT, capture_output=True, text=True)
    if missing.returncode == 0 or "missing proof mapping" not in missing.stderr:
        fail("runner does not fail closed for missing target")

    if "tests/spec98_proof_bundle_map_runner_static_test.py" not in SUITE.read_text():
        fail("Spec98 suite does not run proof bundle map runner guard")
    if "tests/spec98_proof_bundle_map_runner_static_test.py" not in PROOF_SUITE.read_text():
        fail("proof suite static contract does not include proof bundle map runner guard")

    print("✓ PASS: Spec98 proof bundle map runner ok")


if __name__ == "__main__":
    main()
