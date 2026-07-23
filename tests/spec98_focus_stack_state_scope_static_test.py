#!/usr/bin/env python3
"""Spec98/99 Phase C: Focus Stack/Focus State writes are scoped by ProjectRootKey + WorkstreamKey."""

from pathlib import Path
import re
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.22-focus-stack-state-scope.yaml"
FOCUS = ROOT / "crates/focusa-api/src/routes/focus.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def function_body(text: str, name: str) -> str:
    marker = f"async fn {name}"
    start = text.find(marker)
    if start < 0:
        marker = f"fn {name}"
        start = text.find(marker)
    if start < 0:
        fail(f"function missing: {name}")
    brace = text.find("{", start)
    depth = 0
    for i in range(brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[brace + 1 : i]
    fail(f"function body unterminated: {name}")
    return ""


def struct_body(text: str, name: str) -> str:
    marker = f"struct {name}"
    start = text.find(marker)
    if start < 0:
        fail(f"struct missing: {name}")
    brace = text.find("{", start)
    end = text.find("}\n", brace)
    if end < 0:
        fail(f"struct body missing: {name}")
    return text[brace + 1 : end]


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.focus_stack_state_scope_contract.v1":
        fail("unexpected contract schema_version")
    if data.get("status") != "focus_frame_writes_require_project_workstream_scope":
        fail(
            "contract status is not focus_frame_writes_require_project_workstream_scope"
        )
    text = FOCUS.read_text()
    if "fn clean_scope_value" not in text:
        fail("focus.rs must define clean_scope_value for scope validation")

    push_body = function_body(text, "push_frame")
    if "clean_scope_value(body.project_root.as_deref())" not in push_body:
        fail("push_frame must require project_root")
    if "clean_scope_value(body.continuity_id.as_deref())" not in push_body:
        fail("push_frame must require continuity_id")
    if "project_root: Some(project_root)" not in push_body:
        fail("FocusFramePushed must carry scoped project_root")
    if "continuity_id: Some(continuity_id)" not in push_body:
        fail("FocusFramePushed must carry scoped continuity_id")

    update_struct = struct_body(text, "UpdateDeltaBody")
    if "project_root: Option<String>" not in update_struct:
        fail("UpdateDeltaBody must accept project_root")
    if "continuity_id: Option<String>" not in update_struct:
        fail("UpdateDeltaBody must accept continuity_id")

    update_body = function_body(text, "update_delta")
    if "Never adopt the daemon-global active frame" not in update_body:
        fail(
            "update_delta must document daemon-global active frame is not write authority"
        )
    if (
        "focus_update_requires_frame_id_or_project_root_plus_continuity_id"
        not in update_body
    ):
        fail("update_delta must reject unscoped writes")
    if "clean_scope_value(frame.continuity_id.as_deref()).is_none()" not in update_body:
        fail("update_delta must reject frame_id targets without continuity_id")
    if not re.search(
        r"frame\s*\.project_root\s*\.as_deref\(\)\s*\.map\(normalize_project_root_authority\)",
        update_body,
    ):
        fail("update_delta must compare provided project_root to target frame")
    if not re.search(
        r"frame\s*\.continuity_id\s*\.as_deref\(\)\s*\.map\(str::trim\)", update_body
    ):
        fail("update_delta must compare provided continuity_id to target frame")
    if "(active_id, !session_active)" in update_body:
        fail("update_delta must not write to daemon-global active_id fallback")

    proofs = set(data.get("proof_requirements") or [])
    for proof in [
        "static Focus push requires project_root and continuity_id",
        "static Focus update forbids daemon-global active frame adoption",
    ]:
        if proof not in proofs:
            fail(f"contract missing proof requirement: {proof}")
    print("✓ PASS: Focus Stack/Focus State write scope guard is present")


if __name__ == "__main__":
    main()
