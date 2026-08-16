#!/usr/bin/env python3
"""Spec98 focusa-877z.13: bounded orchestration is separate from cognition authority."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.13-bounded-orchestration-contract.yaml"
TYPES = ROOT / "crates/focusa-core/src/types.rs"
WORK_LOOP = ROOT / "crates/focusa-api/src/routes/work_loop.rs"
CAP_EXTRA = ROOT / "crates/focusa-api/src/routes/capabilities_extra.rs"
TOOLS = ROOT / "apps/pi-extension/src/tools.ts"
SILENT_DOC = ROOT / "docs/focusa-tools/tools/focusa_silent_sessions.md"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def fn_body(source: str, name: str) -> str:
    for marker in [f"fn {name}", f"async fn {name}", f"pub fn {name}"]:
        start = source.find(marker)
        if start != -1:
            break
    else:
        fail(f"missing function {name}")
    brace = source.find("{", start)
    depth = 0
    for i in range(brace, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start : i + 1]
    fail(f"unterminated function {name}")


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.bounded_orchestration_contract.v1":
        fail("unexpected .13 contract schema")
    rule = data.get("normative_rule", "")
    for phrase in [
        "bounded orchestration surfaces",
        "writer ownership",
        "operator controls",
        "do not become Focus State authority",
    ]:
        if phrase not in rule:
            fail(f"contract normative rule missing {phrase}")

    types = TYPES.read_text()
    for needle in [
        "pub enum OrchestrationSurface",
        "WorkLoop",
        "AutonomyCalibration",
        "SilentSession",
        "pub const BOUNDED_ORCHESTRATION_CONTRACT",
    ]:
        if needle not in types:
            fail(f"types.rs missing orchestration contract needle {needle}")
    plane_text = types[
        types.find("pub const FOCUSA_STATE_PLANE_CONTRACT") : types.find(
            "/// The complete cognitive state"
        )
    ]
    for mapping in [
        '("work_loop", AuthorityPlane::BoundedOrchestration)',
        '("autonomy", AuthorityPlane::BoundedOrchestration)',
    ]:
        if mapping not in plane_text:
            fail(f"state plane contract missing {mapping}")
    for mapping in [
        '("work_loop", OrchestrationSurface::WorkLoop)',
        '("autonomy", OrchestrationSurface::AutonomyCalibration)',
        '("silent_session", OrchestrationSurface::SilentSession)',
    ]:
        if mapping not in types:
            fail(f"bounded orchestration contract missing {mapping}")

    work_loop = WORK_LOOP.read_text()
    helper = fn_body(work_loop, "bounded_orchestration_authority_payload")
    for needle in [
        '"authority_plane": "bounded_orchestration"',
        '"canonical": false',
        '"focus_state_authority": false',
        '"writer_ownership_required": true',
        '"operator_controls"',
        '"pause"',
        '"resume"',
        '"stop"',
        '"preflight"',
    ]:
        if needle not in helper:
            fail(f"work-loop authority helper missing {needle}")
    for needle in [
        "ensure_writer_claim",
        "release_writer_claim",
        "ensure_claimed_writer_matches_for_context",
        "x-focusa-writer-id",
        "x-focusa-approval",
    ]:
        if needle not in work_loop:
            fail(f"work-loop writer/operator guard missing {needle}")
    if work_loop.count('"authority": bounded_orchestration_authority_payload()') < 3:
        fail(
            "work-loop health/summary/deep status payloads must expose authority metadata"
        )

    cap_extra = CAP_EXTRA.read_text()
    for name in ["autonomy_status", "autonomy_ledger", "autonomy_explain"]:
        body = fn_body(cap_extra, name)
        for needle in [
            '"authority_plane": "bounded_orchestration"',
            '"canonical": false',
            '"focus_state_authority": false',
        ]:
            if needle not in body:
                fail(f"{name} missing bounded orchestration marker {needle}")
        if "state.focusa.write().await" in body:
            fail(f"{name} must not mutate Focus State authority")

    tools = TOOLS.read_text()
    doc = SILENT_DOC.read_text()
    for needle in [
        "focusa_silent_sessions",
        "approved !== true",
        "force !== true",
        "tmux_new_session",
        "tmux_send_interrupt",
        "tmux_send_keys_literal",
        "tmux_kill_session",
    ]:
        if needle not in tools:
            fail(f"SilentSession tool missing guard/process marker {needle}")
    for needle in [
        "attach_to_workpoint: false",
        "SilentSession start",
        "focusa_resource_mode",
    ]:
        if needle not in tools:
            fail(f"SilentSession tool missing proof/resource posture {needle}")
    for needle in [
        "requires `approved=true`",
        "requires `approved=true` and `force=true`",
        "tmux",
        "as-user <owner>",
        "process-control actions",
    ]:
        if needle not in doc:
            fail(f"SilentSession doc missing guardrail {needle}")

    print(
        "✓ PASS: work-loop, autonomy, and SilentSession are bounded orchestration, not Focus State authority"
    )


if __name__ == "__main__":
    main()
