#!/usr/bin/env python3
"""Spec 100 Phase 1 — ContextCognitionPacket schema static test.

Verifies that:
- The Rust types in focusa-core/src/types.rs define the
  ContextCognitionPacket envelope with the Spec 100 §6 fields.
- The daemon route /v1/context-cognition is wired.
- The CLI subcommand `focusa context-cognition view` exists.
- The Pi extension tool `focusa_context_cognition` is registered.
- The tool contract is present in the JSON registry.
- The choreography edge from `focusa_context_cognition` to its
  next-tools is present.
- The tool doc page exists and references Spec 100.
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
    types_path = ROOT / "crates/focusa-core/src/types.rs"
    if not types_path.exists():
        fail(f"missing: {types_path}")
    types_src = types_path.read_text()

    required_structs = [
        "ContextCognitionPacket",
        "ContextCognitionFreshness",
        "ContextCognitionScope",
        "ContextCognitionAuthority",
        "ContextCognitionSelectedContext",
        "ContextCognitionOntologyFrame",
        "ContextCognitionEvidenceFrame",
        "ContextCognitionReasoningFrame",
        "ContextCognitionOptimizationFrame",
        "ContextCognitionRouteFrame",
        "ContextCognitionRecommendedPacketUse",
    ]
    for name in required_structs:
        if f"pub struct {name}" not in types_src:
            fail(f"types.rs missing struct: {name}")

    # The packet must contain the Spec 100 §6 required fields.
    packet_struct = re.search(
        r"pub struct ContextCognitionPacket\s*\{([\s\S]*?)\n\}", types_src
    )
    if not packet_struct:
        fail("ContextCognitionPacket struct not found")
    body = packet_struct.group(1)
    required_fields = [
        "schema_version",
        "status",
        "advisory",
        "canonical",
        "scope_status",
        "freshness",
        "scope",
        "authority",
        "selected_context",
        "ontology_frame",
        "evidence_frame",
        "reasoning_frame",
        "optimization_frame",
        "route_frame",
        "side_effects",
        "evidence_refs",
        "recommended_packet_use",
    ]
    for field in required_fields:
        if f"pub {field}" not in body:
            fail(f"ContextCognitionPacket missing field: {field}")

    # Daemon route wired.
    server_path = ROOT / "crates/focusa-api/src/server.rs"
    if not server_path.exists():
        fail("server.rs missing")
    if "routes::context_cognition::router()" not in server_path.read_text():
        fail("server.rs missing context_cognition::router() merge")

    route_path = ROOT / "crates/focusa-api/src/routes/context_cognition.rs"
    if not route_path.exists():
        fail("context_cognition.rs route missing")
    route_src = route_path.read_text()
    if '"/v1/context-cognition"' not in route_src:
        fail("context_cognition.rs route not /v1/context-cognition")

    # CLI subcommand wired.
    cli_main = (ROOT / "crates/focusa-cli/src/main.rs").read_text()
    if (
        "ContextCognition(commands::context_cognition::ContextCognitionCmd)"
        not in cli_main
    ):
        fail("CLI main.rs missing ContextCognition subcommand")
    cli_cmd = (ROOT / "crates/focusa-cli/src/commands/context_cognition.rs").read_text()
    if "pub enum ContextCognitionCmd" not in cli_cmd:
        fail("context_cognition.rs missing enum")
    if "View" not in cli_cmd:
        fail("context_cognition.rs missing View subcommand")

    # Pi extension tool + contract + choreo + doc.
    tools_src = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
    if 'name: "focusa_context_cognition"' not in tools_src:
        fail("Pi extension missing focusa_context_cognition tool")

    contracts_src = (ROOT / "apps/pi-extension/src/tool-contracts.ts").read_text()
    if '"focusa_context_cognition"' not in contracts_src:
        fail("tool-contracts.ts missing focusa_context_cognition contract")
    if "focusa_context_cognition" not in re.search(
        r"const TOOL_NEXT_TOOLS: Record<string, string\[\]> = ([\s\S]*?)\n\};",
        contracts_src,
    ).group(1):
        fail("TOOL_NEXT_TOOLS missing focusa_context_cognition")

    registry = json.loads(
        (ROOT / "docs/current/focusa-tool-contracts.json").read_text()
    )
    if not any(
        c.get("name") == "focusa_context_cognition"
        for c in registry.get("contracts", [])
    ):
        fail("focusa-tool-contracts.json missing focusa_context_cognition")
    if registry.get("tool_count", 0) < 66:
        fail(f"tool_count expected >= 66, got {registry.get('tool_count')}")

    choreo = json.loads(
        (ROOT / "docs/current/focusa-tool-choreography.json").read_text()
    )
    if choreo.get("tool_count", 0) < 66:
        fail(f"choreography tool_count expected >= 66, got {choreo.get('tool_count')}")
    if not any(
        e.get("from") == "focusa_context_cognition" for e in choreo.get("edges", [])
    ):
        fail("choreography missing edge from focusa_context_cognition")

    doc_path = ROOT / "docs/focusa-tools/tools/focusa_context_cognition.md"
    if not doc_path.exists():
        fail(f"missing tool doc: {doc_path}")
    doc_src = doc_path.read_text()
    for marker in ["Purpose", "Expected result", "failure_class"]:
        if marker not in doc_src:
            fail(f"doc missing marker: {marker}")
    if "Spec 100" not in doc_src:
        fail("doc missing Spec 100 reference")

    print(
        f"✓ PASS: focusa_context_cognition schema + route + CLI + Pi + contract + doc all wired (tool_count={registry.get('tool_count')})"
    )


if __name__ == "__main__":
    main()
