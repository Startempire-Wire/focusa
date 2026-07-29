#!/usr/bin/env python3
"""Spec 135C-4 UIAI evaluation and failure recovery proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-uiai-evaluation-recovery.v1.json").read_text())
assert C["acceptance_criteria"] == "UIAI evaluations pass production workflows and every failure produces bounded diagnostics and recovery."
assert len(C["evaluation_receipts"]) >= 7
for receipt in C["evaluation_receipts"]:
    assert receipt["status"] == "passed", receipt
    assert receipt["has_diagnostics"]
    assert receipt["browser_session_refs"]
    assert receipt["browser_context_refs"]
    assert (ROOT/receipt["result_ref"]).exists()
assert set(C["diagnostic_categories"]) >= {"visual","console","network","scope"}
assert C["failure_envelope"]["bounded"] is True
assert C["failure_envelope"]["secret_safe"] is True
assert C["failure_envelope"]["raw_page_dump"] is False
for field in ("failure_class","diagnostics_ref","recovery_steps","session_origin","browser_context_ref"):
    assert field in C["failure_envelope"]["required_fields"]
assert "A Focusa link failure must not destroy the UIAI artifact" in C["laws"]
print("Spec 135 C4 UIAI evaluation and failure recovery: PASS")
