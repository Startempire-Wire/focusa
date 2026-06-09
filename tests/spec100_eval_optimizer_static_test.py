#!/usr/bin/env python3
"""Spec 100 Phase 4 + Phase 5 — Curator eval harness + Cognition Optimizer.

Verifies CQRS shape for the eval-ledger + artifact-ledger pair:
- POST /v1/context-cognition/curate/eval (write side: eval ledger)
- GET /v1/context-cognition/curate/eval/runs (read side: eval ledger)
- POST /v1/context-cognition/curate/optimize (write side: artifact ledger)
- GET /v1/context-cognition/optimizer/artifacts (read side: artifact ledger)
- focusa_context_cognition_curate_eval + focusa_context_cognition_curate_optimize
  + focusa_context_cognition_optimizer_artifacts Pi tools
- focusa context-cognition curate-eval + curate-eval-runs + curate-optimize
  + optimizer artifacts CLI subcommands
- Tool contracts, choreo edges, doc pages
- CQRS read/write separation is explicit (GET vs POST)
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

    # Routes (CQRS read/write split)
    for marker in [
        "/v1/context-cognition/curate/eval",          # write side
        "post(curate_eval)",
        "/v1/context-cognition/curate/eval/runs",     # read side
        "get(curate_eval_runs)",
        "/v1/context-cognition/optimizer/artifacts",  # read side
        "get(optimizer_artifacts)",
        "/v1/context-cognition/curate/optimize",      # write side
        "post(curate_optimize)",
        # Types
        "CuratorEvalRun",
        "CognitionOptimizerArtifact",
        "CurateEvalRequest",
        "CurateOptimizeRequest",
        "OptimizerArtifactsRequest",
        # Score helpers
        "compute_precision_recall",
        "compute_f1",
        # Promotion rule
        "promoted",
        "decision",
        "rollback_ref",
    ]:
        if marker not in route_src:
            fail(f"context_cognition.rs missing marker: {marker}")

    # CQRS-shape: GET routes for reads, POST routes for writes
    if not re.search(r'"/v1/context-cognition/curate/eval/runs"\s*,\s*axum::routing::get\(curate_eval_runs\)', route_src):
        fail("curate/eval/runs must be GET (read side)")
    if not re.search(r'"/v1/context-cognition/optimizer/artifacts"\s*,\s*axum::routing::get\(optimizer_artifacts\)', route_src):
        fail("optimizer/artifacts must be GET (read side)")
    if not re.search(r'"/v1/context-cognition/curate/eval"\s*,\s*axum::routing::post\(curate_eval\)', route_src):
        fail("curate/eval must be POST (write side)")
    if not re.search(r'"/v1/context-cognition/curate/optimize"\s*,\s*axum::routing::post\(curate_optimize\)', route_src):
        fail("curate/optimize must be POST (write side)")

    # Persistence (HLT-pattern ledgers)
    types_src = (ROOT / "crates/focusa-core/src/types.rs").read_text()
    for marker in [
        "pub struct CuratorEvalCase",
        "pub struct CuratorEvalRun",
        "pub struct CognitionOptimizerArtifact",
        "pub struct CuratorEvalCandidate",
    ]:
        if marker not in types_src:
            fail(f"types.rs missing struct: {marker}")

    persistence_src = (ROOT / "crates/focusa-core/src/runtime/persistence_sqlite.rs").read_text()
    for marker in [
        "curator_eval_ledger_dir_for_project",
        "cognition_optimizer_artifacts_dir_for_project",
        "fn append_curator_eval_run",
        "fn read_curator_eval_runs",
        "fn append_cognition_optimizer_artifact",
        "fn read_cognition_optimizer_artifacts",
        "fn latest_promoted_artifact",
    ]:
        if marker not in persistence_src:
            fail(f"persistence_sqlite.rs missing: {marker}")

    # CLI
    cli_cmd = (ROOT / "crates/focusa-cli/src/commands/context_cognition.rs").read_text()
    for marker in [
        "ContextCognitionCmd::CurateEval",
        "ContextCognitionCmd::CurateEvalRuns",
        "ContextCognitionCmd::CurateOptimize",
        "ContextCognitionCmd::OptimizerArtifacts",
        "print_eval_human",
        "print_optimize_human",
    ]:
        if marker not in cli_cmd:
            fail(f"context_cognition.rs CLI missing marker: {marker}")

    # Pi extension
    tools_src = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
    for tool in [
        "focusa_context_cognition_curate_eval",
        "focusa_context_cognition_curate_optimize",
        "focusa_context_cognition_optimizer_artifacts",
    ]:
        if f'name: "{tool}"' not in tools_src:
            fail(f"Pi extension missing tool: {tool}")
    if "/v1/context-cognition/curate/eval" not in tools_src:
        fail("Pi extension missing /v1/context-cognition/curate/eval route")
    if "/v1/context-cognition/curate/optimize" not in tools_src:
        fail("Pi extension missing /v1/context-cognition/curate/optimize route")
    if "/v1/context-cognition/optimizer/artifacts" not in tools_src:
        fail("Pi extension missing /v1/context-cognition/optimizer/artifacts route")

    # Contracts
    contracts_src = (ROOT / "apps/pi-extension/src/tool-contracts.ts").read_text()
    for tool in [
        "focusa_context_cognition_curate_eval",
        "focusa_context_cognition_curate_optimize",
        "focusa_context_cognition_optimizer_artifacts",
    ]:
        if f'"{tool}"' not in contracts_src:
            fail(f"tool-contracts.ts missing contract: {tool}")
    ntt = re.search(r'const TOOL_NEXT_TOOLS: Record<string, string\[]> = ([\s\S]*?)\n\};', contracts_src)
    if not ntt:
        fail("TOOL_NEXT_TOOLS not found")
    next_tools = ntt.group(1)
    for tool in [
        "focusa_context_cognition_curate_eval",
        "focusa_context_cognition_curate_optimize",
        "focusa_context_cognition_optimizer_artifacts",
    ]:
        if f'"{tool}"' not in next_tools:
            fail(f"TOOL_NEXT_TOOLS missing entry: {tool}")

    # JSON registry
    registry = json.loads((ROOT / "docs/current/focusa-tool-contracts.json").read_text())
    for tool in [
        "focusa_context_cognition_curate_eval",
        "focusa_context_cognition_curate_optimize",
        "focusa_context_cognition_optimizer_artifacts",
    ]:
        if not any(c.get("name") == tool for c in registry.get("contracts", [])):
            fail(f"focusa-tool-contracts.json missing: {tool}")
    if registry.get("tool_count", 0) < 72:
        fail(f"tool_count expected >= 72, got {registry.get('tool_count')}")

    # Choreography
    choreo = json.loads((ROOT / "docs/current/focusa-tool-choreography.json").read_text())
    if choreo.get("tool_count", 0) < 72:
        fail(f"choreography tool_count expected >= 72, got {choreo.get('tool_count')}")
    for tool in [
        "focusa_context_cognition_curate_eval",
        "focusa_context_cognition_curate_optimize",
        "focusa_context_cognition_optimizer_artifacts",
    ]:
        if not any(e.get("from") == tool for e in choreo.get("edges", [])):
            fail(f"choreography missing edge from: {tool}")

    # Doc pages
    for tool in [
        "focusa_context_cognition_curate_eval",
        "focusa_context_cognition_curate_optimize",
        "focusa_context_cognition_optimizer_artifacts",
    ]:
        doc = ROOT / f"docs/focusa-tools/tools/{tool}.md"
        if not doc.exists():
            fail(f"missing doc: {doc}")
        doc_src = doc.read_text()
        for marker in ["Purpose", "Expected result", "failure_class", "CQRS"]:
            if marker not in doc_src:
                fail(f"doc {tool} missing marker: {marker}")

    # README
    readme = (ROOT / "README.md").read_text()
    if "**72**" not in readme and "**72 tools" not in readme:
        # Check for any reference to the new tool rows
        for tool in [
            "focusa_context_cognition_curate_eval",
            "focusa_context_cognition_curate_optimize",
            "focusa_context_cognition_optimizer_artifacts",
        ]:
            if f"`{tool}`" not in readme:
                fail(f"README missing tool row: {tool}")

    # Spec 100 CQRS framing
    spec = (ROOT / "docs/100-context-cognition-spec.md").read_text()
    if "15.1 CQRS framing" not in spec:
        fail("docs/100-context-cognition-spec.md missing §15.1 CQRS framing")

    print(f"✓ PASS: focusa_context_cognition_curate_eval + curate_optimize + optimizer_artifacts (Spec 100 P4+P5) wired across core, api, cli, pi, contract, choreo, doc; CQRS read/write split; tool_count={registry.get('tool_count')}")


if __name__ == "__main__":
    main()
