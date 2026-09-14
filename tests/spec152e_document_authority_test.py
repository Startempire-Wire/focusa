#!/usr/bin/env python3
"""Fail closed unless Spec 152E is registered across licensing authority contracts."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = "docs/152e-edd-centered-universal-multi-surface-licensing-and-branded-facade-addendum.md"
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
    assert full.is_file(), f"missing required Spec 152E authority file: {path}"
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
            "WPUIAI.com EDD is the canonical authority",
            "No EDD customer, canonical authority account, checkout, Evaluation, license, node, or lease",
            "mailbox control is verified",
            "Local `--eval` issuance is forbidden",
            "Spec 158 remains excluded",
        ),
    )

    docs = {path: read(path) for path in CONTRACTS}
    for path, text in docs.items():
        require(path, text, (SPEC,))

    require(
        CONTRACTS[0],
        docs[CONTRACTS[0]],
        (
            "work_item_root: focusa-vbcqu.20.13",
            "status: release_blocking",
            "tests/spec152e_document_authority_test.py",
            "installer-local or self-issued Evaluation",
            "split authority registries and direct install-site issuance",
            "implicit EDD product mappings and caller-controlled grants",
            "canonical customer or entitlement creation before mailbox verification",
            "independent entitlement authority on branded facade domains",
        ),
    )
    require(
        CONTRACTS[1],
        docs[CONTRACTS[1]],
        (
            "WPUIAI.com EDD is the sole customer, commerce, human-license, and entitlement authority",
            "pending registration mailbox verification atomic EDD customer promotion",
            "forbidden: customer order Evaluation license node lease price grant or refund authority",
        ),
    )
    require(
        CONTRACTS[2],
        docs[CONTRACTS[2]],
        (
            "current_work_item: focusa-vbcqu.20.13.2",
            "Freeze the deployed WPUIAI.com EDD and install.focusa.dev authority/facade parity",
            "publication: forbidden_until_focusa-vbcqu.20.13.63_and_focusa-vbcqu.20.14.52_close",
        ),
    )
    require(
        CONTRACTS[3],
        docs[CONTRACTS[3]],
        (
            "authority-unverified-email-promotion",
            "authority-split-registry-direct-stripe-issuance",
            "authority-implicit-edd-product-grants",
            "authority-human-key-node-lease-delivery",
            "authority-facade-origin-and-proxy-boundary",
            "authority-paid-evaluator-migration-cutover",
        ),
    )
    require(
        CONTRACTS[4],
        docs[CONTRACTS[4]],
        (
            "blocked_for_new_evaluator_customer_and_stable_distribution",
            "direct install-site or facade license issuance",
            "caller-supplied EDD product price tier feature limit or commercial grant",
            "before mailbox verification",
        ),
    )
    require(
        CONTRACTS[5],
        docs[CONTRACTS[5]],
        (
            "spec152e_registered: true",
            "spec152e_correction_status: in_progress",
            "distribution_status: blocked",
            "publication_rule: forbidden until focusa-vbcqu.20.13.63 and focusa-vbcqu.20.14.52 close",
        ),
    )
    require(
        CONTRACTS[6],
        docs[CONTRACTS[6]],
        (
            "active_release_blocking_correction",
            "deployed_split_authority_superseded",
            "superseded_identity_flow",
            "presenter_only",
            "superseded_evaluation_flow",
            "Spec 158 remains excluded",
        ),
    )

    for path, text in docs.items():
        require(path, text, ("spec_158: excluded",) if path != CONTRACTS[6] else ("Spec 158 remains excluded",))

    print("Spec 152E document authority gate passed")
    print(f"contracts_checked={len(CONTRACTS)}")
    print("distribution_status=blocked")
    print("next_work_item=focusa-vbcqu.20.13.2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
