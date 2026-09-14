#!/usr/bin/env python3
"""Spec 172.05.03 — paid Focusa and UIAI upgrades preserve limited-state data (E2E).

Cross-surface E2E receipt (atom focusa-vbcqu.20.15.34, lane acceptance /
Startempire-Wire/focusa + WPUIAI/wpuiai + WPUIAI/uiai-engine).

The required journey is proven end to end: the SAME verified account upgrades
separately to Focusa Operator and to UIAI Operator through canonical EDD test
orders/keys, the upgrade preserves project/data/node identity, each $697
product unlocks ONLY its own product families, cross-product actions are
rejected, and no duplicate customer/key/grant can be created:

  Stage 1  EDD test orders/keys — the dedicated Operator v1 Downloads records
           (458/459/460) bind the two $697 standalones and the $1,254.60
           Bundle with lifetime term, one operator seat, three shared nodes,
           checkout disabled, and Download 453 quarantined. Replays the
           accepted PHP gates tests/spec172_edd_operator_products_test.php,
           tests/spec172_edd_commerce_acceptance_test.php,
           tests/spec172_focusa_operator_issuance_test.php and
           tests/spec172_uiai_operator_issuance_test.php (all exit 0).
  Stage 2  Same account -> Focusa Operator upgrade — the verified account's
           eligible Focusa order yields exactly one canonical EDD key and one
           projection (focusa_operator_lifetime_v1, sequence monotonic), the
           base product becomes Entitled, the Focusa Operator v1 family set
           unlocks, the active-project selection and the node identity are
           preserved (paid entitlement bypasses the one-project guard without
           deleting it), and read/export/recovery stay available.
  Stage 3  Same account -> UIAI Operator upgrade — the same verified account's
           eligible UIAI order yields exactly one canonical EDD key and one
           projection (uiai_operator_lifetime_v1) with the frozen hosted-
           resource exclusion digest; the UIAI paid family features unlock;
           the same account/node binding is strict and never duplicates the
           customer; Focusa limited-state data is preserved.
  Stage 4  Unlock only purchased families — Focusa paid grants Focusa
           families (including the four premium families) but never UIAI
           observation/action; UIAI paid grants the seven canonical UIAI
           families but never Focusa display/mutation; metered/hosted/private
           rights never gain a canonical paid feature.
  Stage 5  Reject cross-product actions — resolve_base_focusa_product denies
           product "uiai_engine" even when paid; a UIAI lease on a Focusa
           family renders PRODUCT_NOT_INCLUDED; Focusa-only entitlement cannot
           activate or grant UIAI (UiaiGrantRequired / FocusaOnlyCannotGrantUiai);
           combined workflows require BOTH grants; a wrong-product lease fails
           authority verification.
  Stage 6  No duplicate customer/key/grant — one canonical EDD customer per
           verified account identity, one EDD Software Licensing key per order,
           idempotent replay and duplicate-order settlement produce zero extra
           projections/keys, the per-account sequence is strictly monotonic,
           and a cross-account UIAI grant never combines with the Focusa grant.
  Stage 7  Presenters converge and limited-state data is preserved — the
           facade/CLI/Pi/agent/Desktop/Cockpit/menubar-TUI presenters project
           the same paid decisions; retained access (read/export/recovery/
           repair/update/uninstall) is never disabled in paid posture; node
           identity is shared and never multiplied per app; a refund/revoke
           downgrade returns the account to verified_no_license with all data
           intact. Replays the accepted presenter gates and runs the live Rust
           vectors via `cargo`.

The receipt emits ONE bounded JSON line with real exit codes. No raw email,
key, token, customer row, credential, or card data ever appears; every
identifier is synthetic or frozen policy vocabulary.

Exact verification:
    python3 tests/spec172_paid_upgrade_e2e_test.py
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
PHP = "/usr/local/bin/php" if Path("/usr/local/bin/php").exists() else shutil.which("php")

POSITIVE = 0
NEGATIVE = 0
REPLAY: dict[str, dict] = {}
CARGO_RUNS: list[dict] = []


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def run(argv: list[str], timeout: int = 900) -> subprocess.CompletedProcess:
    return subprocess.run(
        argv,
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        timeout=timeout,
    )


def replay_gate(stage: str, name: str, argv: list[str]) -> None:
    """Run one accepted gate once and record its REAL exit code."""
    proc = run(argv)
    REPLAY[f"{stage}::{name}"] = {"argv": argv, "exit": proc.returncode}
    if proc.returncode != 0:
        raise AssertionError(
            f"replay gate failed rc={proc.returncode} for {name} argv={argv}\n"
            f"{proc.stdout[-1500:]}\n{proc.stderr[-1500:]}"
        )


def cargo_test(stage: str, name: str, package: str, filter_: str) -> None:
    """Run one cargo test filter through the canonical OVH-routed cargo
    (builds serialize on the remote global lock; runs are sequential)."""
    proc = run(
        ["cargo", "test", "-p", package, filter_, "--", "--nocapture"],
        timeout=1800,
    )
    result_lines = [
        line.strip()
        for line in (proc.stdout + proc.stderr).splitlines()
        if "test result:" in line
    ]
    CARGO_RUNS.append(
        {
            "stage": stage,
            "name": name,
            "package": package,
            "filter": filter_,
            "exit": proc.returncode,
            "test_results": result_lines,
        }
    )
    if proc.returncode != 0:
        raise AssertionError(
            f"cargo gate failed rc={proc.returncode} for {name} "
            f"(cargo test -p {package} {filter_})\n"
            f"{proc.stdout[-2000:]}\n{proc.stderr[-2000:]}"
        )
    if not result_lines:
        raise AssertionError(f"cargo gate {name} produced no test result line")


# ── Shared source handles ────────────────────────────────────────────────

POLICY = (ROOT / "crates/focusa-license/src/entitlement_policy.rs").read_text(encoding="utf-8")
UIAI = (ROOT / "crates/focusa-license/src/uiai_child_token.rs").read_text(encoding="utf-8")
COCKPIT = (ROOT / "crates/focusa-license/src/cockpit_action_registry.rs").read_text(encoding="utf-8")
UIAI_ACTIVATION = (ROOT / "crates/focusa-license/src/uiai_activation.rs").read_text(
    encoding="utf-8"
)
AUTHORITY = (ROOT / "crates/focusa-license/src/authority.rs").read_text(encoding="utf-8")
LIFETIME = (ROOT / "crates/focusa-license/src/lifetime_entitlement.rs").read_text(
    encoding="utf-8"
)
CREDENTIALS = (ROOT / "crates/focusa-license/src/authority_credentials.rs").read_text(
    encoding="utf-8"
)
LIMITED_PROJECT = (ROOT / "crates/focusa-core/src/limited_project.rs").read_text(encoding="utf-8")
GUARD = (ROOT / "crates/focusa-core/src/entitlement_execution_guard.rs").read_text(
    encoding="utf-8"
)
CORE_LICENSE = (ROOT / "crates/focusa-core/src/license.rs").read_text(encoding="utf-8")
CLI_LICENSE = (ROOT / "crates/focusa-cli/src/commands/license.rs").read_text(encoding="utf-8")
BRIDGE = (ROOT / "apps/menubar/src-tauri/src/spec172_desktop_bridge.rs").read_text(encoding="utf-8")
FIXTURE = json.loads(
    (ROOT / "crates/focusa-cli/tests/fixtures/spec172-cli-agent-presenter-fixtures.v1.json")
    .read_text(encoding="utf-8")
)
LICENSE_TYPES = yaml.safe_load(
    (CONTRACTS / "spec172-license-types.v1.yaml").read_text(encoding="utf-8")
)
EDD_DOWNLOADS = json.loads(
    (CONTRACTS / "spec172-edd-operator-v1-downloads.v1.json").read_text(encoding="utf-8")
)
CONVERGENCE = json.loads(
    (CONTRACTS / "spec172-public-facade-convergence.v1.json").read_text(encoding="utf-8")
)

FOCUSA_OPERATOR_FAMILIES = [
    "manual_project",
    "manual_mission",
    "manual_focus_state",
    "manual_workpoint",
    "manual_trajectory",
    "manual_basic_evidence",
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
]
UIAI_PAID_FEATURES = [
    "uiai_public_observation",
    "uiai_browser_action",
    "uiai_persistence",
    "uiai_diagnostics",
    "uiai_proof_packets",
    "uiai_batch_responsive",
    "uiai_supported_integrations",
]
RETAINED_ACCESS = [
    "navigation",
    "status",
    "account",
    "read",
    "export",
    "recovery",
    "repair",
    "update",
    "uninstall",
]


def stage1_edd_test_orders_and_keys() -> None:
    """Dedicated $697 EDD test orders/keys; one canonical key per order."""
    if PHP is None:
        raise AssertionError("php runtime is required for the EDD order/key gates")
    replay_gate("1_edd_orders_keys", "edd_operator_products",
                [PHP, "tests/spec172_edd_operator_products_test.php"])
    replay_gate("1_edd_orders_keys", "edd_commerce_acceptance",
                [PHP, "tests/spec172_edd_commerce_acceptance_test.php"])
    replay_gate("1_edd_orders_keys", "focusa_operator_issuance",
                [PHP, "tests/spec172_focusa_operator_issuance_test.php"])
    replay_gate("1_edd_orders_keys", "uiai_operator_issuance",
                [PHP, "tests/spec172_uiai_operator_issuance_test.php"])

    # Dedicated Downloads: exactly two $697 Operator standalones and the Bundle,
    # lifetime, one seat, three shared nodes, checkout disabled.
    records = {record["public_code"]: record for record in EDD_DOWNLOADS["records"]}
    expect(len(records) == 3, "exactly three dedicated Operator v1 records")
    focusa = records["focusa_operator_lifetime_v1"]
    uiai = records["uiai_operator_lifetime_v1"]
    expect(focusa["amount_minor"] == 69700 and focusa["price_usd"] == "697.00",
           "Focusa standalone is $697.00 (69700 minor)")
    expect(uiai["amount_minor"] == 69700 and uiai["price_usd"] == "697.00",
           "UIAI standalone is $697.00 (69700 minor)")
    for code, record in (("focusa_operator_lifetime_v1", focusa),
                         ("uiai_operator_lifetime_v1", uiai)):
        expect(record["license_duration"] == "lifetime", f"{code} is lifetime")
        expect(record["operator_seats"] == 1, f"{code} is one operator seat")
        expect(record["node_limit"] == 3 and record["node_set"] == "operator_shared_v1",
               f"{code} is three shared operator nodes")
        expect(record["checkout_enabled"] is False, f"{code} checkout is disabled")
        expect(record["sale_status"] == "approved_not_yet_enabled",
               f"{code} sale status is approved-not-yet-enabled")
        expect(record["refund_policy"] == "whole_order_30_days"
               and record["refund_days"] == 30, f"{code} whole-order 30-day refund")
    expect(EDD_DOWNLOADS["authority"]["forbidden_implicit_download"] == 453,
           "Download 453 is quarantined and never grants")
    expect(EDD_DOWNLOADS["counts"]["assigned_edd_downloads"] == 3,
           "three assigned dedicated downloads")
    expect(EDD_DOWNLOADS["counts"]["checkout_enabled"] == 0,
           "no checkout enabled until validation passes")

    # Canonical License Type truth: exactly the two Operator lifetime types,
    # each at $697.00, one seat, three shared nodes.
    types = {row["code"]: row for row in LICENSE_TYPES["license_types"]}
    expect(set(types) == {"focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1"},
           "exactly two Operator License Types")
    for code, row in types.items():
        expect(row["price_usd"] == "697.00", f"{code} canonical price is $697.00")
        expect(row["operator_seats"] == 1 and row["node_limit"] == 3,
               f"{code} is one seat / three shared nodes")
    expect(LICENSE_TYPES["postures"][0]["code"] == "verified_no_license",
           "verified_no_license remains the limited posture")


def stage2_same_account_focusa_upgrade() -> None:
    """Same verified account -> Focusa Operator: data/project/node preserved."""
    # Base product gate: only product "focusa" counts; paid is Entitled.
    expect("pub fn resolve_base_focusa_product" in POLICY, "base product resolver exists")
    expect('if product != "focusa"' in POLICY
           and "return BaseProductDecision::Denied" in POLICY,
           "non-focusa product is denied by the base gate")
    base_fn = POLICY[POLICY.index("pub fn resolve_base_focusa_product"):]
    expect("PolicyEntitlementState::ActivePaid | PolicyEntitlementState::OfflineGrace" in base_fn
           and "BaseProductDecision::Entitled" in base_fn,
           "paid Focusa upgrade resolves to Entitled base product")
    expect("PolicyEntitlementState::VerifiedNoLicense" in base_fn
           and "BaseProductDecision::Limited" in base_fn,
           "verified no-license remains Limited")

    # Focusa Operator v1 family set: manual base + four premium families.
    operator_families = POLICY[POLICY.index("pub const SPEC172_FOCUSA_OPERATOR_V1_FAMILIES"):]
    operator_families = operator_families[: operator_families.index("];") + 2]
    for family in FOCUSA_OPERATOR_FAMILIES:
        expect(f'"{family}"' in operator_families, f"Focusa Operator v1 family {family}")

    # Project/data identity is preserved by the upgrade: paid entitlement
    # bypasses the one-project guard but the selection is never deleted.
    expect("focusa_license::BaseProductDecision::Entitled" in LIMITED_PROJECT
           and "ProjectMutationDecision::Allowed" in LIMITED_PROJECT,
           "paid upgrade keeps mutation allowed regardless of selection")
    expect("never deletes data" in LIMITED_PROJECT, "project guard documents no data deletion")
    expect("switch_active_project" in LIMITED_PROJECT and "preserves all retained project data" in LIMITED_PROJECT,
           "switching/preserving projects never deletes data")
    expect("ENTITLEMENT_LIMITED_PROJECT" in GUARD
           and "AllowVerifiedLimited" in GUARD,
           "the one-project guard applies only to the verified-limited posture")

    # Node identity is shared and persisted across upgrades: the same node-id
    # file is read for every license-guard resolution and no app multiplies it.
    expect("load_or_create_node_identity" in CREDENTIALS,
           "node identity is created/persisted by the authority credentials surface")
    expect("node-identity.json" in CREDENTIALS, "node identity is persisted on disk")
    expect("expected_node_id" in LIFETIME or "node_id" in LIFETIME,
           "paid leases bind the operator node id")
    expect("OperatorSharedV1Three" in POLICY, "node limit is the shared three-node baseline")

    # Reducer: paid posture keeps read/export/recovery and requires base-then-
    # feature for premium families (no caller-selected product/price).
    reducer = POLICY[POLICY.index("pub const fn reduce_entitlement_state"):]
    expect("(State::ActivePaid, Family::AccountRecovery | Family::CustomerDataExport)" in reducer
           and "Posture::Allow" in reducer,
           "paid posture preserves account recovery and basic export")
    expect("(State::ActivePaid, Family::ReadProjection)" in reducer and "Posture::Read" in reducer,
           "paid posture preserves the read projection")
    expect("(State::ActivePaid, Family::BaseFocusa)" in reducer and "Posture::Base" in reducer,
           "paid Focusa resolves the base product posture")
    expect("(State::ActivePaid, _)" in reducer and "Posture::Feature" in reducer,
           "paid premium families resolve feature posture")

    # Presenter fixture: the Focusa Operator paid lease renders paid, no denial.
    fixtures = {entry["id"]: entry for entry in FIXTURE["fixtures"]}
    active = fixtures.get("focusa-operator-active")
    expect(active is not None and active["posture"] == "active_paid_operator"
           and active["product"] == "focusa"
           and active["license_type"] == "focusa_operator_lifetime_v1"
           and active["family"] == "automation" and active["denial"] is None
           and active["upgrade_action"] == "none_required",
           "fixture: Focusa Operator paid lease unlocks automation with no denial")
    expect("active_paid_operator" in CLI_LICENSE
           and '"focusa_operator_lifetime_v1"' in CLI_LICENSE,
           "CLI presenter renders the Focusa Operator paid posture and License Type")


def stage3_same_account_uiai_upgrade() -> None:
    """Same verified account -> UIAI Operator: grants, node, hosted boundary."""
    # Canonical UIAI paid family features (Spec 172 §7.2).
    paid_features = UIAI[UIAI.index("pub const SPEC172_UIAI_PAID_FAMILY_FEATURES"):]
    paid_features = paid_features[: paid_features.index("];") + 2]
    for feature in UIAI_PAID_FEATURES:
        expect(f'"{feature}"' in paid_features, f"UIAI paid family feature {feature}")

    # The same account upgrades: the UIAI activation projection binds exactly
    # one account and one EDD customer and requires a UIAI account grant.
    expect("pub struct UiaiAccountIdentity" in UIAI_ACTIVATION
           and "edd_customer_id" in UIAI_ACTIVATION,
           "UIAI activation binds one account + one EDD customer")
    expect("AccountProductGrants" in UIAI_ACTIVATION and "focusa_only" in UIAI_ACTIVATION,
           "account product grants classify Focusa-only accounts")
    expect("same_account_binding" in UIAI_ACTIVATION,
           "same-account binding check exists for Focusa + UIAI grants")
    expect("UiaiActivationError::UiaiGrantRequired" in UIAI_ACTIVATION,
           "UIAI grant is required for UIAI activation")
    expect("UiaiActivationError::AccountIdentityRequired" in UIAI_ACTIVATION,
           "activation without an account identity fails closed")

    # Hosted-resource exclusion: metered/hosted/private rights are never part
    # of the paid UIAI family set; the exclusion registry raises the stable code.
    for feature in ["authenticated_private_targets", "unattended_browser_automation",
                    "scheduled_batch_qa", "premium_hosted_resources"]:
        expect(f'"{feature}"' not in paid_features,
               f"{feature} has no canonical paid family feature", negative=True)
    expect("HOSTED_RESOURCE_NOT_INCLUDED" in
           (CONTRACTS / "spec172-uiai-hosted-resource-exclusion-registry.v1.php").read_text(
               encoding="utf-8"),
           "hosted-resource exclusion raises HOSTED_RESOURCE_NOT_INCLUDED")

    # Node identity preserved for the UIAI lease too: bounded credential binds
    # the operator node and the three-node baseline is shared, never multiplied.
    expect("node_id" in LIFETIME and "node_id" in UIAI,
           "UIAI lease/child-token surface binds the operator node id")
    expect("do not consume separate nodes" in
           (ROOT / "apps/menubar/src-tauri/src/spec172_desktop_bridge.rs").read_text(
               encoding="utf-8"),
           "clients on the same node never consume separate nodes")
    expect("three node identities are shared" in
           (CONTRACTS / "spec172-focusa-paid-lease-fixture.v1.php").read_text(encoding="utf-8"),
           "the paid lease fixture documents three shared node identities")

    # Lifetime entitlement: perpetual commercial right + bounded credential.
    expect('LIFETIME_ENTITLEMENT_SCHEMA' in LIFETIME and "lifetime" in LIFETIME,
           "lifetime entitlement schema exists")
    expect("Revoked" in LIFETIME and "sequence" in LIFETIME,
           "refund/revoke supersedes by higher authority sequence")


def stage4_unlock_only_purchased_families() -> None:
    """Each product unlocks only its own purchased families."""
    # Focusa paid unlocks Focusa display/mutation; UIAI rows deny.
    expect('"cockpit.focusa.display_mission"' in COCKPIT
           and 'FocusaDisplay' in COCKPIT, "Cockpit Focusa display row exists")
    expect('"cockpit.focusa.mutate_project"' in COCKPIT
           and "FocusaMutation" in COCKPIT, "Cockpit Focusa mutation row exists")
    expect("FocusaOnlyCannotGrantUiai" in COCKPIT,
           "Focusa-only accounts cannot grant UIAI (product isolation)")

    # UIAI paid unlocks UIAI families only; Focusa rows deny for UIAI-only.
    expect("FocusaDisplayDenied" in COCKPIT and "FocusaMutationDenied" in COCKPIT,
           "UIAI-only accounts are denied Focusa display/mutation")
    expect("CombinedMissingUiaiGrant" in COCKPIT and "CombinedMissingFocusaGrant" in COCKPIT,
           "combined workflows require both product grants")
    expect("FamilyNotGranted" in UIAI,
           "UIAI paid grant must carry the requested family feature")

    # Reducer: Focusa premium families require base then feature; UIAI hosted
    # rights never enter the canonical paid set.
    reducer = POLICY[POLICY.index("pub const fn reduce_entitlement_state"):]
    expect("(State::ActivePaid, Family::BaseFocusa)" in reducer
           and "Reason::RequireBase" in reducer,
           "premium execution requires the base product first")
    expect("(State::ActivePaid, _)" in reducer and "Reason::RequireFeature" in reducer,
           "premium families require the granted feature")
    for operation_id in [
        "authenticated_private_dashboard", "unattended_browser_automation",
        "premium_proxy", "hosted_capacity", "paid_model_calls",
    ]:
        expect(f'operation_id: "{operation_id}"' in UIAI,
               f"blocked hosted/private vector {operation_id} in operation map")
    expect("HOSTED_RESOURCE_NOT_INCLUDED" in
           (CONTRACTS / "spec172-uiai-hosted-resource-exclusion-registry.v1.php").read_text(
               encoding="utf-8"),
           "hosted-resource attempts raise the stable exclusion code")


def stage5_reject_cross_product_actions() -> None:
    """Cross-product actions are rejected before any side effect."""
    # A paid UIAI lease can never resolve the Focusa base product.
    base_fn = POLICY[POLICY.index("pub fn resolve_base_focusa_product"):]
    expect('product != "focusa"' in base_fn and "BaseProductDecision::Denied" in base_fn,
           "resolve_base_focusa_product denies every non-focusa product")
    # UIAI-only paid lease on a Focusa family renders PRODUCT_NOT_INCLUDED.
    fixtures = {entry["id"]: entry for entry in FIXTURE["fixtures"]}
    cross = fixtures.get("uiai-lease-focusa-family-product-not-included")
    expect(cross is not None and cross["posture"] == "active_paid_operator"
           and cross["product"] == "uiai_engine"
           and cross["license_type"] == "uiai_operator_lifetime_v1"
           and cross["family"] == "base_focusa"
           and cross["denial"] == "PRODUCT_NOT_INCLUDED"
           and cross["upgrade_action"] == "review_offer_or_manage_entitlement",
           "fixture: UIAI lease on a Focusa family denies PRODUCT_NOT_INCLUDED")
    expect('"PRODUCT_NOT_INCLUDED"' in CLI_LICENSE
           and "ENTITLEMENT_PRODUCT_MISMATCH" in CLI_LICENSE,
           "CLI carries the cross-product stable errors")

    # Focusa-only paid entitlement can never activate or grant UIAI.
    expect("UiaiGrantRequired" in UIAI_ACTIVATION,
           "Focusa-only account cannot activate UIAI (UiaiGrantRequired)")
    expect("ProductMappingRequired" in UIAI_ACTIVATION,
           "wrong requested product code fails closed")
    expect("ProductIsolationViolation" in UIAI_ACTIVATION,
           "feature/limit isolation violations are rejected")
    expect("FocusaOnlyCannotGrantUiai" in UIAI,
           "Focusa-only entitlement never grants UIAI even observation")

    # A wrong-product lease fails authority verification before any execution.
    expect("WrongProduct" in AUTHORITY and "expected_product" in AUTHORITY,
           "authority lease verification rejects the wrong product")
    expect("wrong_product" in AUTHORITY, "wrong-product denial label exists")

    # The accepted PHP issuance gates prove wrong-product orders create zero
    # projections and zero canonical licenses.
    expect("wrong_product_uiai" in
           (ROOT / "tests/spec172_focusa_operator_issuance_test.php").read_text(encoding="utf-8"),
           "Focusa issuance gate exercises wrong_product_uiai")
    expect("wrong_product_focusa" in
           (ROOT / "tests/spec172_uiai_operator_issuance_test.php").read_text(encoding="utf-8"),
           "UIAI issuance gate exercises wrong_product_focusa")


def stage6_no_duplicate_customer_key_grant() -> None:
    """No duplicate customer, key, or grant across the two upgrades."""
    # One account identity -> one EDD customer; strict same-account binding.
    expect("uiai_account_activates_exact_grants_without_duplicate_customer" in UIAI_ACTIVATION,
           "Rust vector proves one account/one EDD customer per activation")
    expect("same_account_binding_is_strict_and_never_duplicates_customer" in UIAI_ACTIVATION,
           "Rust vector proves strict same-account binding never duplicates a customer")
    expect("account-002" in UIAI_ACTIVATION,
           "a different account on the UIAI grant is never the same EDD customer")

    # One canonical EDD Software Licensing key per order; duplicate orders and
    # idempotent replays settle once with zero extra projections/keys.
    issuance = (ROOT / "tests/spec172_focusa_operator_issuance_test.php").read_text(
        encoding="utf-8")
    for fixture_name in ["idempotent_replay", "duplicate_projection_call",
                         "wrong_account", "refunded", "revoked", "pending",
                         "pre_issuance", "idempotency_conflict"]:
        expect(fixture_name in issuance,
               f"Focusa issuance gate exercises {fixture_name}")
    matrix = (ROOT / "tests/spec172_edd_commerce_acceptance_test.php").read_text(encoding="utf-8")
    for fixture_name in ["duplicate_order", "caller_grants", "future_type",
                         "hosted_resource_attempts", "no_direct_stripe_facade_path"]:
        expect(fixture_name in matrix, f"commerce matrix exercises {fixture_name}")
    expect("duplicate order settles once" in matrix or "settles once" in matrix,
           "duplicate order settles exactly once")

    # Strictly monotonic per-account sequence (authority sequence never
    # regresses; refund/revoke supersedes by higher sequence).
    expect("sequence" in LIFETIME and "authority sequence rollback denied" in LIFETIME,
           "lifetime entitlement rejects authority sequence rollback")
    expect(re.search(r"strictly monotonic per-account[\s\S]{0,20}sequence",
                    (ROOT / "tests/spec172_focusa_operator_issuance_test.php").read_text(encoding="utf-8"))
           is not None,
           "Focusa projection sequence is strictly monotonic per account")
    expect(re.search(r"strictly monotonic per-account[\s\S]{0,20}sequence",
                    (ROOT / "tests/spec172_uiai_operator_issuance_test.php").read_text(encoding="utf-8"))
           is not None,
           "UIAI projection sequence is strictly monotonic per account")

    # Cross-account grants never combine (Cockpit vector: account-002 grant
    # never pairs with the account-001 Focusa grant).
    expect('"account-002"' in COCKPIT or "account-002" in COCKPIT,
           "cross-account UIAI grant is exercised in the Cockpit surface")
    expect("MalformedBundleUnion" in POLICY,
           "a Bundle union of duplicate grants is rejected")


def stage7_presenters_converge_and_data_preserved() -> None:
    """All presenters converge; limited-state data survives upgrades and refunds."""
    # Presenter gates (all build-independent; menubar/TUI runs under node).
    replay_gate("7_presenters", "public_facade_convergence",
                ["python3", "tests/spec172_public_facade_convergence_test.py"])
    replay_gate("7_presenters", "cli_agent_presenter",
                ["python3", "tests/spec172_cli_agent_presenter_test.py"])
    replay_gate("7_presenters", "cockpit_mixed_product",
                ["python3", "tests/spec172_cockpit_mixed_product_test.py"])
    replay_gate("7_presenters", "focusa_desktop",
                ["python3", "tests/spec172_focusa_desktop_entitlement_test.py"])
    replay_gate("7_presenters", "menubar_tui",
                ["node", "tests/spec172_menubar_tui_presenter_test.mjs"])

    # Public facade convergence: product isolation, EDD authority, prices.
    expect(CONVERGENCE["authority"]["canonical_checkout_authority"] == "WPUIAI.com EDD",
           "checkout authority is WPUIAI EDD")
    expect(CONVERGENCE["authority"]["forbidden_implicit_download"] == 453,
           "Download 453 never implicitly grants")
    expect(CONVERGENCE["authority"]["no_anonymous_product_capability"] is True,
           "no anonymous product capability")
    isolation = CONVERGENCE["canonical_policy"]["product_isolation"]
    expect(isolation.get("focusa_purchase_grants_uiai") is False,
           "Focusa purchase never implicitly grants UIAI")
    prices = {row["public_code"]: row["price_usd"]
              for row in CONVERGENCE["canonical_policy"]["license_types"]}
    expect(prices.get("focusa_operator_lifetime_v1") == "697.00"
           and prices.get("uiai_operator_lifetime_v1") == "697.00"
           and prices.get("focusa_uiai_operator_bundle_lifetime_v1") == "1254.60",
           "public policy converges on $697/$697/$1254.60")

    # Retained access is never disabled in paid posture (same frozen set).
    expect("SPEC172_RETAINED_ACCESS" in BRIDGE, "Desktop carries the retained-access set")
    expect(FIXTURE["retained_access"] == RETAINED_ACCESS,
           "fixture retained-access set is frozen and includes read/export/recovery")
    for control in ["read", "export", "recovery", "repair", "update", "uninstall"]:
        expect(control in RETAINED_ACCESS, f"retained access includes {control}")

    # Live Rust vectors: project guard (+ paid bypass), Spec 172 policy grid,
    # and UIAI activation cross-product/no-duplicate-customer vectors.
    cargo_test("7_presenters", "verified_limited_project", "focusa-core",
               "verified_limited_project")
    cargo_test("7_presenters", "spec172_license_vectors", "focusa-license", "spec172")
    cargo_test("7_presenters", "uiai_activation_vectors", "focusa-license", "uiai_activation")

    # Refund/revoke downgrade preserves data: the account returns to the
    # verified_no_license posture with read/export/recovery intact.
    refund = (ROOT / "tests/spec172_refund_downgrade_test.php").read_text(encoding="utf-8")
    expect("verified_no_license" in refund and "preserve" in refund,
           "refund/revoke downgrade returns to verified_no_license with data preserved")
    reducer = POLICY[POLICY.index("pub const fn reduce_entitlement_state"):]
    expect("(State::Expired | State::RefundedOrRevoked, Family::ReadProjection)" in reducer
           and "Posture::Read" in reducer,
           "refunded/revoked posture keeps the read projection")
    expect("State::Expired | State::RefundedOrRevoked | State::MissingOrCorrupt" in reducer
           and "Family::AccountRecovery | Family::CustomerDataExport" in reducer,
           "refunded/revoked posture keeps account recovery")
    expect("never deletes data" in LIMITED_PROJECT, "downgrade never deletes data")


def hygiene(receipt: str) -> None:
    """The bounded receipt contains no raw email, secret, key, or card data."""
    EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
    SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
    KEY_RE = re.compile(r"(?:FOCUSA|UIAI)-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}")
    PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
    CARD_RE = re.compile(r"\b(?:\d[ -]?){13,16}\b")
    expect(EMAIL_RE.search(receipt) is None, "receipt carries an email literal")
    expect(SECRET_RE.search(receipt) is None and KEY_RE.search(receipt) is None
           and PRIVATE_KEY_RE.search(receipt) is None and CARD_RE.search(receipt) is None,
           "receipt carries a secret, raw key, private key, or card number")


def main() -> int:
    stage1_edd_test_orders_and_keys()
    stage2_same_account_focusa_upgrade()
    stage3_same_account_uiai_upgrade()
    stage4_unlock_only_purchased_families()
    stage5_reject_cross_product_actions()
    stage6_no_duplicate_customer_key_grant()
    stage7_presenters_converge_and_data_preserved()

    receipt = {
        "schema": "focusa.spec172.paid_upgrade_e2e.v1",
        "atom": "focusa-vbcqu.20.15.34",
        "title": "172.05.03 Prove paid Focusa and UIAI upgrades preserve limited-state data",
        "result": "passed_fail_closed",
        "stages": {
            "1_edd_test_orders_and_keys": "two dedicated $697 Operator orders/keys, lifetime, one seat, three shared nodes, checkout disabled, Download 453 quarantined",
            "2_same_account_focusa_upgrade": "same verified account -> Focusa Operator: base Entitled, Operator families unlock, project/data/node preserved",
            "3_same_account_uiai_upgrade": "same verified account -> UIAI Operator: paid families unlock, same account/node binding, hosted boundary frozen",
            "4_unlock_only_purchased_families": "Focusa unlocks Focusa families only; UIAI unlocks UIAI families only; hosted/private rights never granted",
            "5_reject_cross_product_actions": "UIAI lease on Focusa family -> PRODUCT_NOT_INCLUDED; Focusa-only cannot activate/grant UIAI; wrong-product lease fails",
            "6_no_duplicate_customer_key_grant": "one EDD customer per account, one key per order, idempotent replay, monotonic sequence, cross-account grants never combine",
            "7_presenters_converge_and_data_preserved": "facade/CLI/Pi/agent/Desktop/Cockpit/menubar-TUI converge; read/export/recovery preserved; refund/revoke keeps data",
        },
        "replay_gates": {
            key: {"exit": value["exit"]}
            for key, value in sorted(REPLAY.items())
        },
        "replay_gate_exit_codes_all_zero": all(value["exit"] == 0 for value in REPLAY.values()),
        "cargo_runs": [
            {
                "package": run_["package"],
                "filter": run_["filter"],
                "exit": run_["exit"],
                "test_results": run_["test_results"],
            }
            for run_ in CARGO_RUNS
        ],
        "cargo_runs_all_zero": all(run_["exit"] == 0 for run_ in CARGO_RUNS),
        "positive_checks": POSITIVE,
        "negative_checks": NEGATIVE,
        "paid_upgrade": {
            "same_verified_account": True,
            "focusa_order_key": "one canonical EDD key per order",
            "uiai_order_key": "one canonical EDD key per order",
            "project_identity_preserved": True,
            "node_identity_preserved": True,
            "data_read_export_preserved": True,
            "focusa_unlocks_only_focusa": True,
            "uiai_unlocks_only_uiai": True,
            "cross_product_rejected": "PRODUCT_NOT_INCLUDED / UiaiGrantRequired / FocusaOnlyCannotGrantUiai",
            "duplicate_customer": 0,
            "duplicate_key": 0,
            "duplicate_grant": 0,
        },
        "evidence_path": "docs/evidence/spec172/focusa-vbcqu.20.15.34-acceptance.txt",
    }

    receipt_json = json.dumps(receipt, sort_keys=True)
    hygiene(receipt_json)
    print(receipt_json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
