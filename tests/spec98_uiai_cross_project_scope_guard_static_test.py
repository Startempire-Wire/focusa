#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.11 UIAI cross-project scope guard."""
from pathlib import Path
import json
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/focusa-uiai-scope-guard"
DOC = ROOT / "docs/current/FOCUSA_UIAI_CROSS_PROJECT_SCOPE_GUARD.md"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"
PROOF_SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_proof_suite_static_test.py"
EXPECTED_ROOT = "/home/wirebot/focusa"
EXPECTED_CONT = "focusa-cont-focusa-9a80fe5f-1b3b-4fbe-91d6-958fc38aace6"


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def run(packet: dict) -> tuple[int, str]:
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as tmp:
        json.dump(packet, tmp)
        path = tmp.name
    result = subprocess.run([
        str(SCRIPT), path,
        "--expected-project-root", EXPECTED_ROOT,
        "--expected-continuity-id", EXPECTED_CONT,
        "--json",
    ], cwd=ROOT, capture_output=True, text=True)
    return result.returncode, result.stdout + result.stderr


def main() -> None:
    text = SCRIPT.read_text()
    for term in [
        "cross_project_uiai_scope_guard",
        "cross_workstream_uiai_scope_guard",
        "scope_source_not_focusa_verified",
        "expected_project_root_unverified_or_unsafe",
        "capture_status",
        "scope_mismatch",
        "tool_result_v1",
        "do_not_retry_unchanged",
    ]:
        if term not in text:
            fail(f"scope guard missing {term}")
    doc = DOC.read_text()
    for term in ["project_root + continuity_id", "scope_source", "failure_class=scope_mismatch", "cannot be rendered as captured Focusa evidence"]:
        if term not in doc:
            fail(f"doc missing {term}")

    ok_code, ok_out = run({"scope": {"project_root": EXPECTED_ROOT, "continuity_id": EXPECTED_CONT, "scope_source": "focusa_verified"}})
    if ok_code != 0 or '"status": "verified"' not in ok_out or '"capture_status": "capture_pending"' not in ok_out:
        fail(f"verified packet did not pass: {ok_out}")
    bad_project_code, bad_project_out = run({"scope": {"project_root": "/home/other/project", "continuity_id": EXPECTED_CONT, "scope_source": "focusa_verified"}})
    if bad_project_code == 0 or "cross_project_uiai_scope_guard" not in bad_project_out or '"failure_class": "scope_mismatch"' not in bad_project_out:
        fail(f"cross-project packet not blocked: {bad_project_out}")
    bad_cont_code, bad_cont_out = run({"scope": {"project_root": EXPECTED_ROOT, "continuity_id": "other-cont", "scope_source": "focusa_verified"}})
    if bad_cont_code == 0 or "cross_workstream_uiai_scope_guard" not in bad_cont_out:
        fail(f"cross-workstream packet not blocked: {bad_cont_out}")
    unverified_code, unverified_out = run({"scope": {"project_root": EXPECTED_ROOT, "continuity_id": EXPECTED_CONT, "scope_source": "caller_supplied"}})
    if unverified_code == 0 or "scope_source_not_focusa_verified" not in unverified_out:
        fail(f"unverified source packet not blocked: {unverified_out}")

    if "tests/spec98_uiai_cross_project_scope_guard_static_test.py" not in SUITE.read_text():
        fail("Spec98 suite does not run UIAI cross-project scope guard")
    if "tests/spec98_uiai_cross_project_scope_guard_static_test.py" not in PROOF_SUITE.read_text():
        fail("proof suite static contract does not include UIAI cross-project scope guard")
    print("✓ PASS: Spec98 UIAI cross-project scope guard ok")


if __name__ == "__main__":
    main()
