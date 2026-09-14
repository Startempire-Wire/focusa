#!/usr/bin/env python3
"""Fail closed unless Spec 152F is registered as the simple entitlement policy."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = "docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md"
CONTRACTS = [
    "docs/contracts/spec152-document-set.v1.yaml",
    "docs/contracts/spec152-implementation-owners.v1.yaml",
    "docs/contracts/spec152-next-command.v1.yaml",
    "docs/contracts/spec152-open-code-gaps.v1.yaml",
    "docs/contracts/spec152-release-blocker-summary.v1.yaml",
    "docs/contracts/spec152-final-audit-status.v1.yaml",
    "docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml",
]


def read(path: str) -> str:
    full = ROOT / path
    assert full.is_file(), f"missing required Spec 152F authority file: {path}"
    return full.read_text(encoding="utf-8")


def require(path: str, text: str, tokens: tuple[str, ...]) -> None:
    for token in tokens:
        assert token in text, f"{path}: missing required token {token!r}"


def main() -> int:
    spec = read(SPEC)
    require(
        SPEC,
        spec,
        (
            "one base entitlement gate",
            "four optional premium capability families",
            "## 4. Entitlement-state grid",
            "## 7. Surface inheritance grid",
            "## 9. Dormant future-granularity model",
            "## 10. Future-granularity activation requirements",
            "The unmatched inventory SHALL be reconciled",
            "395 independent paywalls",
            "Spec 158",
        ),
    )

    docs = {path: read(path) for path in CONTRACTS}
    for path, text in docs.items():
        require(path, text, (SPEC,))

    require(
        CONTRACTS[0],
        docs[CONTRACTS[0]],
        (
            "work_item_root: focusa-vbcqu.20.14",
            "final_work_item: focusa-vbcqu.20.14.52",
            "status: release_blocking",
            "tests/spec152f_document_authority_test.py",
            "docs/contracts/spec152f-implementation-taskgraph.v1.json",
            "tests/spec152f_taskgraph_contract_test.py",
            "one base entitlement gate for value-producing Focusa operations",
            "four optional premium capability families",
            "surface inheritance instead of independent REST CLI desktop worker or facade paywalls",
        ),
    )
    require(
        CONTRACTS[1],
        docs[CONTRACTS[1]],
        (
            "simplification_bead_root: focusa-vbcqu.20.14",
            "entitlement policy registry base and premium resolver",
            "future-granularity activation guard",
        ),
    )
    require(
        CONTRACTS[2],
        docs[CONTRACTS[2]],
        (
            "parallel_policy_work_item: focusa-vbcqu.20.14.1",
            "focusa-vbcqu.20.14.2",
            "forbidden_until_focusa-vbcqu.20.13.63_and_focusa-vbcqu.20.14.52_close",
        ),
    )
    require(
        CONTRACTS[3],
        docs[CONTRACTS[3]],
        (
            "focusa-simple-base-entitlement-gate",
            "focusa-surface-policy-inheritance",
            "focusa-permanent-recovery-and-customer-control",
            "focusa-four-premium-family-boundaries",
            "focusa-dormant-future-granularity-activation-guard",
        ),
    )
    require(
        CONTRACTS[4],
        docs[CONTRACTS[4]],
        (
            "simplification_final_work_item: focusa-vbcqu.20.14.52",
            "all 395 baseline surfaces reconciled",
            "four optional premium family boundaries",
        ),
    )
    require(
        CONTRACTS[5],
        docs[CONTRACTS[5]],
        (
            "spec152f_registered: true",
            "spec152f_policy_status: implementation_open",
            "spec152f_task_count: 52",
            "focusa-vbcqu.20.13.63 and focusa-vbcqu.20.14.52",
        ),
    )
    require(
        CONTRACTS[6],
        docs[CONTRACTS[6]],
        (
            "simple_entitlement_policy:",
            "active_release_blocking_simplification_policy",
            "one base product gate four optional premium families",
            "complete focusa-vbcqu.20.14 through focusa-vbcqu.20.14.52",
            "Spec 150 + 150A + 152 + 152E + 152F",
        ),
    )

    for path, text in docs.items():
        require(
            path,
            text,
            ("spec_158: excluded",)
            if path != CONTRACTS[6]
            else ("Spec 158 remains excluded",),
        )

    print("Spec 152F document authority gate passed")
    print(f"contracts_checked={len(CONTRACTS)}")
    print("work_item_root=focusa-vbcqu.20.14")
    print("final_work_item=focusa-vbcqu.20.14.52")
    print("distribution_status=blocked")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
