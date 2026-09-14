#!/usr/bin/env python3
"""Validate the bounded, generated Spec 152E facade registry contract."""

import copy
import importlib.util
import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/contracts/spec152e-facade-registry.v1.yaml"
OUTPUT = ROOT / "docs/contracts/spec152e-facade-registry.v1.json"
PHP_OUTPUT = ROOT / "docs/contracts/spec152e-facade-registry.v1.php"
GENERATOR = ROOT / "scripts/generate-spec152e-facade-registry.py"
PRODUCT_REGISTRY = ROOT / "docs/contracts/spec152e-edd-product-registry.v1.json"
CALL_STACK = ROOT / "docs/contracts/spec152e-activation-call-stack.v1.yaml"

spec = importlib.util.spec_from_file_location("spec152e_facade_generator", GENERATOR)
module = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(module)

source = yaml.safe_load(SOURCE.read_text(encoding="utf-8"))
actual = json.loads(OUTPUT.read_text(encoding="utf-8"))
expected = module.build()
assert actual == expected
assert module.render_json(expected) == OUTPUT.read_text(encoding="utf-8")
assert module.render_php(expected) == PHP_OUTPUT.read_text(encoding="utf-8")

assert actual["schema"] == "focusa.spec152e.facade_registry.v1"
assert actual["registry_version"] == 1
assert actual["owner"] == "WPUIAI/wpuiai"
assert actual["authority"] == {
    "canonical": "WPUIAI.com EDD",
    "facade_role": "presenter_and_bounded_proxy_only",
    "entitlement_issuance": "forbidden",
    "customer_or_commerce_truth": "forbidden",
    "wildcard_authority": "forbidden",
    "spec158": "excluded",
}

expected_origins = {
    "https://install.focusa.dev",
    "https://focusa.dev",
    "https://forge.focusa.dev",
    "https://arena.focusa.dev",
    "https://engine.focusa.dev",
    "https://wpuiai.com",
}
facades = actual["facades"]
assert {origin for row in facades for origin in row["exact_origins"]} == expected_origins
assert {row["facade_id"] for row in facades} == {
    "focusa_install_v1", "focusa_marketing_v1", "focusa_forge_v1",
    "focusa_arena_v1", "uiai_engine_v1", "wpuiai_public_v1",
}
assert all(row["status"] == "registered_presenter" for row in facades)
assert all("bounded_authority_proxy" in row["presenter_capabilities"] for row in facades)
assert len({row["sender"]["identity"] for row in facades}) == len(facades)

product_registry = json.loads(PRODUCT_REGISTRY.read_text(encoding="utf-8"))
product_codes = {row["public_code"] for row in product_registry["protected_offers"]}
assert all(set(row["products"]) <= product_codes for row in facades)
assert next(row for row in facades if row["facade_id"] == "uiai_engine_v1")["products"] == [
    "uiai_operator_lifetime_v1", "focusa_uiai_operator_bundle_lifetime_v1",
]

call_stack = yaml.safe_load(CALL_STACK.read_text(encoding="utf-8"))
expected_routes = {row["id"].replace(".", "_"): row["path"] for row in call_stack["operations"]}
assert actual["proxy_routes"] == expected_routes
assert set(actual["request_contract"]["required"]) == {
    "facade_id", "origin", "product_code", "route", "callback_handle",
    "locale", "timestamp", "request_id", "idempotency_key",
}
assert {"edd_download_id", "edd_price_id", "price", "products", "features", "grants", "limits", "sender_email", "callback_url", "redirect_url", "authority", "credential", "secret"} <= set(actual["request_contract"]["forbidden"])

raw = OUTPUT.read_text(encoding="utf-8")
assert "*" not in raw
assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)
assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw, re.I)
assert not re.search(r"focusa_live_[0-9]+_[0-9a-f]+", raw, re.I)

# Validator mutation checks prove that config cannot widen each authority dimension.
def denied(mutator, message):
    candidate = copy.deepcopy(source)
    mutator(candidate)
    try:
        module.validate(candidate)
    except ValueError:
        return
    raise AssertionError(message)


denied(lambda row: row["facades"][0]["exact_origins"].append("https://*.focusa.dev"), "wildcard origin accepted")
denied(lambda row: row["facades"][0]["exact_origins"].append("https://child.install.focusa.dev/path"), "non-origin URL accepted")
denied(lambda row: row["facades"][0]["products"].append("attacker_product_v1"), "unknown product accepted")
denied(lambda row: row["proxy_routes"].update({"authority_issue": "/v1/authority/issue"}), "unknown route accepted")
denied(lambda row: row["facades"][0]["callbacks"].update({"success": "https://evil.invalid/callback"}), "absolute callback accepted")
denied(lambda row: row["facades"][0]["sender"].update({"identity": "attacker"}), "unregistered sender shape accepted")
denied(lambda row: row["facades"][0]["locale"].update({"allowed": ["en-US", "*"]}), "wildcard locale accepted")
denied(lambda row: row["authority"].update({"entitlement_issuance": "allowed"}), "facade issuance authority accepted")

assert actual["counts"] == {
    "facades": 6,
    "exact_origins": 6,
    "product_bindings": 14,
    "sender_identities": 6,
    "callback_handles": 18,
    "proxy_routes": 11,
}
print(json.dumps({
    "schema": "focusa.spec152e.facade_registry_contract_validation.v1",
    **actual["counts"],
    "negative_validator_checks": 8,
    "result": "passed_fail_closed",
}, sort_keys=True))
