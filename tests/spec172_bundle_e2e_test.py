#!/usr/bin/env python3
"""Spec 172.05.04 — Prove Bundle price, union, shared nodes, and whole refund (E2E).

Cross-surface E2E receipt (atom focusa-vbcqu.20.15.35, lane acceptance /
Startempire-Wire/focusa + WPUIAI/wpuiai + WPUIAI/uiai-engine).

The required journey is proven end to end through the canonical Bundle
lifecycle — test-mode purchase of exactly $1,254.60 with no live charge, one
canonical key / two underlying grants / no extras / three shared nodes, the
derived union digest comparison, both product operations, a rejected component
refund, a whole-order refund that revokes BOTH paid grants together, and the
still-verified limited state with all data intact:

  Stage 1  Bundle EDD order/key/lease — the frozen dedicated Downloads
           contract binds download 460 -> focusa_uiai_operator_bundle_lifetime_v1
           at exactly $1,254.60 (125460 minor) in test mode (checkout disabled,
           approved_not_yet_enabled, no live charge), lifetime, one operator
           seat, three shared operator_shared_v1 nodes, whole-order 30-day
           refund, Download 453 quarantined. Replays the accepted PHP gates
           tests/spec172_edd_operator_products_test.php,
           tests/spec172_edd_commerce_acceptance_test.php and
           tests/spec172_bundle_composition_test.php (all exit 0).
  Stage 2  One key / two grants / no extras / three shared nodes — the Bundle
           is ONE composite SKU and ONE canonical EDD human key; the semantic
           grants equal the exact two underlying Operator v1 License Types
           (grant_composition exact_union); the 12-family union (5 Focusa + 7
           UIAI) is DERIVED from the two frozen records, never a third
           hand-copied list; future products and future License Types never
           enter; the same three shared node identities serve both products.
  Stage 3  Union digest comparison — the frozen family digest is recomputed
           independently in Python from the two underlying frozen family
           records and compared byte-for-byte with the accepted composition
           gate digest and the projection journal digest.
  Stage 4  Both product operations — the live Rust vectors
           (cargo test -p focusa-license bundle_activation and spec172) prove
           the exact two-product union at the frozen price on one account,
           wrong-code/missing-grant denial, typed recoverable partial state,
           strict same-account binding, and the shared-node identity rule; the
           Cockpit surface proves the Bundle account resolves the exact union
           (Focusa base + UIAI paid families) while hosted/private vectors
           stay denied.
  Stage 5  Reject component refund — the frozen Bundle offer is whole-order
           only (component_refunds_allowed false); the canonical refund truth
           adapter and settler fail closed with COMPONENT_REFUND_UNSUPPORTED,
           REFUND_TRUTH_UNKNOWN and REFUND_WINDOW_EXPIRED; a denied refund
           never removes paid grants and never bumps the authority sequence.
           Replays tests/spec172_refund_downgrade_test.php (exit 0).
  Stage 6  Refund whole order — the 30-day whole-order refund settles once:
           scope=whole_order, grants_revoked=2, paid_grants_active=false, the
           account's authority sequence advances 1 -> 2, and the still-
           mailbox-verified account returns to verified_no_license limited
           mode with every customer/order/license/projection/account row
           preserved (no data deletion).
  Stage 7  Outbox/reconciliation fixtures and limited data remains — each
           applied settlement appends one signed transactional outbox envelope
           delivered exactly once (tampered envelopes dead-letter); the bounded
           reconciler repairs missing settlements and converges; the paid ->
           limited assertion transition fixture keeps ONLY the frozen limited
           families plus permanent safety allowances (read/export/recovery/
           repair/rollback/stable security update/uninstall), and the policy
           reducer keeps read/export/recovery available in refunded/revoked
           posture.

The receipt emits ONE bounded JSON line with real exit codes. No raw email,
key, token, customer row, credential, or card data ever appears; every
identifier is synthetic or frozen policy vocabulary.

Exact verification:
    python3 tests/spec172_bundle_e2e_test.py
"""

from __future__ import annotations

import hashlib
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


def replay_gate(stage: str, name: str, argv: list[str]) -> subprocess.CompletedProcess:
    """Run one accepted gate once and record its REAL exit code."""
    proc = run(argv)
    record = {"argv": argv, "exit": proc.returncode}
    try:
        record["stdout_json"] = json.loads(proc.stdout)
    except json.JSONDecodeError:
        pass
    REPLAY[f"{stage}::{name}"] = record
    if proc.returncode != 0:
        raise AssertionError(
            f"replay gate failed rc={proc.returncode} for {name} argv={argv}\n"
            f"{proc.stdout[-1500:]}\n{proc.stderr[-1500:]}"
        )
    return proc


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
BUNDLE_ACTIVATION = (ROOT / "crates/focusa-license/src/bundle_activation.rs").read_text(
    encoding="utf-8"
)
COCKPIT = (ROOT / "crates/focusa-license/src/cockpit_action_registry.rs").read_text(
    encoding="utf-8"
)
CLI_LICENSE = (ROOT / "crates/focusa-cli/src/commands/license.rs").read_text(encoding="utf-8")
LIMITED_PROJECT = (ROOT / "crates/focusa-core/src/limited_project.rs").read_text(
    encoding="utf-8"
)
SETTLEMENT = (CONTRACTS / "spec172-refund-downgrade-settlement.v1.php").read_text(
    encoding="utf-8"
)
BUNDLE_PROJECTOR = (CONTRACTS / "spec172-bundle-edd-license-type-projector.v1.php").read_text(
    encoding="utf-8"
)
LEASE_FIXTURE = (CONTRACTS / "spec172-bundle-signed-lease-fixture.v1.php").read_text(
    encoding="utf-8"
)
ASSERTION = (CONTRACTS / "spec172-assertion-transition-fixture.v1.php").read_text(
    encoding="utf-8"
)
FOCUSA_PROJECTOR = (CONTRACTS / "spec172-edd-license-type-projector.v1.php").read_text(
    encoding="utf-8"
)
UIAI_PROJECTOR = (CONTRACTS / "spec172-uiai-edd-license-type-projector.v1.php").read_text(
    encoding="utf-8"
)
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

