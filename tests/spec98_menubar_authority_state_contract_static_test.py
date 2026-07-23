#!/usr/bin/env python3
"""Spec98 / focusa-877z.8.7 menubar authority state contract guard."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
API = ROOT / "apps/menubar/src/lib/api.ts"
WORKPOINT = ROOT / "apps/menubar/src/lib/components/WorkpointPeek.svelte"
PROOF = ROOT / "apps/menubar/src/lib/components/ProofPeek.svelte"
DOC = ROOT / "docs/current/FOCUSA_MENUBAR_AUTHORITY_STATE_CONTRACT.md"
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
        API,
        [
            "advisory",
            "stale",
            "scope_status",
            "scope_source",
            "side_effects",
            "evidence_refs",
            "tool_result_v1",
        ],
        "menubar API normalization",
    )
    require(
        WORKPOINT,
        [
            "advisory",
            "stale",
            "scopeStatus",
            "scopeSource",
            "scope:",
            "source:",
            "non-canonical",
        ],
        "WorkpointPeek authority chips",
    )
    require(
        PROOF,
        [
            "normalizeToolResult",
            "sideEffects",
            "scopeStatus",
            "side:",
            "authority_posture?.authority_status",
            "history_only",
        ],
        "ProofPeek proof/side-effect rendering",
    )
    require(
        DOC,
        [
            "Menubar is a read/display surface",
            "canonical",
            "advisory",
            "degraded",
            "stale",
            "scope.scope_status",
            "side_effects",
            "evidence_refs",
            "must never mint canonical authority",
        ],
        "menubar authority contract doc",
    )
    if (
        "tests/spec98_menubar_authority_state_contract_static_test.py"
        not in SUITE.read_text()
    ):
        fail("Spec98 suite does not run menubar authority contract guard")
    if (
        "tests/spec98_menubar_authority_state_contract_static_test.py"
        not in PROOF_SUITE.read_text()
    ):
        fail(
            "proof suite static contract does not include menubar authority contract guard"
        )
    print("✓ PASS: Spec98 menubar authority state contract ok")


if __name__ == "__main__":
    main()
