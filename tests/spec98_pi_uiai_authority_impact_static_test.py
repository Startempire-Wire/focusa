#!/usr/bin/env python3
"""Spec98 / focusa-877z.14 Pi + UIAI authority impact worksheet coverage test."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKSHEET = ROOT / "docs/worksheets/focusa-877z.14-pi-uiai-impact.yaml"
SPEC98 = ROOT / "docs/98-project-root-crdt-reconciliation-foundation-spec.md"
UIAI_SPEC = ROOT / "docs/current/UIAI_BROWSER_DIAGNOSTICS_FOCUSA_INTEGRATION_SPEC.md"
CONTRACTS = ROOT / "apps/pi-extension/src/tool-contracts.ts"
TOOLS = ROOT / "apps/pi-extension/src/tools.ts"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"

REQUIRED_TOP_LEVEL = {
    "impacted_focusa_tools",
    "impacted_uiai_tools",
    "packet_schemas",
    "diagnostics_and_evidence_flows",
    "tool_contract_registry_entries",
    "render_and_compaction_behavior",
    "reliability_proof_matrix",
    "risk_register",
}
REQUIRED_FOCUSA_TOOLS = {
    "focusa_project_identity",
    "focusa_project_verify",
    "focusa_workpoint_checkpoint",
    "focusa_workpoint_resume",
    "focusa_workpoint_link_evidence",
    "focusa_evidence_capture",
    "focusa_browser_diagnostics_intake",
    "focusa_trajectory_view",
    "focusa_active_object_resolve",
    "focusa_tool_doctor",
    "focusa_resource_mode",
    "focusa_predict_record",
    "focusa_predict_evaluate",
    "focusa_metacog_capture",
}
REQUIRED_UIAI_TOOLS = {
    "pi_uiai_agent_card",
    "pi_uiai_tool_search",
    "pi_uiai_tool_graph",
    "uiai_health",
    "uiai_browser_open",
    "uiai_browser_screenshot",
    "uiai_browser_snapshot",
    "uiai_browser_diagnostics",
    "uiai_browser_diagnostics_clear",
    "uiai_screenshot",
}
REQUIRED_PACKET_SCHEMAS = {
    "focusa_tool_result_v1",
    "focusa_session_identity",
    "uiai_focusa_research_diagnostics_packet_v1",
    "focusa_scope",
    "visual_evidence_handle",
}
REQUIRED_FLOWS = {"browser_failure", "visual_workflow", "headless_packet"}
REQUIRED_PHRASES = [
    "proposal_only_until_focusa_capture_or_link_succeeds",
    "metadata_only; validate against active Focusa project/workstream",
    "First packet line says whether UIAI packet is captured",
    "headless fallback must be explicit",
]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def flatten_tools(section: dict) -> set[str]:
    tools: set[str] = set()
    for entry in section.values():
        for name in entry.get("tools", []) or []:
            tools.add(str(name))
    return tools


def main() -> None:
    if not WORKSHEET.exists():
        fail(f"worksheet missing: {WORKSHEET}")
    data = yaml.safe_load(WORKSHEET.read_text())
    if data.get("status") != "implementation_ready":
        fail("worksheet status is not implementation_ready")
    missing_top = REQUIRED_TOP_LEVEL - set(data)
    if missing_top:
        fail(f"worksheet missing top-level sections: {sorted(missing_top)}")

    focusa_tools = flatten_tools(data["impacted_focusa_tools"])
    missing_focusa = REQUIRED_FOCUSA_TOOLS - focusa_tools
    if missing_focusa:
        fail(f"worksheet missing Focusa tools: {sorted(missing_focusa)}")

    uiai_tools = flatten_tools(data["impacted_uiai_tools"])
    missing_uiai = REQUIRED_UIAI_TOOLS - uiai_tools
    if missing_uiai:
        fail(f"worksheet missing UIAI tools: {sorted(missing_uiai)}")

    packet_schemas = set(data["packet_schemas"].keys())
    missing_packets = REQUIRED_PACKET_SCHEMAS - packet_schemas
    if missing_packets:
        fail(f"worksheet missing packet schemas: {sorted(missing_packets)}")

    flows = set(data["diagnostics_and_evidence_flows"].keys())
    missing_flows = REQUIRED_FLOWS - flows
    if missing_flows:
        fail(f"worksheet missing diagnostics/evidence flows: {sorted(missing_flows)}")

    registry_entries = "\n".join(
        data["tool_contract_registry_entries"].get("required_entries", [])
    )
    for expected in [
        "apps/pi-extension/src/tool-contracts.ts",
        "docs/current/focusa-tool-contracts.json",
        "focusa_browser_diagnostics_intake",
        "focusa_tool_doctor",
    ]:
        if expected not in registry_entries:
            fail(f"registry entries missing {expected}")

    proof_text = yaml.safe_dump(data.get("reliability_proof_matrix", {}))
    for expected in [
        "tests/spec98_runtime_bleed_crdt_regression_suite.sh",
        "npm --prefix apps/pi-extension run check",
        "UIAI scripts/check-tool-parity.sh",
        "UIAI browser diagnostics smoke",
    ]:
        if expected not in proof_text:
            fail(f"reliability proof matrix missing {expected}")

    worksheet_text = WORKSHEET.read_text()
    for phrase in REQUIRED_PHRASES:
        if phrase not in worksheet_text:
            fail(f"worksheet missing required phrase: {phrase}")

    spec98_text = SPEC98.read_text()
    for phrase in [
        "Pi agent tools and UIAI browser integration gate",
        "ResearchDiagnosticsPacket",
        "focusa_scope",
    ]:
        if phrase not in spec98_text:
            fail(f"Spec98 missing expected section phrase: {phrase}")

    uiai_text = UIAI_SPEC.read_text()
    for phrase in ["focusa_browser_diagnostics_intake", "focusa_scope", "uiai_browser"]:
        if phrase not in uiai_text:
            fail(f"UIAI integration spec missing expected phrase: {phrase}")

    pi_surface_text = CONTRACTS.read_text() + "\n" + TOOLS.read_text()
    for phrase in ["focusa_browser_diagnostics_intake", "uiai_browser", "focusa_scope"]:
        if phrase not in pi_surface_text:
            fail(f"Pi tool contracts/tools missing expected phrase: {phrase}")

    if "tests/spec98_pi_uiai_authority_impact_static_test.py" not in SUITE.read_text():
        fail("Spec98 regression suite does not run Pi/UIAI authority impact test")

    print("✓ PASS: Spec98 Pi/UIAI authority impact worksheet coverage ok")


if __name__ == "__main__":
    main()