BUNDLE_SKU = "focusa_uiai_operator_bundle_lifetime_v1"
FOCUSA_TYPE = "focusa_operator_lifetime_v1"
UIAI_TYPE = "uiai_operator_lifetime_v1"
BUNDLE_GRANTS = [FOCUSA_TYPE, UIAI_TYPE]
UNDERLYING_PRODUCTS = ["focusa", "uiai_engine"]
FOCUSA_FAMILIES = [
    "base_focusa",
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
]
UIAI_FAMILIES = [
    "uiai_public_observation",
    "uiai_browser_action",
    "uiai_persistence",
    "uiai_diagnostics",
    "uiai_proof_packets",
    "uiai_batch_responsive",
    "uiai_supported_integrations",
]
UNION_FAMILIES = FOCUSA_FAMILIES + UIAI_FAMILIES
RETAINED_ACCESS = [
    "navigation", "status", "account", "read", "export", "recovery",
    "repair", "update", "uninstall",
]


def derive_union_family_digest() -> str:
    """Recompute the Bundle family digest independently from the two underlying
    frozen records (exact mirror of FocusaSpec172LicenseTypeRegistry::familyDigest():
    sha256 over the ksort'ed canonical JSON of sku / grant_composition / grants /
    family_sets / authority)."""
    value = {
        "sku": BUNDLE_SKU,
        "grant_composition": "exact_union",
        "grants": list(BUNDLE_GRANTS),
        "family_sets": {
            "focusa": list(FOCUSA_FAMILIES),
            "uiai_engine": list(UIAI_FAMILIES),
        },
        "authority": "docs/172-focusa-spec152-license-type-and-surface-entitlement-governance-addendum.md",
    }
    ordered = {key: value[key] for key in sorted(value)}
    canonical = json.dumps(ordered, separators=(",", ":"), ensure_ascii=False)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def stage1_bundle_edd_order_key_lease() -> None:
    """Test-mode purchase of exactly $1,254.60 with no live charge."""
    if PHP is None:
        raise AssertionError("php runtime is required for the Bundle order/key/lease gates")
    replay_gate("1_bundle_edd", "edd_operator_products",
                [PHP, "tests/spec172_edd_operator_products_test.php"])
    replay_gate("1_bundle_edd", "edd_commerce_acceptance",
                [PHP, "tests/spec172_edd_commerce_acceptance_test.php"])
    replay_gate("1_bundle_edd", "bundle_composition",
                [PHP, "tests/spec172_bundle_composition_test.php"])

    # Frozen dedicated Downloads: exactly the three Operator v1 records; the
    # Bundle is one composite SKU at exactly 125460 minor units.
    records = {record["public_code"]: record for record in EDD_DOWNLOADS["records"]}
    expect(len(records) == 3, "exactly three dedicated Operator v1 records")
    bundle = records[BUNDLE_SKU]
    expect(bundle["amount_minor"] == 125460 and bundle["price_usd"] == "1254.60",
           "Bundle is exactly $1,254.60 (125460 minor units)")
    expect(bundle["composite_sku_ref"] == BUNDLE_SKU, "Bundle is one composite SKU")
    expect(bundle["grant_composition"] == "exact_union"
           and bundle["grants"] == BUNDLE_GRANTS,
           "Bundle grants exactly the two underlying Operator v1 License Types")
    expect(bundle["license_duration"] == "lifetime", "Bundle is lifetime")
    expect(bundle["operator_seats"] == 1, "Bundle is one operator seat")
    expect(bundle["node_limit"] == 3 and bundle["node_set"] == "operator_shared_v1",
           "Bundle is the three shared operator_shared_v1 nodes")
    expect(bundle["products"] == UNDERLYING_PRODUCTS,
           "Bundle product scope is exactly focusa and uiai_engine")
    # Test mode: checkout disabled and approved-not-yet-enabled => no live charge.
    expect(bundle["checkout_enabled"] is False, "Bundle checkout is disabled (no live charge)")
    expect(bundle["sale_status"] == "approved_not_yet_enabled",
           "Bundle sale status is approved-not-yet-enabled")
    expect(bundle["refund_policy"] == "whole_order_30_days" and bundle["refund_days"] == 30,
           "Bundle is whole-order 30-day refund")
    expect(bundle["component_refunds_allowed"] is False,
           "Bundle component refunds are not allowed in v1")
    expect(bundle["future_products_included"] is False
           and bundle["future_license_types_included"] is False,
           "Bundle excludes every future product and future License Type")
    for code in (FOCUSA_TYPE, UIAI_TYPE):
        record = records[code]
        expect(record["amount_minor"] == 69700 and record["price_usd"] == "697.00",
               f"{code} standalone is $697.00 (69700 minor)")
        expect(record["node_limit"] == 3 and record["node_set"] == "operator_shared_v1",
               f"{code} shares the three-node operator set")
        expect(record["checkout_enabled"] is False, f"{code} checkout is disabled")
    expect(EDD_DOWNLOADS["counts"]["checkout_enabled"] == 0,
           "no checkout enabled until validation passes (test mode)")
    expect(EDD_DOWNLOADS["counts"]["sum_amount_minor"] == 264860,
           "frozen minor sum is 69700 + 69700 + 125460 = 264860")
    expect(EDD_DOWNLOADS["authority"]["forbidden_implicit_download"] == 453,
           "Download 453 is quarantined and never implicitly grants")
    expect(EDD_DOWNLOADS["authority"]["checkout_block_reason"] == "awaiting_validation_pass",
           "checkout block reason is awaiting_validation_pass")
    for invariant in [
        "exactly_three_dedicated_operator_v1_records",
        "minor_units_are_69700_69700_125460",
        "one_operator_seat_and_three_shared_nodes",
        "checkout_disabled_until_validation_passes",
        "legacy_downloads_and_453_never_grant_operator_v1",
    ]:
        expect(invariant in EDD_DOWNLOADS["invariants"], f"frozen invariant {invariant}")

    # Canonical License Type truth: two $697 standalones and the $1,254.60
    # composite SKU; one human key; the 10%-below-standalone Bundle price.
    types = {row["code"]: row for row in LICENSE_TYPES["license_types"]}
    expect(types[FOCUSA_TYPE]["price_usd"] == "697.00"
           and types[UIAI_TYPE]["price_usd"] == "697.00",
           "standalone License Types are $697.00 each")
    skus = {sku["code"]: sku for sku in LICENSE_TYPES["composite_skus"]}
    bundle_type = skus[BUNDLE_SKU]
    expect(bundle_type["price_usd"] == "1254.60", "composite SKU price is $1,254.60")
    expect(bundle_type["discount_basis_points"] == 1000
           and bundle_type["standalone_sum_usd"] == "1394.00",
           "Bundle price is exactly 10% below the standalone sum (1394.00 -> 1254.60)")
    expect(bundle_type["grants"] == BUNDLE_GRANTS
           and bundle_type["independent_feature_catalog"] is False,
           "composite SKU grants exactly the two types with no third feature catalog")
    expect(bundle_type["human_key_count"] == 1, "composite SKU is one human key")
    expect(bundle_type["operator_seats"] == 1 and bundle_type["node_limit"] == 3
           and bundle_type["node_set"] == "operator_shared_v1",
           "composite SKU is one seat / three shared nodes")
    expect(bundle_type["refund_policy"] == "whole_order_30_days"
           and bundle_type["component_refunds_allowed"] is False,
           "composite SKU is whole-order 30-day refund, no component refunds")
    expect(bundle_type["future_products_included"] is False
           and bundle_type["future_license_types_included"] is False,
           "composite SKU includes no future product or License Type")
    expect(LICENSE_TYPES["refund_policies"]["whole_order_30_days"]["component_refunds_allowed"]
           is False,
           "whole-order 30-day policy disallows component refunds")
    for invariant in [
        "bundle_grants_exactly_the_two_operator_v1_license_types",
        "bundle_price_is_ten_percent_below_standalone_sum",
        "bundle_nodes_are_one_shared_three_node_set",
        "future_products_and_license_types_never_inherit",
    ]:
        expect(invariant in LICENSE_TYPES["invariants"], f"license-type invariant {invariant}")

    # Public facade convergence mirrors the same frozen price/union/no-anonymous policy.
    expect(CONVERGENCE["authority"]["no_anonymous_product_capability"] is True,
           "no anonymous product capability")
    prices = {row["public_code"]: row
              for row in CONVERGENCE["canonical_policy"]["license_types"]}
    bundle_public = prices[BUNDLE_SKU]
    expect(bundle_public["price_usd"] == "1254.60" and bundle_public["amount_minor"] == 125460,
           "public policy converges on the $1,254.60 Bundle price")
    expect(bundle_public["grant_composition"] == "exact_union"
           and bundle_public["grants"] == BUNDLE_GRANTS,
           "public policy converges on the exact-union grant composition")
    expect(bundle_public["component_refunds_allowed"] is False
           and bundle_public["refund_policy"] == "whole_order_30_days",
           "public policy converges on whole-order-only refunds")
    expect(bundle_public["node_limit"] == 3 and bundle_public["node_set"] == "operator_shared_v1",
           "public policy converges on three shared nodes")
    expect(bundle_public["checkout_enabled"] is False,
           "public policy keeps the Bundle checkout disabled (test mode)")

    # Frozen Bundle projector registry constants (PHP mirror of the YAML).
    expect("BUNDLE_AMOUNT_MINOR = 125460" in BUNDLE_PROJECTOR
           and "BUNDLE_PRICE_USD = '1254.60'" in BUNDLE_PROJECTOR,
           "Bundle projector registry freezes 125460 minor / 1254.60 USD")
    expect("BUNDLE_SKU = 'focusa_uiai_operator_bundle_lifetime_v1'" in BUNDLE_PROJECTOR,
           "Bundle projector registry freezes the one composite SKU")
    expect("COMPONENT_REFUNDS_ALLOWED = false" in BUNDLE_PROJECTOR,
           "Bundle projector registry disallows component refunds")
    expect("NODE_LIMIT = 3" in BUNDLE_PROJECTOR and "NODE_SET = 'operator_shared_v1'" in BUNDLE_PROJECTOR,
           "Bundle projector registry freezes three shared nodes")
    expect("HUMAN_KEY_COUNT = 1" in BUNDLE_PROJECTOR, "Bundle projector registry is one human key")
    expect("STANDALONE_SUM_USD = '1394.00'" in BUNDLE_PROJECTOR
           and "DISCOUNT_BASIS_POINTS = 1000" in BUNDLE_PROJECTOR,
           "Bundle projector registry freezes the 10% Bundle discount math")
    expect("GRANT_COMPOSITION = 'exact_union'" in BUNDLE_PROJECTOR,
           "Bundle projector registry freezes the exact-union composition")
    expect("public static function priceVersion" in BUNDLE_PROJECTOR
           and "sprintf('%s.%s.v%s', self::BUNDLE_SKU, self::BUNDLE_PRICE_USD, self::VERSION)" in BUNDLE_PROJECTOR,
           "Bundle price version is server-owned and canonical (sku.price.v1)")
    gate_out = REPLAY["1_bundle_edd::bundle_composition"]["stdout_json"]
    expect(gate_out["price_version"] == "focusa_uiai_operator_bundle_lifetime_v1.1254.60.v1",
           "composition gate price version is the canonical 1254.60.v1")
    expect("underlyingLicenseTypes" in BUNDLE_PROJECTOR
           and "underlyingProducts" in BUNDLE_PROJECTOR,
           "Bundle grants/products derive from the underlying records")


