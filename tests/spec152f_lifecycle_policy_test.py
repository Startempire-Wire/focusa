#!/usr/bin/env python3
"""Build-independent policy gate for Spec 152F.05.04 lifecycle receipts.

Atom focusa-vbcqu.20.14.40 (152F.05.04): bind installer and lifecycle
receipts to the simple policy.

Exact verification:
    python3 tests/170_focusa_installer_entitlement_activation_gate_test.py \\
        && python3 tests/spec152f_lifecycle_policy_test.py

The gate checks the Spec 150A lifecycle acceptance receipts and the official
and source installers without building anything:

1. `LifecycleReceiptV1` records a `LifecyclePolicyBinding` carrying the digest
   of the single embedded entitlement policy, the canonical capability
   family, the entitlement state, the lease sequence, the recovery posture,
   and the reconciled product-ready flag; receipts never record raw keys,
   tokens, credentials, or customer PII (structural by construction).
2. `reconcile_policy` re-checks the recorded binding against the canonical
   embedded registry and this receipt's own authority fields, and the Rust
   unit tests prove positive, recovery, and tamper-failed paths.
3. Official installs (`scripts/install-focusa.sh`, `scripts/install-focusa.ps1`)
   and the source Rust installer use the same policy: both delegate to the
   canonical installer/activation client, never create local Evaluation, never
   accept raw license keys, and never duplicate product/price/grant logic.
4. The first useful value mutation requires an authority-backed
   Evaluation/paid lease: the lifecycle orchestrator demands a canonical
   entitlement decision, product grant, and feature grant before mutation and
   the installer performs entitlement acquisition before asset download.
"""

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RECEIPTS = ROOT / "crates/focusa-core/src/install_lifecycle/receipts.rs"
RECEIPT_TESTS = ROOT / "crates/focusa-core/src/install_lifecycle/receipt_tests.rs"
ORCHESTRATOR = ROOT / "crates/focusa-core/src/install_lifecycle/orchestrator.rs"
INSTALL = ROOT / "crates/focusa-cli/src/commands/install.rs"
SH = ROOT / "scripts/install-focusa.sh"
PS1 = ROOT / "scripts/install-focusa.ps1"

RECEIPT_MARKERS = [
    "LifecyclePolicyBinding",
    "pub policy_digest",
    "pub capability_family",
    "pub entitlement_state",
    "pub lease_sequence",
    "pub recovery_posture",
    "pub product_ready",
    "pub fn from_acceptance",
    "pub fn reconcile_policy",
    "embedded_entitlement_policy_registry",
    "PolicyReconciliation",
    "LIFECYCLE_CAPABILITY_FAMILIES",
    "focusa.lifecycle_policy_binding.v1",
]

FAMILIES = {
    "account_recovery",
    "read_projection",
    "base_focusa",
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
    "customer_data_export",
    "internal_maintenance",
}

RECONCILE_TEST_NAMES = [
    "receipt_records_canonical_simple_policy_binding",
    "receipt_recovery_posture_records_recovery_family_state_and_sequence",
    "receipt_tampered_policy_binding_fails_reconciliation",
    "receipt_policy_binding_never_records_raw_key_material",
]

ORCHESTRATOR_MARKERS = [
    "EntitlementRequired",
    "ProductGrantRequired",
    "FeatureGrantRequired",
    "EntitlementBlocked",
    "allows_product_execution_at",
    "requires_entitlement",
]

FORBIDDEN_RAW_FRAGMENTS = [
    "license_key",
    "private_key",
    "secret_key",
    "signing_key",
    "customer_email",
    "access_token",
    "card_number",
]

SAME_POLICY_MARKERS_SH = [
    "authority-issued only",
    "never creates local evaluation state",
    "Raw license keys and email addresses are intentionally not accepted",
    "product/price/grant/feature",
    "ARGS=(install --target=",
]

SAME_POLICY_MARKERS_PS1 = [
    "authority-issued only",
    "never creates local evaluation state",
    "Raw license keys",
    "product/price/grant/feature",
    '@("install", "--target=',
]


