#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.10 UIAI packet capture-status rendering guard."""
from pathlib import Path
import json
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/focusa-uiai-packet-render"
DOC = ROOT / "docs/current/FOCUSA_UIAI_PACKET_CAPTURE_STATUS_RENDERING.md"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"
PROOF_SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_proof_suite_static_test.py"


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    text = SCRIPT.read_text()
    for term in ["CAPTURE_RENDER", "proposal_only", "capture_pending", "captured", "workpoint_linked", "scope_mismatch", "degraded_unknown", "summary_line", "tool_result_v1", "focusa_browser_diagnostics_intake"]:
        if term not in text:
            fail(f"renderer missing {term}")
    doc = DOC.read_text()
    for term in ["capture=proposal_only", "capture=pending_focusa_tool", "capture=focusa_captured", "capture=workpoint_linked", "capture=rejected", "scope_source=focusa_verified", "canonical=true"]:
        if term not in doc:
            fail(f"doc missing {term}")

    cases = [
        ("proposal_only", "caller_supplied", "capture=proposal_only", False),
        ("capture_pending", "focusa_verified", "capture=pending_focusa_tool", False),
        ("captured", "focusa_verified", "capture=focusa_captured", True),
        ("workpoint_linked", "focusa_verified", "capture=workpoint_linked", True),
        ("scope_mismatch", "mismatch_candidate", "capture=rejected", False),
        ("degraded_unknown", "missing", "capture=degraded_unknown", False),
    ]
    for capture, source, needle, canonical in cases:
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as tmp:
            json.dump({"capture_status": capture, "scope_source": source, "scope_status": "verified" if source == "focusa_verified" else "present", "evidence_ref": "uiai:test"}, tmp)
            path = tmp.name
        result = subprocess.run([str(SCRIPT), path, "--json"], cwd=ROOT, capture_output=True, text=True)
        if result.returncode != 0 or needle not in result.stdout or f'"canonical": {str(canonical).lower()}' not in result.stdout:
            fail(f"render case failed for {capture}/{source}: {result.stdout}\n{result.stderr}")

    if "tests/spec98_uiai_packet_capture_status_rendering_static_test.py" not in SUITE.read_text():
        fail("Spec98 suite does not run UIAI packet capture-status renderer guard")
    if "tests/spec98_uiai_packet_capture_status_rendering_static_test.py" not in PROOF_SUITE.read_text():
        fail("proof suite static contract does not include UIAI packet capture-status renderer guard")
    print("✓ PASS: Spec98 UIAI packet capture-status rendering ok")


if __name__ == "__main__":
    main()