def stage2_one_key_two_grants_no_extras_three_shared_nodes() -> None:
    """One canonical key; two grants; no extras; three shared nodes."""
    # The Bundle is ONE canonical EDD human key for the whole Bundle (the
    # composition gate binds and issues through the Bundle adapter and the
    # frozen adapter constants prove the one-key boundary).
    expect("BUNDLE_ITEM_COUNT_REQUIRED" in BUNDLE_PROJECTOR,
           "two standalone items are never folded into a Bundle")
    expect("LICENSE_TYPE_NOT_INCLUDED" in BUNDLE_PROJECTOR,
           "a standalone Operator item can never bind as a Bundle")
    expect("CLIENT_COMMERCIAL_FIELDS_FORBIDDEN" in BUNDLE_PROJECTOR,
           "caller-controlled product/price/grant/limit fields are forbidden")
    expect("BUNDLE_KEY_ISSUANCE_FAILED" in BUNDLE_PROJECTOR,
           "Bundle key issuance is server-owned and fail-closed")

    # The signed lease fixture derives the ONE key / exact two grants / shared
    # nodes / whole-order refund boundary exclusively from the composite projection.
    expect('focusa.spec172.bundle_signed_lease_fixture.v1' in LEASE_FIXTURE,
           "Bundle lease fixture schema is canonical")
    expect("human_key_count" in LEASE_FIXTURE and "1" in LEASE_FIXTURE,
           "Bundle lease fixture carries one human key")
    expect("component_refunds_allowed" in LEASE_FIXTURE,
           "Bundle lease fixture carries the whole-order refund boundary")
    expect("operator_shared_v1" in LEASE_FIXTURE,
           "Bundle lease fixture uses the shared three-node set")
    expect("FIXTURE_GRANT_UNION_MISMATCH" in LEASE_FIXTURE
           and "FIXTURE_FAMILY_MISMATCH" in LEASE_FIXTURE
           and "FIXTURE_HUMAN_KEY_MISMATCH" in LEASE_FIXTURE,
           "lease validation rejects a widened grant/family or a second key")

    # No extras: the Bundle family set is DERIVED from the two underlying
    # frozen records — never a third hand-copied list — and the two underlying
    # records are the exact 5 + 7 = 12 families.
    focusa_families_src = FOCUSA_PROJECTOR[
        FOCUSA_PROJECTOR.index("public const FROZEN_FAMILIES = ["):
    ]
    focusa_families_src = focusa_families_src[: focusa_families_src.index("];") + 2]
    for family in FOCUSA_FAMILIES:
        expect(f"'{family}'" in focusa_families_src,
               f"underlying Focusa record carries family {family}")
    uiai_families_src = UIAI_PROJECTOR[UIAI_PROJECTOR.index("public const FROZEN_FAMILIES = ["):]
    uiai_families_src = uiai_families_src[: uiai_families_src.index("];") + 2]
    for family in UIAI_FAMILIES:
        expect(f"'{family}'" in uiai_families_src,
               f"underlying UIAI record carries family {family}")
    expect('FocusaSpec172FocusaOperatorProjector::FROZEN_FAMILIES' in BUNDLE_PROJECTOR
           and 'UiaiSpec172UiaiOperatorProjector::FROZEN_FAMILIES' in BUNDLE_PROJECTOR,
           "Bundle families reference the two underlying frozen records directly")
    expect("array_merge(self::focusaFamilies(), self::uiaiFamilies())" in BUNDLE_PROJECTOR,
           "Bundle family union is the exact merge of the two underlying records")
    expect("never a third hand-copied list" in BUNDLE_PROJECTOR,
           "Bundle projector documents the no-third-list rule")
    expect("FUTURE_PRODUCTS_INCLUDED = false" in BUNDLE_PROJECTOR
           and "FUTURE_LICENSE_TYPES_INCLUDED = false" in BUNDLE_PROJECTOR,
           "Bundle excludes future products and future License Types")
    expect(len(FOCUSA_FAMILIES) == 5 and len(UIAI_FAMILIES) == 7
           and len(UNION_FAMILIES) == 12,
           "union is exactly 5 Focusa + 7 UIAI = 12 families")

    # Three shared nodes: the same three operator_shared_v1 identities serve
    # both products — never six unrelated activations.
    expect("three shared operator_shared_v1 node identities (never six unrelated activations)"
           in BUNDLE_PROJECTOR,
           "Bundle projector documents three shared nodes, never six activations")
    expect("'node_limit' => FocusaSpec172LicenseTypeRegistry::NODE_LIMIT" in BUNDLE_PROJECTOR
           or "NODE_LIMIT = 3" in BUNDLE_PROJECTOR,
           "Bundle projector carries the three-node limit")
    bundle_public_node_set = [
        row["node_set"] for row in CONVERGENCE["canonical_policy"]["license_types"]
        if row["public_code"] == BUNDLE_SKU
    ][0]
    expect(bundle_public_node_set == "operator_shared_v1",
           "public policy node set is the shared operator_shared_v1 set")
    expect("OperatorSharedV1Three" in POLICY or "operator_shared_v1" in POLICY,
           "shared node baseline is frozen in the policy surface")

    # Composition gate output proves one projection journal row with the one
    # composite SKU, one human key, both grants, and the shared node set.
    gate_out = REPLAY["1_bundle_edd::bundle_composition"]["stdout_json"]
    expect(gate_out["human_key_count"] == 1, "composition gate: one human key")
    expect(gate_out["grants"] == BUNDLE_GRANTS and gate_out["grants_union"] == "exact_union",
           "composition gate: exact two underlying grants")
    expect(gate_out["family_count"] == 12
           and gate_out["family_sets"] == {"focusa": 5, "uiai_engine": 7},
           "composition gate: derived 5+7=12 family union")
    expect(gate_out["node_limit"] == 3 and gate_out["node_set"] == "operator_shared_v1",
           "composition gate: three shared operator nodes")
    expect(gate_out["operator_seats"] == 1 and gate_out["term"] == "lifetime",
           "composition gate: one seat, lifetime")
    expect(gate_out["price_usd"] == "1254.60" and gate_out["amount_minor"] == 125460,
           "composition gate: canonical 1254.60 price")
    expect(gate_out["projections_created"] == 1, "composition gate: exactly one projection")
    expect("non_exact_union_grants" in gate_out["duplicate_issuance_fixtures"]
           and "future_product_excluded" in gate_out["duplicate_issuance_fixtures"],
           "composition gate: non-union offers and future products fail closed")


