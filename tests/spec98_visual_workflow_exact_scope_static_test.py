#!/usr/bin/env python3
"""Spec98 / focusa-877z.16 visual workflow exact-handle and scope semantics guard."""
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
VISUAL = ROOT / "crates/focusa-api/src/routes/visual_workflow.rs"
COMMANDS = ROOT / "crates/focusa-api/src/routes/commands.rs"
RUNTIME = ROOT / "tests/visual_workflow_evidence_routes_contract_test.sh"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        fail(f"missing {label}: {needle}")


def main() -> None:
    visual = VISUAL.read_text()
    commands = COMMANDS.read_text()
    runtime = RUNTIME.read_text()

    body = visual[visual.find("struct StoreVisualEvidenceBody"):visual.find("impl StoreVisualEvidenceBody")]
    for needle in ["project_root: Option<String>", "continuity_id: Option<String>", "workpoint_id: Option<String>"]:
        require(body, needle, "visual store body scope field")

    for needle in [
        "let handle_id = uuid::Uuid::now_v7();",
        "handle_id: Some(handle_id)",
        "project_root: body.project_root.clone()",
        "continuity_id: body.continuity_id.clone()",
        ".find(|h| h.id == handle_id)",
        ".cloned()",
        '"handle": handle',
        '"scope": {',
        '"workpoint_id": body.workpoint_id',
        '"tool_result_v1"',
        '"reference_store_write"',
    ]:
        require(visual, needle, "visual exact-handle/scope behavior")
    if ".find(|h| h.label ==" in visual:
        fail("visual route must not poll newly-created handles by label")

    payload = commands[commands.find("struct VisualEvidencePayload"):commands.find("impl VisualEvidencePayload")]
    for needle in ["project_root: Option<String>", "continuity_id: Option<String>"]:
        require(payload, needle, "visual command payload scope field")
    command_dispatch = commands[commands.find('"visual.register_reference_artifacts"'):commands.find('"instances.connect"')]
    for needle in ["project_root: p.project_root", "continuity_id: p.continuity_id"]:
        require(command_dispatch, needle, "visual command scope dispatch")

    for needle in [
        "duplicate visual labels return distinct exact handles",
        ".id == .handle.id",
        ".handle.project_root == $root",
        ".handle.continuity_id == $cont",
        ".scope.workpoint_id == $wp",
        "focusa-handle:",
    ]:
        require(runtime, needle, "runtime visual exact/scope assertion")

    require(SUITE.read_text(), "tests/spec98_visual_workflow_exact_scope_static_test.py", "Spec98 suite wiring")
    print("✓ PASS: Spec98 visual workflow exact-handle/scope contract ok")


if __name__ == "__main__":
    main()
