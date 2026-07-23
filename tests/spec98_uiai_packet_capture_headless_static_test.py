#!/usr/bin/env python3
"""Spec98 / focusa-877z.15 UIAI packet capture + headless parity contract guard."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKSHEET = ROOT / "docs/worksheets/focusa-877z.15-uiai-packet-capture-headless.yaml"
IMPACT = ROOT / "docs/worksheets/focusa-877z.14-pi-uiai-impact.yaml"
SPEC98 = ROOT / "docs/98-project-root-crdt-reconciliation-foundation-spec.md"
AUDIT = ROOT / "docs/99-original-intent-vs-implementation-audit.md"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"

REQUIRED_CAPTURE_STATUSES = {
    "proposal_only",
    "capture_pending",
    "captured",
    "linked",
    "rejected",
    "degraded",
}
REQUIRED_SCOPE_SOURCES = {
    "focusa_verified_scope",
    "caller_supplied_scope",
    "uiai_local_default_scope",
    "missing_scope",
    "mismatch_candidate",
}
REQUIRED_COLUMNS = {
    "schema",
    "mode",
    "scope_status",
    "scope_source",
    "focusa_scope.project_root",
    "focusa_scope.continuity_id",
    "focusa_scope.workpoint_id",
    "evidence_refs",
    "recommended_focusa.preferred_tool",
    "recommended_focusa.args_preview",
    "capture_status",
    "headless_next_action",
    "render.summary_line",
    "cleanup.session_closed",
    "proof_commands",
}
REQUIRED_PARITY_SURFACES = {"pi_tui", "pi_rpc_json", "mcp", "http", "cli"}
REQUIRED_RENDER_TERMS = {"scope=", "scope_source=", "capture=", "tool=", "next="}
REQUIRED_HANDOFF_TERMS = {
    "capture_status",
    "scope_source",
    "summary_line",
    "proposal_only",
    "HTTP/MCP/CLI/Pi parity",
}


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    if not WORKSHEET.exists():
        fail(f"worksheet missing: {WORKSHEET}")
    data = yaml.safe_load(WORKSHEET.read_text())
    if data.get("schema_version") != "focusa.uiai_packet_capture_headless_semantics.v1":
        fail("unexpected worksheet schema_version")
    if data.get("status") != "implementation_ready":
        fail("worksheet status must be implementation_ready")

    capture_statuses = set((data.get("packet_status_semantics") or {}).keys())
    missing_capture = REQUIRED_CAPTURE_STATUSES - capture_statuses
    if missing_capture:
        fail(f"missing capture statuses: {sorted(missing_capture)}")
    for status, block in (data.get("packet_status_semantics") or {}).items():
        if not block.get("render_text") or "capture=" not in block.get(
            "render_text", ""
        ):
            fail(f"capture status {status} lacks capture= render text")
        if "focusa_authority" not in block:
            fail(f"capture status {status} lacks focusa_authority")

    scope_sources = set((data.get("scope_source_semantics") or {}).keys())
    missing_sources = REQUIRED_SCOPE_SOURCES - scope_sources
    if missing_sources:
        fail(f"missing scope sources: {sorted(missing_sources)}")
    for source, block in (data.get("scope_source_semantics") or {}).items():
        if not block.get("render_text") or "scope_source=" not in block.get(
            "render_text", ""
        ):
            fail(f"scope source {source} lacks scope_source= render text")
        if not block.get("capture_policy"):
            fail(f"scope source {source} lacks capture_policy")

    columns = set(data.get("required_packet_columns") or [])
    missing_columns = REQUIRED_COLUMNS - columns
    if missing_columns:
        fail(f"missing packet columns: {sorted(missing_columns)}")

    render = data.get("compact_render_contract") or {}
    required_terms = set(render.get("required_terms") or [])
    missing_terms = REQUIRED_RENDER_TERMS - required_terms
    if missing_terms:
        fail(f"missing compact render terms: {sorted(missing_terms)}")
    first_line = render.get("first_line_template", "")
    for term in REQUIRED_RENDER_TERMS:
        if term not in first_line:
            fail(f"first line template missing {term}")

    parity_surfaces = set(
        (data.get("headless_parity_contract") or {}).get("surfaces", {}).keys()
    )
    missing_surfaces = REQUIRED_PARITY_SURFACES - parity_surfaces
    if missing_surfaces:
        fail(f"missing headless parity surfaces: {sorted(missing_surfaces)}")

    proof_text = yaml.safe_dump(data.get("proof_matrix") or {})
    for expected in [
        "tests/spec98_uiai_packet_capture_headless_static_test.py",
        "npm --prefix apps/pi-extension run check",
        "scripts/check-focusa-packet-drift.sh",
        "bun test ./.pi/extensions/uiai-engine.packet-builder.test.ts",
    ]:
        if expected not in proof_text:
            fail(f"proof matrix missing {expected}")

    handoff = "\n".join(data.get("handoff_requirements_for_uiai_repo") or [])
    for term in REQUIRED_HANDOFF_TERMS:
        if term not in handoff:
            fail(f"handoff requirements missing {term}")

    worksheet_text = WORKSHEET.read_text()
    for phrase in [
        "scope_status=present",
        "not Focusa scope verification",
        "preview-only",
        "UIAI local demo defaults",
        "Focusa verification required before canonical capture",
    ]:
        if phrase not in worksheet_text:
            fail(f"worksheet missing required phrase: {phrase}")

    for path, phrases in {
        IMPACT: [
            "proposal_only_until_focusa_capture_or_link_succeeds",
            "headless fallback must be explicit",
        ],
        SPEC98: [
            "Packet renderers should display",
            "proposal_only",
            "not_canonical_until_captured",
        ],
        AUDIT: [
            "packet_status_semantics",
            "proposal-only until Focusa capture/link succeeds",
        ],
    }.items():
        text = path.read_text()
        for phrase in phrases:
            if phrase not in text:
                fail(f"{path.name} missing supporting phrase: {phrase}")

    if (
        "tests/spec98_uiai_packet_capture_headless_static_test.py"
        not in SUITE.read_text()
    ):
        fail("Spec98 regression suite does not run UIAI packet capture/headless guard")

    print("✓ PASS: Spec98 UIAI packet capture/headless parity contract ok")


if __name__ == "__main__":
    main()