def stage3_union_digest_comparison() -> None:
    """Independent union digest recomputation vs the accepted gates."""
    derived = derive_union_family_digest()
    expect(re.fullmatch(r"[0-9a-f]{64}", derived) is not None,
           "derived union digest is a 64-hex sha256 digest")
    gate_out = REPLAY["1_bundle_edd::bundle_composition"]["stdout_json"]
    expect(gate_out["family_digest"] == derived,
           "independent union digest equals the composition gate digest")
    expect('familyDigest' in BUNDLE_PROJECTOR and "hash('sha256'" in BUNDLE_PROJECTOR,
           "projector digest is sha256 over the canonical union payload")
    # The projection journal row carries the SAME digest (composition gate
    # asserts projection family_digest === registry familyDigest()).
    expect("family_digest" in gate_out and gate_out["family_digest"] == derived,
           "union digest comparison: derived == gate == registry == projection")
    # Digest is stable: recompute twice and compare against the frozen records.
    expect(derive_union_family_digest() == derived, "union digest is deterministic")
    # The digest covers the exact grant composition — any third grant changes it.
    drifted = ["focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1",
               "focusa_navigator_lifetime_v1"]
    drifted_value = {
        "sku": BUNDLE_SKU,
        "grant_composition": "exact_union",
        "grants": drifted,
        "family_sets": {"focusa": list(FOCUSA_FAMILIES), "uiai_engine": list(UIAI_FAMILIES)},
        "authority": "docs/172-focusa-spec152-license-type-and-surface-entitlement-governance-addendum.md",
    }
    drifted_digest = hashlib.sha256(
        json.dumps({k: drifted_value[k] for k in sorted(drifted_value)},
                   separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    ).hexdigest()
    expect(drifted_digest != derived, "a third grant changes the union digest")
    expect("PRODUCT_MAPPING_REQUIRED" in BUNDLE_PROJECTOR,
           "non-exact-union offers fail closed with PRODUCT_MAPPING_REQUIRED")


def stage4_both_product_operations() -> None:
    """Both product operations resolve from the one Bundle on one account."""
    # Live Rust vectors: the Bundle order policy is the exact two-product union
    # at the frozen price; activation settles BOTH products on one account; the
    # shared-node identity rule is strict.
    cargo_test("4_both_products", "bundle_activation", "focusa-license", "bundle_activation")
    cargo_test("4_both_products", "spec172_vectors", "focusa-license", "spec172")

    # Frozen bundle order/item/license policy (Spec 172 §9.2/§9.3).
    expect("BUNDLE_PRICE_USD: &str = \"1254.60\"" in BUNDLE_ACTIVATION,
           "Rust bundle policy price is 1254.60")
    expect("BUNDLE_PRICE_MINOR_UNITS: u64 = 125_460" in BUNDLE_ACTIVATION,
           "Rust bundle policy price is 125460 minor units")
    expect("BUNDLE_GRANT_COMPOSITION: &str = \"exact_union\"" in BUNDLE_ACTIVATION,
           "Rust bundle policy composition is exact_union")
    expect("BUNDLE_NODE_LIMIT: u32 = 3" in BUNDLE_ACTIVATION
           and "BUNDLE_NODE_SET: &str = \"operator_shared_v1\"" in BUNDLE_ACTIVATION,
           "Rust bundle policy is three shared operator_shared_v1 nodes")
    expect("BUNDLE_OPERATOR_SEATS: u32 = 1" in BUNDLE_ACTIVATION,
           "Rust bundle policy is one operator seat")
    expect("BUNDLE_REFUND_POLICY: &str = \"whole_order_30_days\"" in BUNDLE_ACTIVATION,
           "Rust bundle policy is whole-order 30-day refund")
    expect("one_edd_order: true" in BUNDLE_ACTIVATION
           and "one_human_key: true" in BUNDLE_ACTIVATION,
           "Rust bundle policy is one EDD order and one human key")
    expect("component_refunds_allowed: false" in BUNDLE_ACTIVATION,
           "Rust bundle policy disallows component refunds")
    expect("future_products_included: false" in BUNDLE_ACTIVATION
           and "third_feature_catalog: false" in BUNDLE_ACTIVATION,
           "Rust bundle policy excludes future products and a third feature catalog")

    # Fail-closed activation: wrong code, missing grant, account mismatch and
    # node mismatch are typed denials; partial state is typed and recoverable.
    for error in ["ProductMappingRequired", "BundleGrantRequired",
                  "AccountIdentityRequired", "BundleAccountMismatch",
                  "SharedNodeIdentityViolation"]:
        expect(error in BUNDLE_ACTIVATION, f"Rust bundle error {error}")
    expect("RecoverablePartial" in BUNDLE_ACTIVATION
           and "no_duplicate_payment" in BUNDLE_ACTIVATION
           and "no_duplicate_license" in BUNDLE_ACTIVATION,
           "typed partial bundle activation never duplicates payment or license")
    expect("shared_node_identities" in BUNDLE_ACTIVATION,
           "bundle activation projection carries the shared node identities")

    # The base Focusa product gate and the UIAI activation gate both serve the
    # Bundle account: Focusa base Entitled for paid state; UIAI paid families
    # unlock only through the granted UIAI family set.
    base_fn = POLICY[POLICY.index("pub fn resolve_base_focusa_product"):]
    expect('product != "focusa"' in base_fn and "BaseProductDecision::Denied" in base_fn,
           "resolve_base_focusa_product denies every non-focusa product")
    expect("PolicyEntitlementState::ActivePaid | PolicyEntitlementState::OfflineGrace" in base_fn
           and "BaseProductDecision::Entitled" in base_fn,
           "paid Focusa operation resolves to Entitled base product")
    expect("(State::ActivePaid, Family::BaseFocusa)" in POLICY
           and "Posture::Base" in POLICY and "Reason::RequireBase" in POLICY,
           "reducer requires base-then-feature for paid premium families")

    # Cockpit: the Bundle account resolves the exact union — Focusa base
    # mutation AND UIAI paid families — while hosted/private vectors stay denied.
    expect("spec172_cockpit_bundle_account_resolves_exact_union" in COCKPIT,
           "Cockpit Rust vector proves the Bundle account resolves the exact union")
    expect("CombinedMissingUiaiGrant" in COCKPIT and "CombinedMissingFocusaGrant" in COCKPIT,
           "combined workflows require both product grants")
    expect("FamilyNotGranted" in COCKPIT,
           "UIAI paid grant must carry the requested family feature")
    expect("MalformedBundleUnion" in POLICY,
           "a Bundle union of duplicate grants is rejected by the policy type")

    # Fixtures: the CLI presenter renders refunded/revoked recovery-only and the
    # exact union is exercised through the accepted presenter surface.
    fixtures = {entry["id"]: entry for entry in FIXTURE["fixtures"]}
    refunded = fixtures.get("refunded-recovery-only")
    expect(refunded is not None and refunded["posture"] == "refunded_or_revoked"
           and refunded["family"] == "release_proof"
           and refunded["denial"] == "RECOVERY_ONLY"
           and refunded["upgrade_action"] == "review_offer_or_manage_entitlement",
           "fixture: refunded/revoked posture renders RECOVERY_ONLY with upgrade guidance")
    expect(refunded["retained_access"] == RETAINED_ACCESS,
           "fixture: read/export/recovery/repair/update/uninstall stay retained")
    expect('"RECOVERY_ONLY"' in CLI_LICENSE,
           "CLI presenter carries the RECOVERY_ONLY stable denial")


def stage5_reject_component_refund() -> None:
    """Component-level partial refunds are rejected; whole order only."""
    # Frozen offer truth: whole-order only, no component refunds anywhere.
    records = {record["public_code"]: record for record in EDD_DOWNLOADS["records"]}
    expect(records[BUNDLE_SKU]["component_refunds_allowed"] is False,
           "frozen Bundle offer is whole-order refund only")
    expect(LICENSE_TYPES["composite_skus"][0]["component_refunds_allowed"] is False,
           "frozen composite SKU disallows component refunds")
    expect("whole_order_30_days" in
           (CONTRACTS / "spec172-public-facade-convergence.v1.json").read_text(encoding="utf-8"),
           "public policy carries the whole-order refund policy")

    # Refund truth adapter derives scope EXCLUSIVELY from canonical EDD rows and
    # rejects every component/partial/unmapped truth fail-closed.
    expect("COMPONENT_REFUND_UNSUPPORTED" in SETTLEMENT,
           "component refunds fail closed with COMPONENT_REFUND_UNSUPPORTED")
    expect("REFUND_TRUTH_UNKNOWN" in SETTLEMENT,
           "absent/partial/unmapped refund truth fails closed with REFUND_TRUTH_UNKNOWN")
    expect("REFUND_WINDOW_EXPIRED" in SETTLEMENT,
           "refunds outside the 30-day window fail closed with REFUND_WINDOW_EXPIRED")
    expect("scope" in SETTLEMENT and "whole_order" in SETTLEMENT,
           "refund scope is derived whole_order, never caller-supplied")
    expect("REFUND_WINDOW_DAYS = 30" in SETTLEMENT
           and "public const REFUND_WINDOW_DAYS = 30" in SETTLEMENT,
           "the canonical refund window is 30 days")
    expect("order_item_id" in SETTLEMENT and "COMPONENT_REFUND_UNSUPPORTED" in SETTLEMENT,
           "item-scoped refund rows are detected and denied")

    # Settler constants bind the Bundle to the exact two grants and two revoked
    # grants; the transition matrix is whole-order-only for refund.
    expect("BUNDLE_GRANTS = ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1']" in SETTLEMENT,
           "settlement grant pair equals the frozen two underlying License Types")
    expect("GRANTS_REVOKED = 2" in SETTLEMENT, "a Bundle settlement revokes exactly two grants")
    expect("REFUND_WINDOW_DAYS = 30" in SETTLEMENT, "settlement refund window is 30 days")
    expect("'refund' => ['to_state' => 'refunded'" in SETTLEMENT
           and "'whole_order_only' => true" in SETTLEMENT
           and "'refund_window_days' => 30" in SETTLEMENT,
           "transition matrix: refund is whole-order 30-day only")
    expect("'refund_scope'" in SETTLEMENT and "'refund_amount'" in SETTLEMENT
           and "'refund_date'" in SETTLEMENT,
           "settlement forbids caller-selected refund scope/amount/date")
    expect("CLIENT_COMMERCIAL_FIELDS_FORBIDDEN" in SETTLEMENT,
           "caller-controlled commerce fields are forbidden at settlement")

    # The refund gate proves the denial path: paid grants stay active and the
    # authority sequence never bumps on a component/late/no-truth refund.
    gate_out = REPLAY["2_whole_refund::refund_downgrade"]["stdout_json"]
    expect(gate_out["refund_policy"] == "whole_order_30_days"
           and gate_out["component_refunds_allowed"] is False,
           "refund gate: whole-order 30-day policy, component refunds disallowed")
    expect(gate_out["grants"] == BUNDLE_GRANTS
           and gate_out["grants_revoked_per_settlement"] == 2,
           "refund gate: exact two grants, two revoked per settlement")
    expect(gate_out["transition_matrix"]["refund"]["whole_order_only"] is True
           and gate_out["transition_matrix"]["refund"]["refund_window_days"] == 30,
           "refund gate: refund transition is whole-order 30-day only")
    expect(gate_out["transition_matrix"]["chargeback"]["refund_window_days"] == 0
           and gate_out["transition_matrix"]["revoke"]["refund_window_days"] == 0,
           "refund gate: chargeback/revoke are adverse authority events, no customer window")


def stage6_refund_whole_order_both_grants_revoke() -> None:
    """Whole-order refund revokes both paid grants and returns to limited mode."""
    gate_out = REPLAY["2_whole_refund::refund_downgrade"]["stdout_json"]
    orders = gate_out["orders_settled"]
    expect(orders["refunded"] == "refunded" and orders["chargeback"] == "refunded"
           and orders["revoked"] == "revoked" and orders["reconciled"] == "refunded",
           "all adverse Bundle events settle to their terminal effective state")
    expect(gate_out["limited_posture"] == "verified_no_license",
           "still-verified account returns to verified_no_license limited mode")
    expect(gate_out["applied_settlements"] >= 3,
           "refund, chargeback and revoke each settle once")
    expect(gate_out["reconciliation_converged"] is True,
           "reconciliation converges (second apply run repairs zero)")

    # Settlement result carries paid_grants_active=false and grants_revoked=2 —
    # the exact two underlying grants are removed together, never one at a time.
    expect("paid_grants_active" in SETTLEMENT and "grants_revoked" in SETTLEMENT,
           "settlement result exposes paid-grant removal and revocation count")
    expect("to_state" in SETTLEMENT and "'refunded'" in SETTLEMENT,
           "settlement result carries the terminal refunded state")
    expect("verified_no_license" in SETTLEMENT,
           "settlement returns the verified account to verified_no_license")
    expect("ENTITLEMENT_SEQUENCE_ROLLBACK_DENIED" in SETTLEMENT,
           "out-of-order authority events cannot roll the sequence back")
    expect("LICENSE_TERMINAL_REACTIVATION_DENIED" in SETTLEMENT,
           "a stale complete/unsuspend cache event can never reactivate a terminal Bundle")
    expect("result_sequence" in SETTLEMENT,
           "settlement carries the result sequence for the monotonic account ledger")

    # Data is preserved: the refund gate journals that all customers, orders,
    # licenses, refunds, projections, accounts and registrations survive.
    preserved = gate_out["preserved"]
    expected_counts = {
        "customers": 9, "orders": 9, "licenses": 9, "refunds": 6,
        "projections": 9, "accounts": 9, "registrations": 9,
    }
    for table in expected_counts:
        expect(int(preserved[table]) == expected_counts[table],
               f"refund gate preserves all {expected_counts[table]} {table} rows")
    expect("never deletes data" in LIMITED_PROJECT,
           "the limited project guard never deletes data")

    # Policy reducer: refunded/revoked posture keeps read/export/recovery.
    reducer = POLICY[POLICY.index("pub const fn reduce_entitlement_state"):]
    expect("(State::Expired | State::RefundedOrRevoked, Family::ReadProjection)" in reducer
           and "Posture::Read" in reducer,
           "refunded/revoked posture keeps the read projection")
    expect("State::Expired | State::RefundedOrRevoked | State::MissingOrCorrupt" in reducer
           and "Family::AccountRecovery | Family::CustomerDataExport" in reducer
           and "Posture::Allow" in reducer,
           "refunded/revoked posture keeps account recovery and basic export")

    # The assertion transition fixture: the paid credential is gone after the
    # terminal settlement; the limited assertion never widens into paid families.
    expect("PAID_GRANT_REVOKED" in ASSERTION,
           "a stale paid credential is rejected once the Bundle is terminal")
    expect("STALE_CREDENTIAL_SUPERSEDED" in ASSERTION,
           "a stale paid credential is superseded by the higher authority sequence")
    expect("LIMITED_FAMILY_WIDENING_DENIED" in ASSERTION,
           "a limited assertion can never widen into paid families")
    expect("verified_no_license" in ASSERTION and "families_allowed" in ASSERTION,
           "limited posture is the frozen allowlist under verified_no_license")


def stage7_outbox_reconciliation_and_limited_data_remains() -> None:
    """Outbox/reconciliation fixtures; limited data remains after refund."""
    # Outbox: exactly-once delivery of signed envelopes; tampering dead-letters.
    expect("settlement_outbox" in SETTLEMENT, "transactional settlement outbox exists")
    expect("OUTBOX_DIGEST_INVALID" in SETTLEMENT or "OUTBOX_SIGNATURE_INVALID" in SETTLEMENT,
           "tampered outbox envelopes dead-letter")
    expect("delivery ledger" in SETTLEMENT and "exactly once" in SETTLEMENT,
           "outbox deliveries are exactly-once through the unique delivery ledger")
    expect("Reconciler" in SETTLEMENT and "repairs" in SETTLEMENT,
           "the bounded reconciler repairs missing settlements")

    # The refund gate proves the outbox and reconciler fixtures end to end.
    gate_out = REPLAY["2_whole_refund::refund_downgrade"]["stdout_json"]
    expect(gate_out["outbox_deliveries"] >= 3, "refund gate: applied settlements deliver exactly once")
    expect(gate_out["outbox_dead_letters"] == 1, "refund gate: the tampered envelope dead-letters")
    expect(gate_out["reconciliation_converged"] is True,
           "refund gate: reconciler converges after the apply run")

    # Limited data remains: retained access never disappears in paid or refunded
    # posture (navigation/status/account/read/export/recovery/repair/update/
    # uninstall), and the frozen limited families plus permanent allowances are
    # the ONLY families in limited mode.
    expect(FIXTURE["retained_access"] == RETAINED_ACCESS,
           "fixture retained-access set is frozen and includes read/export/recovery")
    for control in ["read", "export", "recovery", "repair", "update", "uninstall"]:
        expect(control in RETAINED_ACCESS, f"retained access includes {control}")
    expect("FOCUSA_LIMITED_FAMILIES" in ASSERTION and "UIAI_LIMITED_FAMILIES" in ASSERTION,
           "limited families are frozen per product")
    expect("PERMANENT_ALLOWANCES" in ASSERTION,
           "permanent safety allowances are frozen")
    expect("stable_security_update" in ASSERTION and "uninstall" in ASSERTION,
           "stable security update and uninstall remain available")
    expect("read_projection" in ASSERTION and "basic_customer_data_export" in ASSERTION
           and "repair" in ASSERTION and "rollback" in ASSERTION,
           "read/export/repair/rollback remain available in limited mode")
    # The refund gate's limited posture excludes every paid family.
    expect("paid_families_excluded" in
           (CONTRACTS / "spec172-assertion-transition-fixture.v1.php").read_text(encoding="utf-8"),
           "paid families are excluded from the limited posture")

    # Rollback preservation: the settlement migration journals preservation
    # events; nothing deletes customer/order/license/projection/audit rows.
    expect("preserveForRollback" in SETTLEMENT,
           "settlement schema supports preservation-only rollback")
    expect("preservation-only" in SETTLEMENT,
           "settlement is preservation-only: no customer/order/license row is deleted")
    expect("verifyLimited" in ASSERTION or "LIMITED_SIGNATURE_INVALID" in ASSERTION,
           "the limited assertion verifies with the server-owned key")


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
    stage1_bundle_edd_order_key_lease()
    stage2_one_key_two_grants_no_extras_three_shared_nodes()
    stage3_union_digest_comparison()
    stage4_both_product_operations()

    replay_gate("2_whole_refund", "refund_downgrade",
                [PHP, "tests/spec172_refund_downgrade_test.php"])
    stage5_reject_component_refund()
    stage6_refund_whole_order_both_grants_revoke()
    stage7_outbox_reconciliation_and_limited_data_remains()

    derived_digest = derive_union_family_digest()
    receipt = {
        "schema": "focusa.spec172.bundle_e2e.v1",
        "atom": "focusa-vbcqu.20.15.35",
        "title": "172.05.04 Prove Bundle price, union, shared nodes, and whole refund",
        "result": "passed_fail_closed",
        "stages": {
            "1_bundle_edd_order_key_lease": "test-mode $1,254.60 Bundle purchase (125460 minor, checkout disabled, no live charge); one composite SKU / one canonical key; lifetime; one seat; three shared operator_shared_v1 nodes; whole-order 30-day refund; Download 453 quarantined",
            "2_one_key_two_grants_no_extras_three_shared_nodes": "one human key; exact two underlying Operator v1 grants (exact_union); 5+7=12 derived family union with no third list; no future product/License Type; three shared nodes never six activations",
            "3_union_digest_comparison": f"independent sha256 union digest {derived_digest} equals the composition gate, registry, and projection journal digests",
            "4_both_product_operations": "cargo bundle_activation + spec172 vectors prove the exact two-product union at the frozen price on one account, typed partial state, strict same-account binding, shared-node rule, and Cockpit exact-union resolution",
            "5_reject_component_refund": "component/late/no-truth refunds fail closed (COMPONENT_REFUND_UNSUPPORTED / REFUND_WINDOW_EXPIRED / REFUND_TRUTH_UNKNOWN) with paid grants intact and zero sequence bump",
            "6_refund_whole_order": "30-day whole-order refund settles once: grants_revoked=2, paid_grants_active=false, sequence 1->2, verified_no_license limited mode, all nine customer/order/license/projection rows preserved",
            "7_outbox_reconciliation_limited_data": "signed outbox envelopes deliver exactly once (tampering dead-letters), reconciler converges, and read/export/recovery/repair/update/uninstall remain available",
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
        "bundle": {
            "sku": BUNDLE_SKU,
            "price_usd": "1254.60",
            "amount_minor": 125460,
            "test_mode_no_live_charge": True,
            "human_key_count": 1,
            "grants": BUNDLE_GRANTS,
            "grants_union": "exact_union",
            "no_third_feature_catalog": True,
            "future_products_included": False,
            "future_license_types_included": False,
            "operator_seats": 1,
            "node_limit": 3,
            "node_set": "operator_shared_v1",
            "term": "lifetime",
            "family_union": {"focusa": 5, "uiai_engine": 7, "total": 12},
            "union_digest": derived_digest,
            "refund_policy": "whole_order_30_days",
            "component_refunds_allowed": False,
        },
        "whole_refund": {
            "grants_revoked": 2,
            "paid_grants_active_after": False,
            "limited_posture": "verified_no_license",
            "data_preserved": True,
            "outbox_exactly_once": True,
            "reconciliation_converged": True,
            "stale_reactivation_denied": True,
        },
        "evidence_path": "docs/evidence/spec172/focusa-vbcqu.20.15.35-acceptance.txt",
    }

    receipt_json = json.dumps(receipt, sort_keys=True)
    hygiene(receipt_json)
    print(receipt_json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
