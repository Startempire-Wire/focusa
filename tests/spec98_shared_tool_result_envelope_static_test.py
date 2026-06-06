#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.3 shared tool result envelope schema/stub guard."""
from pathlib import Path
import sys
import json

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "docs/contracts/focusa-tool-result-schema-v1.json"
STUBS = ROOT / "docs/current/SHARED_TOOL_RESULT_ENVELOPE_STUBS.md"
TOOLS = ROOT / "apps/pi-extension/src/tools.ts"
MENUBAR_API = ROOT / "apps/menubar/src/lib/api.ts"
UIAI_WORKSHEET = ROOT / "docs/worksheets/focusa-877z.15-uiai-packet-capture-headless.yaml"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"

REQUIRED_FIELDS = {"ok", "status", "failure_class", "canonical", "advisory", "degraded", "stale", "summary", "retry", "side_effects", "evidence_refs", "next_tools"}
REQUIRED_STATUS = {"accepted", "completed", "pending", "no_op", "blocked", "validation_rejected", "degraded", "offline", "error"}
REQUIRED_SCOPE_STATUS = {"verified", "present", "missing", "partial", "mismatch_candidate", "unsafe", "unknown"}
REQUIRED_STUB_TERMS = {"API route stub", "CLI JSON stub", "Pi tool wrapper stub", "UIAI packet bridge stub", "Menubar display stub"}


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    schema = json.loads(SCHEMA.read_text())
    required = set(schema.get("required") or [])
    missing_required = REQUIRED_FIELDS - required
    if missing_required:
        fail(f"schema missing required fields: {sorted(missing_required)}")
    props = schema.get("properties") or {}
    for field in ["advisory", "stale", "scope"]:
        if field not in props:
            fail(f"schema missing property: {field}")
    status_enum = set(props.get("status", {}).get("enum") or [])
    missing_status = REQUIRED_STATUS - status_enum
    if missing_status:
        fail(f"schema missing status values: {sorted(missing_status)}")
    scope_status_enum = set(props.get("scope", {}).get("properties", {}).get("scope_status", {}).get("enum") or [])
    missing_scope = REQUIRED_SCOPE_STATUS - scope_status_enum
    if missing_scope:
        fail(f"schema missing scope_status values: {sorted(missing_scope)}")

    stubs = STUBS.read_text()
    for term in REQUIRED_STUB_TERMS:
        if term not in stubs:
            fail(f"stubs doc missing section: {term}")
    for term in ["canonical=false", "degraded=true", "advisory=true", "scope.scope_status", "side_effects", "evidence_refs"]:
        if term not in stubs:
            fail(f"stubs doc missing shared semantics term: {term}")

    tools = TOOLS.read_text()
    for term in ["focusaToolResult", "canonical", "degraded", "side_effects", "evidence_refs", "next_tools"]:
        if term not in tools:
            fail(f"Pi tools missing envelope term: {term}")

    menubar = MENUBAR_API.read_text()
    for term in ["canonical", "degraded", "side_effects", "evidence_refs"]:
        if term not in menubar:
            fail(f"Menubar API missing envelope term: {term}")

    uiai = UIAI_WORKSHEET.read_text()
    for term in ["capture_status", "scope_source", "proposal_only", "headless_next_action"]:
        if term not in uiai:
            fail(f"UIAI worksheet missing bridge term: {term}")

    if "tests/spec98_shared_tool_result_envelope_static_test.py" not in SUITE.read_text():
        fail("Spec98 regression suite does not run shared envelope guard")

    print("✓ PASS: Spec98 shared tool result envelope schema/stubs ok")


if __name__ == "__main__":
    main()
