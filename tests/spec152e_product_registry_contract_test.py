#!/usr/bin/env python3
"""Validate deterministic, fail-closed Spec 152E EDD product registry."""

import importlib.util
import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/contracts/spec152e-edd-product-registry.v1.yaml"
OUTPUT = ROOT / "docs/contracts/spec152e-edd-product-registry.v1.json"
PHP_OUTPUT = ROOT / "docs/contracts/spec152e-edd-product-registry.v1.php"
GENERATOR = ROOT / "scripts/generate-spec152e-product-registry.py"
INVENTORY = ROOT / "docs/contracts/spec152e-deployed-surface-inventory.v1.json"

spec = importlib.util.spec_from_file_location("spec152e_product_generator", GENERATOR)
module = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(module)

source = yaml.safe_load(SOURCE.read_text(encoding="utf-8"))
actual = json.loads(OUTPUT.read_text(encoding="utf-8"))
expected = module.build()
assert actual == expected
assert module.render_json(expected) == OUTPUT.read_text(encoding="utf-8")
assert module.render_php(expected) == PHP_OUTPUT.read_text(encoding="utf-8")
for path in (SOURCE, OUTPUT, PHP_OUTPUT, GENERATOR, Path(__file__)):
    assert len(path.read_text(encoding="utf-8").splitlines()) < 500, path

assert actual["schema"] == "focusa.spec152e.edd_product_registry.v1"
assert actual["owner"] == "WPUIAI/wpuiai"
assert actual["authority"]["customer_commerce_human_key_refund_entitlement"] == "WPUIAI.com EDD"
assert actual["authority"]["runtime_grant"] == "authority_issued_signed_lease"
assert actual["authority"]["spec158"] == "excluded"
assert set(actual["authority"]["caller_controls_forbidden"]) == {
    "edd_download_id", "edd_price_id", "price", "tier", "products", "features",
    "limits", "commercial_rights", "evaluation_duration", "node_limit",
}

expected_codes = {
    "focusa_operator",
    "uiai_engine_operator",
    "focusa_uiai_bundle",
    "focusa_evaluation",
}
offers = actual["protected_offers"]
assert {row["public_code"] for row in offers} == expected_codes
assert len(offers) == len(expected_codes)
for offer in offers:
    assert offer["mapping_status"] == "blocked_unassigned"
    assert offer["checkout_enabled"] is False
    assert offer["edd_download_id"] is None
    assert offer["edd_price_id"] is None
    assert offer["license_duration"] is None
    assert offer["node_limit"] is None
    assert offer["supported_facades"] == []
    assert offer["features"] == []
    assert offer["commercial_rights"] == []
    assert offer["products"]
assert next(row for row in offers if row["public_code"] == "focusa_uiai_bundle")["products"] == ["focusa", "uiai-engine"]
assert next(row for row in offers if row["public_code"] == "focusa_evaluation")["evaluation"] is True
assert all(row["evaluation"] is False for row in offers if row["public_code"] != "focusa_evaluation")

catalog = actual["current_edd_catalog"]["entries"]
assert len(catalog) == 14
assert len({row["download_id"] for row in catalog}) == 14
assert {row["download_id"] for row in catalog} == {16, 17, 21, 22, 23, 24, 25, 66, 452, 453, 454, 455, 456, 457}
assert {row["entitlement_disposition"] for row in catalog} == {"quarantine", "retire"}
download453 = next(row for row in catalog if row["download_id"] == 453)
assert download453["title"] == "WPUIAI Pro Lifetime"
assert download453["entitlement_disposition"] == "quarantine"
assert download453["reason"] == "implicit_focusa_mapping_forbidden"
assert all(row["edd_download_id"] != 453 for row in offers)
credit_packs = [row for row in catalog if row["download_id"] in {455, 456, 457}]
assert all(row["entitlement_disposition"] == "retire" and not row["licensing_enabled"] for row in credit_packs)

legacy = actual["legacy_record_classes"]
assert len(legacy) == 10
assert len({row["id"] for row in legacy}) == len(legacy)
assert {row["disposition"] for row in legacy} == {"migrate", "quarantine", "retire"}
assert sum(row["observed_count"] for row in legacy if row["id"].startswith("edd_")) == 11
assert sum(row["observed_count"] for row in legacy if row["id"].startswith("install_")) == 83
assert all(row["requirement"] for row in legacy)
refunded = next(row for row in legacy if row["id"] == "install_refunded_focusa")
revoked = next(row for row in legacy if row["id"] == "install_revoked_focusa")
assert refunded["disposition"] == revoked["disposition"] == "retire"
assert "never_reactivate" in refunded["requirement"] and "never_reactivate" in revoked["requirement"]

assert actual["counts"] == {
    "protected_offers": 4,
    "checkout_enabled": 0,
    "assigned_edd_downloads": 0,
    "catalog_entries": 14,
    "legacy_record_classes": 10,
    "legacy_migrate": 3,
    "legacy_quarantine": 5,
    "legacy_retire": 2,
}
assert set(actual["invariants"]) >= {
    "no_protected_offer_is_currently_checkout_enabled",
    "download_453_is_not_focusa_authority",
    "credit_packs_never_grant_entitlement",
    "caller_metadata_never_selects_price_product_grant_limit_or_right",
    "evaluation_requires_verified_identity_and_dedicated_edd_mapping",
    "paid_records_are_never_downgraded_to_evaluation",
    "refunded_or_revoked_records_never_reactivate",
    "legacy_email_match_alone_never_transfers_ownership",
    "every_legacy_class_is_migrate_quarantine_or_retire",
}
assert len(actual["activation_requirements"]) == 10

inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
assert inventory["bounded_aggregates"]["download_453"]["title"] == download453["title"]
assert inventory["bounded_aggregates"]["wpuiai_edd_licenses"]["total"] == 11
assert sum(row["count"] for row in inventory["bounded_aggregates"]["install_registry"]) == 83

raw = OUTPUT.read_text(encoding="utf-8")
assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)
assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw)
assert not re.search(r"focusa_live_[0-9]+_[0-9a-f]+", raw)

print(json.dumps({
    "schema": "focusa.spec152e.product_registry_contract_validation.v1",
    **actual["counts"],
    "result": "passed_fail_closed_pending_operator_mapping",
}, sort_keys=True))
