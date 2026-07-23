#!/usr/bin/env python3
"""Spec98/99 Phase C: Workpoint/Trajectory active selectors fail closed without project/workstream scope."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = (
    ROOT / "docs/worksheets/focusa-877z.21-workpoint-trajectory-active-scope.yaml"
)
WORKPOINT = ROOT / "crates/focusa-api/src/routes/workpoint.rs"
TRAJECTORY = ROOT / "crates/focusa-api/src/routes/trajectory.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def function_body(text: str, name: str) -> str:
    marker = f"fn {name}"
    start = text.find(marker)
    if start < 0:
        fail(f"function missing: {name}")
    brace = text.find("{", start)
    if brace < 0:
        fail(f"function body missing: {name}")
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


def main() -> None:
    if not CONTRACT.exists():
        fail(f"contract missing: {CONTRACT}")
    data = yaml.safe_load(CONTRACT.read_text())
    if (
        data.get("schema_version")
        != "focusa.workpoint_trajectory_active_scope_contract.v1"
    ):
        fail("unexpected contract schema_version")
    if data.get("status") != "primary_selectors_fail_closed_on_missing_scope":
        fail("contract status is not primary_selectors_fail_closed_on_missing_scope")

    wp = WORKPOINT.read_text()
    wp_body = function_body(wp, "active_workpoint_for_scope")
    if "clean_resume_scope_value(project_root)?" not in wp_body:
        fail("active_workpoint_for_scope must require project_root via ?")
    if "clean_resume_scope_value(continuity_id)?" not in wp_body:
        fail("active_workpoint_for_scope must require continuity_id via ?")
    if "active_workpoint(state)" in wp_body:
        fail(
            "active_workpoint_for_scope must not fall back to daemon-global active_workpoint(state)"
        )
    if (
        "record.project_root.as_deref().map(str::trim) == Some(clean_project.as_str())"
        not in wp_body
    ):
        fail("active_workpoint_for_scope must exact-match project_root")
    if (
        "record.continuity_id.as_deref().map(str::trim) == Some(clean_continuity.as_str())"
        not in wp_body
    ):
        fail("active_workpoint_for_scope must exact-match continuity_id")
    forbidden_scoped_fallback = """if expected_project_root.is_some() || expected_continuity_id.is_some() {
            active_workpoint_for_scope(
                &focusa,
                expected_project_root.as_deref(),
                expected_continuity_id.as_deref(),
            )
            .or_else(|| active_workpoint(&focusa))"""
    if forbidden_scoped_fallback in wp:
        fail(
            "scoped Workpoint resolution must not fall back to daemon-global active_workpoint"
        )

    tr = TRAJECTORY.read_text()
    tr_body = function_body(tr, "active_persisted_trajectory")
    if "let expected_project_root = clean(project_root)?;" not in tr_body:
        fail("active_persisted_trajectory must require project_root via ?")
    if "let expected_continuity_id = clean(continuity_id)?;" not in tr_body:
        fail("active_persisted_trajectory must require continuity_id via ?")
    if "unwrap_or(true)" in tr_body:
        fail(
            "active_persisted_trajectory must not use unwrap_or(true) wildcard scope matching"
        )
    if (
        "record.project_root.as_deref() == Some(expected_project_root.as_str())"
        not in tr_body
    ):
        fail("active_persisted_trajectory must exact-match project_root")
    if (
        "record.continuity_id.as_deref() == Some(expected_continuity_id.as_str())"
        not in tr_body
    ):
        fail("active_persisted_trajectory must exact-match continuity_id")

    proofs = set(data.get("proof_requirements") or [])
    for proof in [
        "static selector fail-closed test",
        "two-project active Workpoint isolation test",
        "trajectory prior-project fallback advisory provenance test",
    ]:
        if proof not in proofs:
            fail(f"contract missing proof requirement: {proof}")
    print(
        "✓ PASS: Workpoint/Trajectory active selectors fail closed without project/workstream scope"
    )


if __name__ == "__main__":
    main()
