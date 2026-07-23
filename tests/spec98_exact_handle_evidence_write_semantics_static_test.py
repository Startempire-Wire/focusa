#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.9 exact-handle evidence write semantics guard."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
ECS = ROOT / "crates/focusa-api/src/routes/ecs.rs"
VISUAL = ROOT / "crates/focusa-api/src/routes/visual_workflow.rs"
TOOLS = ROOT / "apps/pi-extension/src/tools.ts"
DOC = ROOT / "docs/current/FOCUSA_EXACT_HANDLE_EVIDENCE_WRITE_SEMANTICS.md"
SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_regression_suite.sh"
PROOF_SUITE = ROOT / "tests/spec98_runtime_bleed_crdt_proof_suite_static_test.py"


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def require(path: Path, terms: list[str], label: str) -> None:
    text = path.read_text()
    for term in terms:
        if term not in text:
            fail(f"{label} missing {term}")


def main() -> None:
    require(
        ECS,
        [
            "let handle_id = uuid::Uuid::now_v7();",
            "handle_id: Some(handle_id)",
            ".find(|h| h.id == handle_id)",
            '"id": handle.id',
            '"handle": handle',
            "handle_authority_posture",
            "evidence_handle_only_not_object_truth",
            "legacy_scope_missing",
        ],
        "ECS exact handle route",
    )
    require(
        VISUAL,
        [
            "let handle_id = uuid::Uuid::now_v7();",
            "Some(handle_id)",
            "ReferenceStore::new",
            "any(|h| h.id == handle.id)",
            '"id": handle.id',
            '"handle": handle',
            "focusa-handle:",
            '"project_root": body.project_root',
            '"continuity_id": body.continuity_id',
            '"workpoint_id": body.workpoint_id',
        ],
        "Visual workflow exact handle route",
    )
    require(
        TOOLS,
        [
            "focusa_evidence_capture",
            "focusa_workpoint_link_evidence",
            "focusa_browser_diagnostics_intake",
            "evidence_ref",
            "tool_result_v1",
            "evidence_refs",
            "workpoint_id",
            "continuity_id",
            "project_root",
        ],
        "Pi evidence tools",
    )
    require(
        DOC,
        [
            "pre-generate `handle_id`",
            "h.id == handle_id",
            "Duplicate labels cannot select the wrong artifact",
            "legacy_scope_missing",
            "evidence_handle_only_not_object_truth",
        ],
        "exact-handle docs",
    )
    if (
        "tests/spec98_exact_handle_evidence_write_semantics_static_test.py"
        not in SUITE.read_text()
    ):
        fail("Spec98 suite does not run exact-handle evidence guard")
    if (
        "tests/spec98_exact_handle_evidence_write_semantics_static_test.py"
        not in PROOF_SUITE.read_text()
    ):
        fail("proof suite static contract does not include exact-handle evidence guard")
    print("✓ PASS: Spec98 exact-handle evidence write semantics ok")


if __name__ == "__main__":
    main()
