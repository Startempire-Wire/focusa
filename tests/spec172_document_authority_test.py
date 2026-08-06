#!/usr/bin/env python3
"""Static registration gate for the Spec 172 licensing authority addendum."""

from __future__ import annotations

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = "docs/172-focusa-spec152-license-type-and-surface-entitlement-governance-addendum.md"
CONTRACTS = (
    "docs/contracts/spec152-document-set.v1.yaml",
    "docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml",
    "docs/contracts/spec152-final-audit-status.v1.yaml",
)


def read(path: str) -> str:
    full = ROOT / path
    if not full.is_file():
        raise AssertionError(f"missing required authority surface: {path}")
    return full.read_text(encoding="utf-8")


def require(text: str, path: str, tokens: tuple[str, ...]) -> list[str]:
    return [f"{path}: missing {token!r}" for token in tokens if token not in text]


def main() -> int:
    failures: list[str] = []
    spec = read(SPEC)
    documents = {path: read(path) for path in CONTRACTS}

    failures += require(spec, SPEC, (
        "Status:** Normative, release-blocking addendum",
        "verified_no_license",
        "focusa_operator_lifetime_v1",
        "uiai_operator_lifetime_v1",
        "focusa_uiai_operator_bundle_lifetime_v1",
        "$1,254.60",
    ))

    for path, text in documents.items():
        failures += require(text, path, (SPEC, "spec_158: excluded"))

    failures += require(documents[CONTRACTS[0]], CONTRACTS[0], (
        "license_type_and_surface_governance_authority:",
        "narrow_supersession_only:",
        "preserved_authority:",
        "tests/spec172_document_authority_test.py",
    ))
    failures += require(documents[CONTRACTS[1]], CONTRACTS[1], (
        "license_type_and_surface_governance:",
        "active_release_blocking_license_type_and_surface_governance_addendum",
        "verified_no_license is a permanent authority-signed limited posture",
        "Bundle is USD 1254.60 and exactly unions the two Operator grants",
        "Spec 172 narrowly supersedes only",
    ))
    failures += require(documents[CONTRACTS[2]], CONTRACTS[2], (
        "spec172_registered: true",
        "spec172_policy_status: implementation_open",
        "spec172_supersession_scope: Evaluation product-model Bundle-price and future-family-product-inheritance conflicts only",
        "spec172_preserved_authority: identity EDD key lease refund-revoke node sequence recovery privacy customer-data-preservation",
        "distribution_status: blocked",
    ))

    document_set = documents[CONTRACTS[0]]
    if document_set.index(SPEC) < document_set.index("docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md"):
        failures.append(f"{CONTRACTS[0]}: Spec 172 must be registered after Spec 152F")

    if failures:
        print("Spec 172 document authority test FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Spec 172 document authority test passed")
    print("authority_contracts=3")
    print("supersession_scope=narrow")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
