#!/usr/bin/env python3
"""Release gate for active Focusa licensing/onboarding documentation.

Historical specs/evidence may describe old behavior. Active guides, agent entry points,
and packaged lifecycle runbooks must not recommend self-issued Evaluation or conflate
source/pairing/local tokens with entitlement.
"""

from __future__ import annotations

import hashlib
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]

OPERATOR_GUIDES = [
    "README.md",
    "LICENSE-FAQ.md",
    "docs/INSTALL_PURCHASE_PUBLIC_STATUS.md",
    "docs/PHASE2_OPERATOR_PREVIEW.md",
    "docs/current/FIRST_RUN_FLOW.md",
    "docs/current/INSTALLER_UPDATE_POLICY.md",
    "docs/current/FOCUSA_FRIENDLY_ONBOARDING.md",
    "docs/current/COMMERCIAL_PACKAGING.md",
    "docs/agent/01-focusa-agent-docs-index.md",
    ".pi/skills/focusa-install-lifecycle/references/01-focusa-install-lifecycle-runbook.md",
    "apps/pi-extension/skills/focusa-install-lifecycle/references/01-focusa-install-lifecycle-runbook.md",
]

NORMATIVE_FILES = [
    "docs/150a-spec152-entitlement-overlay-and-lifecycle-integration.md",
    "docs/152-mandatory-authority-licensing-evaluation-entitlements-and-unified-onboarding-spec.md",
    "docs/152a-protected-distribution-private-feature-capsules-and-anti-tamper-spec.md",
    "docs/152e-edd-centered-universal-multi-surface-licensing-and-branded-facade-addendum.md",
    "docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md",
    "docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml",
]

ACTIVE_FILES = OPERATOR_GUIDES + NORMATIVE_FILES

FORBIDDEN_ACTIVE_PATTERNS = [
    "curl -fsS https://install.focusa.dev/focusa | bash -s -- --eval",
    "bash scripts/install-focusa.sh --dry-run --eval",
    "scripts/install-focusa.sh --dry-run --eval",
]

REQUIRED_CONCEPT_GROUPS = {
    "mandatory_spec": ["Spec 152", "spec152"],
    "authority_issued": ["authority-issued", "authority issued"],
    "recovery_posture": ["recovery"],
}

REQUIRED_MATRIX_TOKENS = [
    "docs/152e-edd-centered-universal-multi-surface-licensing-and-branded-facade-addendum.md",
    "docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md",
    "deployment://install.focusa.dev/custom-license-registry",
    "activation://unverified-email",
    "facade://non-wpuiai-domains",
    "evaluation://installer-local",
    "docs/150-focusa-guided-install-first-project-and-lifecycle-master-spec.md",
    "docs/current/PORTABILITY_AUDIT.md",
    "docs/INSTALL_PURCHASE_PUBLIC_STATUS.md",
    "docs/PHASE2_OPERATOR_PREVIEW.md",
    "scripts/install-focusa.sh",
    "scripts/install-focusa.ps1",
    "crates/focusa-license/src/lib.rs",
    "crates/focusa-core/src/license.rs",
    "apps/menubar/src/lib/components/FirstRunWizard.svelte",
    "docs/current/FIRST_RUN_FLOW.md",
]


def read(path: str) -> str:
    full = ROOT / path
    if not full.is_file():
        raise AssertionError(f"required active licensing document missing: {path}")
    return full.read_text(encoding="utf-8")


def main() -> int:
    failures: list[str] = []
    contents: dict[str, str] = {}

    for path in ACTIVE_FILES:
        try:
            contents[path] = read(path)
        except AssertionError as exc:
            failures.append(str(exc))

    for path, text in contents.items():
        lower = text.lower()
        for pattern in FORBIDDEN_ACTIVE_PATTERNS:
            if pattern.lower() in lower:
                failures.append(
                    f"{path}: publishes legacy self-issued Evaluation command: {pattern!r}"
                )

    for path in OPERATOR_GUIDES:
        text = contents.get(path, "").lower()
        for concept, alternatives in REQUIRED_CONCEPT_GROUPS.items():
            if not any(alt.lower() in text for alt in alternatives):
                failures.append(f"{path}: missing required concept {concept}: {alternatives}")

    canonical_runbook = contents.get(
        ".pi/skills/focusa-install-lifecycle/references/01-focusa-install-lifecycle-runbook.md",
        "",
    ).encode()
    packaged_runbook = contents.get(
        "apps/pi-extension/skills/focusa-install-lifecycle/references/01-focusa-install-lifecycle-runbook.md",
        "",
    ).encode()
    if canonical_runbook != packaged_runbook:
        failures.append("Focusa install lifecycle runbook and packaged Pi copy are not byte-identical")

    matrix_path = "docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml"
    matrix = contents.get(matrix_path, "")
    for token in REQUIRED_MATRIX_TOKENS:
        if token not in matrix:
            failures.append(f"{matrix_path}: missing contradiction/integration entry {token}")

    correction = contents.get(
        "docs/152e-edd-centered-universal-multi-surface-licensing-and-branded-facade-addendum.md",
        "",
    )
    for token in (
        "WPUIAI.com EDD",
        "No EDD customer",
        "mailbox control is verified",
        "Local `--eval` issuance is forbidden",
        "Spec 158 remains excluded",
    ):
        if token not in correction:
            failures.append(f"Spec 152E correction missing required authority token: {token}")

    simplification = contents.get(
        "docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md",
        "",
    )
    for token in (
        "one base entitlement gate",
        "four optional premium capability families",
        "The unmatched inventory SHALL be reconciled",
        "Dormant dimensions MUST NOT deny customer capability",
        "No entitlement:",
    ):
        if token not in simplification:
            failures.append(f"Spec 152F simplification missing required policy token: {token}")

    overlay = contents.get("docs/150a-spec152-entitlement-overlay-and-lifecycle-integration.md", "")
    for token in (
        "LifecycleEntitlementBinding",
        "LifecycleAcceptanceReceipt",
        "Spec 150 `implementation_verified`",
        "scripts/install-focusa.ps1",
        "UIAI",
    ):
        if token not in overlay:
            failures.append(f"Spec 150A overlay missing required integration token: {token}")

    if failures:
        print("Spec 152 documentation consistency gate FAILED:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    digest = hashlib.sha256()
    for path in ACTIVE_FILES:
        digest.update(path.encode())
        digest.update(b"\0")
        digest.update(contents[path].encode())
        digest.update(b"\0")

    print("Spec 152 documentation consistency gate passed")
    print(f"active_files={len(ACTIVE_FILES)}")
    print(f"documentation_digest=sha256:{digest.hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
