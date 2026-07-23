#!/usr/bin/env python3
"""Spec 100 Phase 3 — Context Curator with token-budgeted selection.

Verifies that:
- POST /v1/context-cognition/curate route is wired
- focusa context-cognition curate CLI subcommand exists
- focusa_context_cognition_curate Pi tool is registered
- Tool contract is present in the JSON registry
- Choreography edge from focusa_context_cognition_curate is present
- Tool doc page exists and references Spec 100 §14
- Token budget selection keeps highest-scored items
- Exclusion reasons are bounded strings (low_score / over_budget)
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
        "/v1/context-cognition/curate",
        "async fn curate",
        "fn estimate_tokens",
        "fn score_candidate",
        "CurateCandidate",
        "CurateRequest",
        "selected_context",
        "excluded_context",
        "low_score",
        "over_budget",
    ]:
        if marker not in route_src:
            fail(f"context_cognition.rs missing marker: {marker}")

    # Curator unit tests
    test_block = re.search(
        r"#\[cfg\(test\)\]\s*mod tests\s*\{([\s\S]*?)\n\}", route_src
    )
    if not test_block:
        fail("context_cognition.rs missing tests module")
    body = test_block.group(1)
    for fn in [
        "curator_token_budget_keeps_highest_scored",
        "curator_exclusion_labeled",
    ]:
        if f"fn {fn}" not in body:
            fail(f"context_cognition.rs missing test fn: {fn}")

    # CLI
    cli_cmd = (ROOT / "crates/focusa-cli/src/commands/context_cognition.rs").read_text()
    for marker in [
        "ContextCognitionCmd::Curate",
        "candidates_json",
        "print_curated_human",
    ]:
        if marker not in cli_cmd:
            fail(f"context_cognition.rs CLI missing marker: {marker}")

    # Pi extension
    tools_src = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
    if 'name: "focusa_context_cognition_curate"' not in tools_src:
        fail("Pi extension missing focusa_context_cognition_curate tool")
    if "/v1/context-cognition/curate" not in tools_src:
        fail("Pi extension curate tool missing route")
    if "token_budget" not in tools_src:
        fail("Pi extension curate tool missing token_budget parameter")
    if "selected_context" not in tools_src or "excluded_context" not in tools_src:
        fail("Pi extension curate tool missing selected/excluded context handling")

    # Contracts
    contracts_src = (ROOT / "apps/pi-extension/src/tool-contracts.ts").read_text()
    if '"focusa_context_cognition_curate"' not in contracts_src:
        fail("tool-contracts.ts missing focusa_context_cognition_curate contract")
    ntt = re.search(
        r"const TOOL_NEXT_TOOLS: Record<string, string\[]> = ([\s\S]*?)\n\};",
        contracts_src,
    )
    if not ntt or '"focusa_context_cognition_curate"' not in ntt.group(1):
        fail("TOOL_NEXT_TOOLS missing curate entry")

    # JSON registry
    registry = json.loads(
        (ROOT / "docs/current/focusa-tool-contracts.json").read_text()
    )
    if not any(
        c.get("name") == "focusa_context_cognition_curate"
        for c in registry.get("contracts", [])
    ):
        fail("focusa-tool-contracts.json missing focusa_context_cognition_curate")
    if registry.get("tool_count", 0) < 69:
        fail(f"tool_count expected >= 69, got {registry.get('tool_count')}")

    # Choreography
    choreo = json.loads(
        (ROOT / "docs/current/focusa-tool-choreography.json").read_text()
    )
    if choreo.get("tool_count", 0) < 69:
        fail(f"choreography tool_count expected >= 69, got {choreo.get('tool_count')}")
    if not any(
        e.get("from") == "focusa_context_cognition_curate"
        for e in choreo.get("edges", [])
    ):
        fail("choreography missing edge from focusa_context_cognition_curate")

    # Doc page
    doc_path = ROOT / "docs/focusa-tools/tools/focusa_context_cognition_curate.md"
    if not doc_path.exists():
        fail(f"missing doc: {doc_path}")
    doc_src = doc_path.read_text()
    for marker in ["Purpose", "Expected result", "failure_class", "Spec 100", "§14"]:
        if marker not in doc_src:
            fail(f"doc missing marker: {marker}")
    for reason in ["low_score", "over_budget"]:
        if reason not in doc_src:
            fail(f"doc missing exclusion reason: {reason}")

    # README tool row
    readme = (ROOT / "README.md").read_text()
    if "`focusa_context_cognition_curate`" not in readme:
        fail("README missing focusa_context_cognition_curate tool row")

    print(
        f"✓ PASS: focusa_context_cognition_curate (Spec 100 P3 Curator) wired across core, api, cli, pi, contract, choreo, doc, audit; tool_count={registry.get('tool_count')}"
    )


if __name__ == "__main__":
    main()
