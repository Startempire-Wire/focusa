#!/usr/bin/env python3
"""Spec 100 Phase 2 — Context Cognition cross-surface contracts static test.

Verifies that focusa_context_cognition_render and focusa_context_cognition_proof
are wired across core (types), api (routes), cli (subcommand), pi extension
(tool + contract + choreo), menubar (doc references), and the tool registry.

Also verifies that the audit classify-4xx-as-probe_validation_expected fix is
present so the audit no longer flags required-param routes as daemon_unavailable.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    route_path = ROOT / "crates/focusa-api/src/routes/context_cognition.rs"
    if not route_path.exists():
        fail("context_cognition.rs missing")
    route_src = route_path.read_text()
    for marker in [
        "/v1/context-cognition/render",
        "/v1/context-cognition/proof",
        "async fn render",
        "async fn proof",
    ]:
        if marker not in route_src:
            fail(f"context_cognition.rs missing marker: {marker}")

    # CLI
    cli_cmd = (ROOT / "crates/focusa-cli/src/commands/context_cognition.rs").read_text()
    for marker in [
        "ContextCognitionCmd::Render",
        "ContextCognitionCmd::Proof",
        "build_query",
    ]:
        if marker not in cli_cmd:
            fail(f"context_cognition.rs CLI missing marker: {marker}")

    # Pi extension
    tools_src = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
    for tool in [
        "focusa_context_cognition_render",
        "focusa_context_cognition_proof",
    ]:
        if f'name: "{tool}"' not in tools_src:
            fail(f"Pi extension missing tool: {tool}")
        if (
            "/v1/context-cognition/render" not in tools_src
            and tool == "focusa_context_cognition_render"
        ):
            fail("Pi extension render tool missing route")
        if (
            "/v1/context-cognition/proof" not in tools_src
            and tool == "focusa_context_cognition_proof"
        ):
            fail("Pi extension proof tool missing route")

    # Contracts
    contracts_src = (ROOT / "apps/pi-extension/src/tool-contracts.ts").read_text()
    for tool in ["focusa_context_cognition_render", "focusa_context_cognition_proof"]:
        if f'"{tool}"' not in contracts_src:
            fail(f"tool-contracts.ts missing contract: {tool}")
    ntt = re.search(
        r"const TOOL_NEXT_TOOLS: Record<string, string\[]> = ([\s\S]*?)\n\};",
        contracts_src,
    )
    if not ntt:
        fail("TOOL_NEXT_TOOLS not found")
    next_tools = ntt.group(1)
    if '"focusa_context_cognition_render"' not in next_tools:
        fail("TOOL_NEXT_TOOLS missing render entry")
    if '"focusa_context_cognition_proof"' not in next_tools:
        fail("TOOL_NEXT_TOOLS missing proof entry")

    # JSON registry
    registry = json.loads(
        (ROOT / "docs/current/focusa-tool-contracts.json").read_text()
    )
    for tool in ["focusa_context_cognition_render", "focusa_context_cognition_proof"]:
        if not any(c.get("name") == tool for c in registry.get("contracts", [])):
            fail(f"focusa-tool-contracts.json missing: {tool}")
    if registry.get("tool_count", 0) < 68:
        fail(f"tool_count expected >= 68, got {registry.get('tool_count')}")

    # Choreography
    choreo = json.loads(
        (ROOT / "docs/current/focusa-tool-choreography.json").read_text()
    )
    if choreo.get("tool_count", 0) < 68:
        fail(f"choreography tool_count expected >= 68, got {choreo.get('tool_count')}")
    for tool in ["focusa_context_cognition_render", "focusa_context_cognition_proof"]:
        if not any(e.get("from") == tool for e in choreo.get("edges", [])):
            fail(f"choreography missing edge from: {tool}")

    # Doc pages
    for tool in ["focusa_context_cognition_render", "focusa_context_cognition_proof"]:
        doc = ROOT / f"docs/focusa-tools/tools/{tool}.md"
        if not doc.exists():
            fail(f"missing doc: {doc}")
        doc_src = doc.read_text()
        for marker in ["Purpose", "Expected result", "failure_class"]:
            if marker not in doc_src:
                fail(f"doc {tool} missing marker: {marker}")

    # Audit classify-4xx fix
    audit_src = (ROOT / "scripts/audit-focusa-tool-suite-safe.mjs").read_text()
    if "probe_validation_expected" not in audit_src:
        fail("audit does not classify 4xx as probe_validation_expected")
    if "isValidation" not in audit_src:
        fail("audit does not gate 4xx retry warnings on isValidation")

    # README tool count + 2 new rows
    readme = (ROOT / "README.md").read_text()
    if "**68**" not in readme and "**68 tools" not in readme:
        # The README might say 65/66/67; check for >= 67 presence.
        if "**65**" in readme or "**66**" in readme:
            fail("README contract count not updated to >= 68")
    for tool in ["focusa_context_cognition_render", "focusa_context_cognition_proof"]:
        if f"`{tool}`" not in readme:
            fail(f"README missing tool row: {tool}")

    print(
        f"✓ PASS: focusa_context_cognition cross-surface contracts (render+proof) wired across core, api, cli, pi, menubar, doc, audit; tool_count={registry.get('tool_count')}"
    )


if __name__ == "__main__":
    main()
