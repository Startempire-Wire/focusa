#!/usr/bin/env python3
"""Spec 172 §20 — complete runtime policy matrix acceptance receipt
(atom focusa-vbcqu.20.15.24, 172.03.07, lane spec152f / WPUIAI).

This gate is the cross-layer runtime acceptance receipt for the complete Spec
172 runtime policy matrix. It replays every matrix case — unverified,
verified-limited (Focusa + UIAI), Focusa Operator, UIAI Operator, Bundle,
refunded/revoked, offline, corrupt, unknown family/product/type, future
Navigator, dynamic tool, node/seat, and resource — through the canonical
contract layers (PHP issuance/assertion gates and the Spec 172 Python layer
gates, all read-only and replayable), statically pins the Spec 172 Rust
registry/reducer/core/API surfaces that implement the same fail-closed matrix,
and verifies the resolver/core/API equivalence vectors added by this atom
(`cargo test --workspace spec172_runtime_policy` runs them; the shell records
its exit code separately).

The receipt emits one bounded JSON line. No raw email, key, token, customer
row, credential, or card data ever appears; all identifiers are synthetic.

Exact verification:
    python3 tests/spec172_runtime_policy_acceptance_test.py \
        && cargo test --workspace spec172_runtime_policy
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"

PHP = "/usr/local/bin/php" if Path("/usr/local/bin/php").exists() else shutil.which("php")

POSITIVE = 0
NEGATIVE = 0


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


# ── Matrix case → replay gate mapping (read-only, fast, no cargo/network) ──

# Each entry: (case, language, command, path). Every gate is a previously
# accepted, pinned Spec 172 / 152F contract gate; this receipt re-runs them to
# prove the replay layer of the complete matrix from this commit.
REPLAY_GATES: list[tuple[str, str, list[str]]] = [
    ("unverified", "php", ["php", "tests/spec172_limited_assertion_test.php"]),
    ("verified_limited_focusa_uiai", "php", ["php", "tests/spec172_limited_assertion_test.php"]),
    ("verified_limited_uiai", "py", ["python3", "tests/spec172_uiai_limited_mode_test.py"]),
    ("verified_limited_allowlists", "py", ["python3", "tests/spec172_family_inheritance_test.py"]),
    ("focusa_operator", "php", ["php", "tests/spec172_focusa_operator_issuance_test.php"]),
    ("uiai_operator", "php", ["php", "tests/spec172_uiai_operator_issuance_test.php"]),
    ("bundle", "php", ["php", "tests/spec172_bundle_composition_test.php"]),
    ("refunded_revoked", "php", ["php", "tests/spec172_refund_downgrade_test.php"]),
    ("offline_bounded", "py", ["python3", "tests/spec172_lifetime_credential_test.py"]),
    ("corrupt_tampered", "php", ["php", "tests/spec172_verified_access_schema_test.php"]),
    ("unknown_type", "py", ["python3", "tests/spec172_license_type_contract_test.py"]),
    ("unknown_vector", "py", ["python3", "tests/spec172_limited_assertion_vector_test.py"]),
    ("future_navigator", "py", ["python3", "tests/spec172_license_type_lifecycle_test.py"]),
    ("dynamic_tool", "py", ["python3", "tests/spec172_dynamic_operation_manifest_test.py"]),
    ("node_seat_resource", "php", ["php", "tests/spec172_uiai_operator_issuance_test.php"]),
    ("node_seat_bundle", "php", ["php", "tests/spec172_bundle_composition_test.php"]),
    ("public_commerce", "py", ["python3", "tests/spec172_public_commerce_baseline_test.py"]),
    ("no_sales_cutover", "php", ["php", "tests/spec172_no_sales_cutover_test.php"]),
    ("no_sales_inventory", "py", ["python3", "tests/spec172_no_sales_inventory_test.py"]),
    ("presenter_inheritance", "py", ["python3", "tests/spec172_public_facade_convergence_test.py"]),
    ("call_stack_inheritance", "py", ["python3", "tests/spec172_call_stack_contract_test.py"]),
]

POLICY_RS = ROOT / "crates/focusa-license/src/entitlement_policy.rs"
DYNAMIC_RS = ROOT / "crates/focusa-license/src/dynamic_operation_manifest.rs"
CORE_GUARD_RS = ROOT / "crates/focusa-core/src/entitlement_execution_guard.rs"
CORE_LICENSE_RS = ROOT / "crates/focusa-core/src/license.rs"
API_MIDDLEWARE_RS = ROOT / "crates/focusa-api/src/middleware/spec172_runtime_policy.rs"
API_ENTITLEMENT_RS = ROOT / "crates/focusa-api/src/middleware/entitlement.rs"
LICENSE_TEST_RS = ROOT / "crates/focusa-license/tests/spec172_runtime_policy.rs"
CORE_TEST_RS = ROOT / "crates/focusa-core/tests/spec172_runtime_policy.rs"


def replay_layer(php: str) -> dict[str, int]:
    """Run every replay gate once and record its real exit code."""
    results: dict[str, int] = {}
    for case, lang, command in REPLAY_GATES:
        argv = [php if part == "php" else part for part in command]
        proc = subprocess.run(
            argv,
            cwd=str(ROOT),
            capture_output=True,
            text=True,
            timeout=600,
        )
        results[f"{case}::{lang}"] = proc.returncode
        if proc.returncode != 0:
            raise AssertionError(
                f"replay gate failed rc={proc.returncode} for case={case} cmd={command}\n"
                f"{proc.stdout[-2000:]}\n{proc.stderr[-2000:]}"
            )
    return results


def static_pin_rust_surfaces() -> dict[str, int]:
    """Bound the Spec 172 Rust registry/reducer/core/API surfaces that carry
    the same fail-closed matrix (the cargo filter executes the vectors)."""
    policy = POLICY_RS.read_text(encoding="utf-8")
    dynamic = DYNAMIC_RS.read_text(encoding="utf-8")
    guard = CORE_GUARD_RS.read_text(encoding="utf-8")
    core_license = CORE_LICENSE_RS.read_text(encoding="utf-8")
    api_middleware = API_MIDDLEWARE_RS.read_text(encoding="utf-8")
    api_entitlement = API_ENTITLEMENT_RS.read_text(encoding="utf-8")
    license_test = LICENSE_TEST_RS.read_text(encoding="utf-8")
    core_test = CORE_TEST_RS.read_text(encoding="utf-8")

    # Pure resolver: the complete 7-state × 9-family matrix reducer exists and
    # the Spec 172 overlay (verified_no_license posture, no Evaluation state)
    # is present.
    expect("pub const fn reduce_entitlement_state" in policy, "reducer exists")
    expect("State::VerifiedNoLicense" in policy, "verified_no_license is a resolver state")
    expect("PolicyEntitlementState" in policy, "policy states defined")
    for posture in ["Allow", "Read", "Base", "Feature", "Deny"]:
        expect(f"Posture::{posture}" in policy, f"posture {posture} is a resolver decision")
    for family in [
        "AccountRecovery",
        "ReadProjection",
        "BaseFocusa",
        "Automation",
        "TeamRemote",
        "ReleaseProof",
        "PremiumUpdates",
        "CustomerDataExport",
        "InternalMaintenance",
    ]:
        expect(f"Family::{family}" in policy, f"capability family {family} registered")
    expect(
        "SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES" in policy
        and "SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES" in policy
        and "SPEC172_FOCUSA_OPERATOR_V1_FAMILIES" in policy,
        "verified-limited and Operator family registries exist",
    )

    # Base product gate: no anonymous product capability; no caller-controlled
    # product or commercial right.
    expect("resolve_base_focusa_product" in policy, "base product resolver exists")
    expect("if product != \"focusa\"" in policy, "only the exact authority product id gates")
    expect("BaseProductDecision::Limited" in policy, "verified no-license resolves Limited")
    expect("BaseProductDecision::Denied" in policy, "denied posture exists")

    # License Types / Bundle: Operator codes, exact union, future exclusion.
    expect("FocusaOperatorLifetimeV1" in policy, "Focusa Operator License Type registered")
    expect("UiaiOperatorLifetimeV1" in policy, "UIAI Operator License Type registered")
    expect("operator_bundle_v1" in policy, "bundle union constructor exists")
    expect("MalformedBundleUnion" in policy, "bundle rejects non-union grants")
    expect("OperatorSeats" in policy and "SharedNodeLimit" in policy, "seat/node types exist")
    expect("HostedExcluded" in policy, "hosted resources excluded")
    expect(
        "classify_operator_family_inheritance" in policy
        and "ExcludedPendingAssignment" in policy,
        "future families excluded pending explicit assignment",
    )

    # Dynamic tools / generated UI: trusted metadata required, unknown surfaces
    # quarantine; a tool cannot self-label recovery; no client policy fields.
    expect("verify_dynamic_operation_manifest" in dynamic, "dynamic manifest verifier exists")
    expect("verify_generated_ui_action" in dynamic, "generated-UI verifier exists")
    expect("QuarantinedSelfLabeledRecovery" in dynamic, "self-labeled recovery quarantines")
    expect("QuarantinedClientSelectedPolicy" in dynamic, "client-selected policy quarantines")
    expect("FORBIDDEN_CLIENT_POLICY_FIELDS" in dynamic, "forbidden client policy fields pinned")
    expect("ENTITLEMENT_POLICY_UNKNOWN" in dynamic, "stable unknown-policy error exists")
    expect("REGISTERED_PRODUCT_OWNERS" in dynamic, "registered product owners pinned")

    # Core guard: shared execution chokepoint codes and the one-project guard.
    for code in [
        "ENTITLEMENT_BASE_REQUIRED",
        "ENTITLEMENT_FEATURE_REQUIRED",
        "ENTITLEMENT_REQUIRED",
        "ENTITLEMENT_ROUTE_UNCLASSIFIED",
    ]:
        expect(code in guard, f"core guard stable code {code} exists")
    expect("evaluate_entitlement_execution" in guard, "core execution guard exists")
    expect("evaluate_entitlement_execution_for_project" in guard, "project-aware guard exists")
    expect(
        "require_base_product" in core_license and "BaseProductRequired" in core_license,
        "base product core gate exists and fails closed",
    )

    # API: this atom's runtime-policy vectors are compiled under cargo test and
    # the route gate owns no presenter policy.
    expect("spec172_runtime_policy" in api_entitlement, "API runtime-policy vectors wired")
    expect("route_entitlement_denial" in api_middleware, "API route gate denial helper used")
    expect(
        "route_entitlement_denial" in API_ENTITLEMENT_RS.read_text(encoding="utf-8"),
        "API route gate exists",
    )

    # The cargo filter `spec172_runtime_policy` must execute real vectors in
    # the workspace (resolver + core + API).
    license_vector_count = len(
        re.findall(r"#\[test\]\nfn spec172_runtime_policy_", license_test)
    )
    core_vector_count = len(re.findall(r"#\[test\]\nfn spec172_runtime_policy_", core_test))
    api_vector_count = len(
        re.findall(r"#\[test\]\nfn spec172_runtime_policy_", api_middleware)
    )
    expect(license_vector_count >= 7, f"resolver/registry vectors exist ({license_vector_count})")
    expect(core_vector_count >= 4, f"core guard vectors exist ({core_vector_count})")
    expect(api_vector_count >= 3, f"API route-gate vectors exist ({api_vector_count})")

    return {
        "license_vectors": license_vector_count,
        "core_vectors": core_vector_count,
        "api_vectors": api_vector_count,
    }


def no_legacy_download_453_mapping() -> None:
    """Spec 172 §16.3/§19: Download 453 (and legacy WPUIAI products) remain
    unrelated and quarantined; no implicit legacy 453 → License Type mapping."""
    inventory = json.loads(
        (CONTRACTS / "spec172-no-sales-inventory.v1.json").read_text(encoding="utf-8")
    )
    dedicated = json.loads(
        (CONTRACTS / "spec172-edd-operator-v1-downloads.v1.json").read_text(encoding="utf-8")
    )
    legacy = dedicated.get("authority", {}).get("legacy_download_ids", [])
    expect(453 in legacy, "Download 453 is classified legacy/quarantined")
    expect(
        dedicated.get("authority", {}).get("forbidden_implicit_download") == 453,
        "implicit legacy Download 453 mapping is forbidden",
    )
    expect(
        dedicated.get("authority", {}).get("checkout_enabled") is False,
        "checkout stays disabled until validation passes",
    )
    expect(
        inventory.get("schema") == "focusa.spec172.no_sales_inventory.v1",
        "no-sales inventory schema pinned",
    )


def hygiene() -> None:
    """No raw email, secret, raw key, or card evidence in this receipt's
    surfaces or the bounded evidence output."""
    EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
    SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
    PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
    CARD_RE = re.compile(r"\b(?:\d[ -]?){13,16}\b")
    for path in [
        POLICY_RS,
        DYNAMIC_RS,
        CORE_GUARD_RS,
        API_MIDDLEWARE_RS,
        LICENSE_TEST_RS,
        CORE_TEST_RS,
    ]:
        raw = path.read_text(encoding="utf-8")
        expect(EMAIL_RE.search(raw) is None, f"{path.name} carries an email literal")
        expect(
            SECRET_RE.search(raw) is None
            and PRIVATE_KEY_RE.search(raw) is None
            and CARD_RE.search(raw) is None,
            f"{path.name} carries a secret, raw private key, or card number",
        )


def main() -> int:
    if PHP is None:
        raise AssertionError("php runtime is required for the replay layer")

    replay = replay_layer(PHP)
    expect(all(rc == 0 for rc in replay.values()), "every replay gate exited 0")
    expect(len(replay) == len(REPLAY_GATES), "all replay gates ran exactly once")

    vector_counts = static_pin_rust_surfaces()
    no_legacy_download_453_mapping()
    hygiene()

    summary = {
        "schema": "focusa.spec172.runtime_policy_acceptance.v1",
        "atom": "focusa-vbcqu.20.15.24",
        "result": "passed",
        "replay_gates": len(replay),
        "replay_exit_codes_all_zero": True,
        "replay_cases": sorted({case for case, _lang, _cmd in REPLAY_GATES}),
        "rust_vectors": vector_counts,
        "cargo_filter": "cargo test --workspace spec172_runtime_policy",
        "static_positive_checks": POSITIVE,
        "static_negative_checks": NEGATIVE,
        "legacy_download_453_quarantined": True,
        "checkout_blocked": True,
        "evidence_path": "docs/evidence/spec172/focusa-vbcqu.20.15.24-acceptance.txt",
    }
    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