def struct_body(text: str, struct_name: str) -> str:
    """Return the body of a Rust struct declaration (between braces)."""
    start = text.index(f"pub struct {struct_name} {{")
    depth = 0
    for i in range(start, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[start : i + 1]
    raise AssertionError(f"struct {struct_name} body not found")


def main() -> int:
    failures: list[str] = []

    receipts = RECEIPTS.read_text(encoding="utf-8")
    receipt_tests = RECEIPT_TESTS.read_text(encoding="utf-8")
    orchestrator = ORCHESTRATOR.read_text(encoding="utf-8")
    install = INSTALL.read_text(encoding="utf-8")
    sh = SH.read_text(encoding="utf-8")
    ps1 = PS1.read_text(encoding="utf-8")

    # --- 1. Receipts record the canonical simple-policy binding ---
    for marker in RECEIPT_MARKERS:
        if marker not in receipts:
            failures.append(f"receipts.rs missing marker: {marker}")

    receipt_struct = struct_body(receipts, "LifecycleReceiptV1")
    if "pub policy_binding: LifecyclePolicyBinding" not in receipt_struct:
        failures.append("LifecycleReceiptV1 does not carry the policy binding")

    binding_struct = struct_body(receipts, "LifecyclePolicyBinding")
    for field in [
        "schema_version",
        "policy_digest",
        "capability_family",
        "entitlement_state",
        "lease_sequence",
        "recovery_posture",
        "product_ready",
    ]:
        if f"pub {field}:" not in binding_struct:
            failures.append(f"LifecyclePolicyBinding missing field: {field}")

    family_source = "\n".join(
        line
        for line in receipts.splitlines()
        if "capability_family" in line and "=" in line and ":" in line
    )
    for family in FAMILIES:
        if f'"{family}"' not in family_source and f'"{family}",' not in receipts:
            failures.append(f"canonical family label absent from receipts.rs: {family}")

    # The binding must come from the embedded registry, never caller input.
    if "LifecyclePolicyBinding::from_acceptance(" not in receipts:
        failures.append("receipt construction does not derive the policy binding")

    # --- 2. No raw key material in receipts (structural) ---
    for fragment in FORBIDDEN_RAW_FRAGMENTS:
        if re.search(rf"pub [a-z0-9_]*{re.escape(fragment)}[a-z0-9_]*:", receipt_struct):
            failures.append(f"receipt struct declares raw material field: {fragment}")
        if re.search(rf"pub [a-z0-9_]*{re.escape(fragment)}[a-z0-9_]*:", binding_struct):
            failures.append(f"policy binding declares raw material field: {fragment}")
    # No raw field may be serialized into the binding JSON.
    binding_serde = binding_struct
    for fragment in ["email", "token", "credential"]:
        if re.search(rf"\b{fragment}\b", binding_serde):
            failures.append(f"policy binding text mentions raw fragment: {fragment}")

    # --- 3. Reconcile tests exist and exercise positive/recovery/tamper paths ---
    for name in RECONCILE_TEST_NAMES:
        if f"fn {name}" not in receipt_tests:
            failures.append(f"receipt_tests.rs missing test: {name}")
    if "reconcile_policy" not in receipt_tests:
        failures.append("receipt_tests.rs does not exercise reconcile_policy")

    # --- 4. First useful value mutation requires authority-backed lease ---
    for marker in ORCHESTRATOR_MARKERS:
        if marker not in orchestrator:
            failures.append(f"orchestrator.rs missing marker: {marker}")
    if "focusa.install.channel." not in orchestrator:
        failures.append("orchestrator does not require channel feature grants")

    # --- 5. Official and source installs use the same policy ---
    for marker in SAME_POLICY_MARKERS_SH:
        if marker not in sh:
            failures.append(f"install-focusa.sh missing marker: {marker}")
    for marker in SAME_POLICY_MARKERS_PS1:
        if marker not in ps1:
            failures.append(f"install-focusa.ps1 missing marker: {marker}")

    # No installer creates Evaluation or persists a local evaluation license.
    for path, name in [
        (install, "install.rs"),
        (sh, "install-focusa.sh"),
        (ps1, "install-focusa.ps1"),
    ]:
        if 'return Ok("eval".to_string())' in path:
            failures.append(f"{name} self-issues Evaluation")
        if "persist_eval_license" in path:
            failures.append(f"{name} persists a local evaluation license")
        if re.search(r"create_customer|grant_entitlement|issue_lease", path):
            failures.append(f"{name} duplicates authority-owned grant logic")

    # No caller-controlled product/price/grants in the bootstrappers.
    for path, name in [(sh, "install-focusa.sh"), (ps1, "install-focusa.ps1")]:
        for fragment in ["--price", "--product=", "--grant", "--plan="]:
            if fragment in path:
                failures.append(f"{name} exposes a caller-controlled commercial flag: {fragment}")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}")
        return 1

    print(
        "Spec152f lifecycle policy binding gate: PASS "
        f"({len(RECONCILE_TEST_NAMES)} reconcile tests, {len(FAMILIES)} canonical families)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
