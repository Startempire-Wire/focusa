#!/usr/bin/env python3
"""Contract gate for Spec 172 products, License Types, and Bundle policy."""

from __future__ import annotations

import hashlib
import json
import pathlib
import sys
from decimal import Decimal

import yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs/contracts/spec172-license-types.v1.yaml"
EDD = ROOT / "docs/contracts/spec152e-edd-product-registry.v1.yaml"


def load(path: pathlib.Path) -> dict:
    with path.open(encoding="utf-8") as handle:
        value = yaml.safe_load(handle)
    if not isinstance(value, dict):
        raise AssertionError(f"{path}: expected mapping")
    return value


def main() -> int:
    failures: list[str] = []
    registry = load(REGISTRY)
    edd = load(EDD)

    digest_copy = dict(registry)
    claimed_digest = digest_copy.pop("semantic_digest", None)
    actual_digest = "sha256:" + hashlib.sha256(
        json.dumps(digest_copy, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    ).hexdigest()
    if claimed_digest != actual_digest:
        failures.append(f"semantic digest mismatch: claimed={claimed_digest} actual={actual_digest}")

    postures = {item["code"]: item for item in registry.get("postures", [])}
    limited = postures.get("verified_no_license", {})
    expected_limited = {
        "kind": "account_runtime_posture",
        "is_license_type": False,
        "price_usd": "0.00",
        "duration": "no_automatic_expiry",
        "edd_software_license_key": False,
        "anonymous_access": False,
    }
    for key, expected in expected_limited.items():
        if limited.get(key) != expected:
            failures.append(f"verified_no_license.{key}: expected {expected!r}, got {limited.get(key)!r}")

    types = {item["code"]: item for item in registry.get("license_types", [])}
    expected_products = {
        "focusa_operator_lifetime_v1": "focusa",
        "uiai_operator_lifetime_v1": "uiai_engine",
    }
    for code, product in expected_products.items():
        item = types.get(code, {})
        expected = {
            "product_code": product,
            "price_usd": "697.00",
            "sale_status": "approved_not_yet_enabled",
            "term": "lifetime",
            "operator_seats": 1,
            "node_limit": 3,
            "node_set": "operator_shared_v1",
            "refund_policy": "whole_order_30_days",
            "future_license_types_included": False,
            "future_products_included": False,
        }
        for key, value in expected.items():
            if item.get(key) != value:
                failures.append(f"{code}.{key}: expected {value!r}, got {item.get(key)!r}")

    bundles = {item["code"]: item for item in registry.get("composite_skus", [])}
    bundle = bundles.get("focusa_uiai_operator_bundle_lifetime_v1", {})
    exact_grants = ["focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1"]
    if bundle.get("grants") != exact_grants:
        failures.append(f"Bundle grants must be exact ordered union {exact_grants!r}")
    if Decimal(bundle.get("price_usd", "0")) != Decimal("1254.60"):
        failures.append("Bundle price must be exactly 1254.60")
    if Decimal(bundle.get("standalone_sum_usd", "0")) * Decimal("0.90") != Decimal("1254.6000"):
        failures.append("Bundle formula must be ten percent below 1394.00")
    for key, value in {
        "sale_status": "approved_not_yet_enabled",
        "operator_seats": 1,
        "node_limit": 3,
        "node_set": "operator_shared_v1",
        "refund_policy": "whole_order_30_days",
        "component_refunds_allowed": False,
        "future_license_types_included": False,
        "future_products_included": False,
    }.items():
        if bundle.get(key) != value:
            failures.append(f"Bundle {key}: expected {value!r}, got {bundle.get(key)!r}")

    refund = registry.get("refund_policies", {}).get("whole_order_30_days", {})
    if refund != {"window_days": 30, "scope": "whole_order", "component_refunds_allowed": False}:
        failures.append("whole_order_30_days policy is not canonical")
    for key in ("unknown_product", "future_product", "unknown_license_type", "future_license_type"):
        if registry.get("defaults", {}).get(key) != "excluded":
            failures.append(f"defaults.{key} must be excluded")

    offers = {item["public_code"]: item for item in edd.get("protected_offers", [])}
    if set(offers) != set(expected_products) | {"focusa_uiai_operator_bundle_lifetime_v1"}:
        failures.append("EDD protected offers must contain only the three canonical paid records")
    for code, item in offers.items():
        if item.get("checkout_enabled") is not False or item.get("mapping_status") != "approved_policy_blocked_edd_mapping":
            failures.append(f"{code}: checkout must remain blocked pending dedicated EDD mapping")
    edd_bundle = offers.get("focusa_uiai_operator_bundle_lifetime_v1", {})
    if edd_bundle.get("grants") != exact_grants or edd_bundle.get("price_usd") != "1254.60":
        failures.append("EDD Bundle projection differs from Spec 172 exact union/price")
    edd_limited = edd.get("verified_no_license", {})
    if edd_limited.get("is_license_type") is not False or edd_limited.get("edd_software_license_key") is not False:
        failures.append("EDD registry must project verified_no_license as non-license/no-key posture")

    serialized = json.dumps(edd, sort_keys=True)
    if "focusa_evaluation" in serialized:
        failures.append("active EDD registry still contains focusa_evaluation")

    if failures:
        print("Spec 172 License Type contract test FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Spec 172 License Type contract test passed")
    print(f"semantic_digest={claimed_digest}")
    print("license_types=2 composite_skus=1 postures=1")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
