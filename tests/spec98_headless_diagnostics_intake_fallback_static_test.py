#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.8 headless diagnostics intake fallback guard."""

from pathlib import Path
import json
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/focusa-headless-diagnostics-intake"
DOC = ROOT / "docs/current/FOCUSA_HEADLESS_DIAGNOSTICS_INTAKE_FALLBACK.md"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"
PROOF_SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_proof_suite_static_test.py"


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    text = SCRIPT.read_text()
    for term in [
        "focusa.headless_diagnostics_intake_fallback.v1",
        "capture_status",
        "scope_source",
        "headless_next_action",
        "fallback_commands",
        "focusa_evidence_capture",
        "focusa_browser_diagnostics_intake",
        '"canonical": False',
        '"advisory": True',
        "scope_verification_required",
        "tool_result_v1",
    ]:
        if term not in text:
            fail(f"script missing term: {term}")
    doc = DOC.read_text()
    for term in [
        "No modal/select/input UI",
        "proposal-only",
        "scope_verification_required",
        "focusa_evidence_capture",
        "verified project_root + continuity_id",
    ]:
        if term not in doc:
            fail(f"doc missing term: {term}")

    packet = {
        "capture_status": "proposal_only",
        "scope_source": "focusa_verified",
        "project_root": "/home/wirebot/focusa",
        "continuity_id": "focusa-cont-test",
        "target_ref": "https://example.test",
        "summary_line": "UIAI packet mode=diagnostics evidence=1 scope=verified scope_source=focusa_verified capture=proposal_only tool=focusa_browser_diagnostics_intake next=focusa_evidence_capture",
        "evidence_ref": "uiai-diagnostics:session=test:seq=1",
    }
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as tmp:
        json.dump(packet, tmp)
        tmp_path = tmp.name
    ready = subprocess.run(
        [str(SCRIPT), tmp_path, "--json"], cwd=ROOT, capture_output=True, text=True
    )
    if (
        ready.returncode != 0
        or '"status": "ready"' not in ready.stdout
        or '"canonical": false' not in ready.stdout
    ):
        fail(
            f"ready packet did not render expected fallback JSON: {ready.stderr}\n{ready.stdout}"
        )

    missing = subprocess.run(
        [str(SCRIPT), "--json"],
        input=json.dumps({"target_ref": "missing-scope"}),
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if (
        missing.returncode == 0
        or "scope_verification_required" not in missing.stdout
        or "focusa_project_identity" not in missing.stdout
    ):
        fail("missing-scope packet did not fail closed with scope verification command")

    if (
        "tests/spec98_headless_diagnostics_intake_fallback_static_test.py"
        not in SUITE.read_text()
    ):
        fail("Spec98 suite does not run headless diagnostics fallback guard")
    if (
        "tests/spec98_headless_diagnostics_intake_fallback_static_test.py"
        not in PROOF_SUITE.read_text()
    ):
        fail(
            "proof suite static contract does not include headless diagnostics fallback guard"
        )
    print("✓ PASS: Spec98 headless diagnostics intake fallback ok")


if __name__ == "__main__":
    main()
