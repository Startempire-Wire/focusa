#!/usr/bin/env python3
"""Validate and generate deterministic public Spec 152E facade descriptors."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from urllib.parse import urlsplit

import yaml

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/contracts/spec152e-facade-registry.v1.yaml"
JSON_OUTPUT = ROOT / "docs/contracts/spec152e-facade-registry.v1.json"
PHP_OUTPUT = ROOT / "docs/contracts/spec152e-facade-registry.v1.php"
PRODUCT_REGISTRY = ROOT / "docs/contracts/spec152e-edd-product-registry.v1.yaml"
CALL_STACK = ROOT / "docs/contracts/spec152e-activation-call-stack.v1.yaml"

EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+|focusa_live_[0-9]+_[0-9a-f]+")
FACADE_KEYS = {
    "facade_id", "status", "exact_origins", "products", "brand", "sender",
    "paths", "callbacks", "locale", "presenter_capabilities", "rate_policy_ref",
    "abuse_policy_ref",
}
PATH_KEYS = {"verification", "checkout", "success", "cancel", "manage", "recovery"}
CALLBACK_KEYS = {"success", "cancel", "recovery"}
BOUNDED_CAPABILITIES = {
    "browser_registration", "installer_handoff", "terminal_continuation",
    "bounded_authority_proxy", "marketing_offer_display", "engine_installer_handoff",
    "account_management_handoff", "edd_checkout_handoff",
}


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def _exact_origin(value: object) -> str:
    _require(isinstance(value, str) and value != "", "origin must be a non-empty string")
    parsed = urlsplit(value)
    _require(parsed.scheme == "https", f"origin must use exact https: {value!r}")
    _require(parsed.netloc != "" and parsed.hostname is not None, f"origin has no host: {value!r}")
    _require(parsed.username is None and parsed.password is None, f"origin contains user info: {value!r}")
    _require(parsed.port is None, f"origin contains a port: {value!r}")
    _require(parsed.path == "" and parsed.query == "" and parsed.fragment == "", f"origin must not contain path/query/fragment: {value!r}")
    _require(parsed.hostname == parsed.hostname.lower(), f"origin host must be lowercase: {value!r}")
    _require("*" not in value and value == f"https://{parsed.hostname}", f"origin is not exact: {value!r}")
    return value


def _relative_path(value: object, label: str) -> str:
    _require(isinstance(value, str) and value.startswith("/"), f"{label} must be an absolute relative path")
    parsed = urlsplit(value)
    _require(parsed.scheme == "" and parsed.netloc == "", f"{label} must not be a URL")
    _require(parsed.query == "" and parsed.fragment == "", f"{label} must not contain query/fragment")
    _require("*" not in value and "//" not in value and ".." not in parsed.path.split("/"), f"{label} is not exact")
    return value


def validate(registry: dict) -> dict:
    _require(registry.get("schema") == "focusa.spec152e.facade_registry.v1", "invalid schema")
    _require(registry.get("registry_version") == 1, "invalid registry version")
    _require(registry.get("owner") == "WPUIAI/wpuiai", "invalid owner")
    authority = registry.get("authority", {})
    _require(authority.get("canonical") == "WPUIAI.com EDD", "canonical EDD authority required")
    _require(authority.get("facade_role") == "presenter_and_bounded_proxy_only", "facades must be presenter-only")
    for key in ("entitlement_issuance", "customer_or_commerce_truth", "wildcard_authority"):
        _require(authority.get(key) == "forbidden", f"authority.{key} must be forbidden")
    _require(authority.get("spec158") == "excluded", "Spec 158 must remain excluded")

    request = registry.get("request_contract", {})
    required = request.get("required", [])
    forbidden = request.get("forbidden", [])
    _require(len(required) == len(set(required)) and len(forbidden) == len(set(forbidden)), "request fields must be unique")
    _require(not set(required) & set(forbidden), "request required/forbidden fields overlap")
    _require(set(required) == {"facade_id", "origin", "product_code", "route", "callback_handle", "locale", "timestamp", "request_id", "idempotency_key"}, "request contract is not exact")
    for field in ("edd_download_id", "edd_price_id", "price", "products", "features", "grants", "limits", "sender_email", "callback_url", "redirect_url", "authority", "credential", "secret"):
        _require(field in forbidden, f"request forbidden field missing: {field}")
    for denial in ("unknown_facade", "unknown_origin", "unknown_product", "unknown_route", "unknown_callback", "unknown_sender", "unknown_locale"):
        _require(str(request.get(denial, "")).startswith("FACADE_") and str(request[denial]).endswith("_DENIED"), f"missing fail-closed denial: {denial}")

    call_stack = yaml.safe_load(CALL_STACK.read_text(encoding="utf-8"))
    expected_routes = {row["id"].replace(".", "_"): row["path"] for row in call_stack["operations"]}
    _require(registry.get("proxy_routes") == expected_routes, "proxy routes differ from the canonical activation call stack")
    products = yaml.safe_load(PRODUCT_REGISTRY.read_text(encoding="utf-8"))
    allowed_products = {row["public_code"] for row in products["protected_offers"]}

    facades = registry.get("facades")
    _require(isinstance(facades, list) and facades, "facades must be a non-empty list")
    facade_ids: set[str] = set()
    origins: set[str] = set()
    senders: set[str] = set()
    for facade in facades:
        _require(set(facade) == FACADE_KEYS, f"facade fields are not exact: {facade.get('facade_id')!r}")
        facade_id = facade["facade_id"]
        _require(isinstance(facade_id, str) and re.fullmatch(r"[a-z0-9_]+_v[0-9]+", facade_id) is not None, f"invalid facade ID: {facade_id!r}")
        _require(facade_id not in facade_ids, f"duplicate facade ID: {facade_id}")
        facade_ids.add(facade_id)
        _require(facade["status"] == "registered_presenter", f"facade is not presenter-only: {facade_id}")
        _require(isinstance(facade["exact_origins"], list) and facade["exact_origins"], f"missing origins: {facade_id}")
        for raw_origin in facade["exact_origins"]:
            origin = _exact_origin(raw_origin)
            _require(origin not in origins, f"origin registered more than once: {origin}")
            origins.add(origin)
        _require(isinstance(facade["products"], list) and facade["products"], f"missing products: {facade_id}")
        _require(len(facade["products"]) == len(set(facade["products"])), f"duplicate products: {facade_id}")
        _require(set(facade["products"]) <= allowed_products, f"unknown product in {facade_id}")
        _require(set(facade["brand"]) == {"name", "logo_path"}, f"brand is not bounded: {facade_id}")
        _require(isinstance(facade["brand"]["name"], str) and facade["brand"]["name"], f"missing brand name: {facade_id}")
        _relative_path(facade["brand"]["logo_path"], f"{facade_id}.brand.logo_path")
        _require(set(facade["sender"]) == {"identity", "display_name"}, f"sender is not bounded: {facade_id}")
        sender = facade["sender"]["identity"]
        _require(isinstance(sender, str) and re.fullmatch(r"[a-z0-9_]+_v[0-9]+", sender) is not None, f"invalid sender identity: {facade_id}")
        _require(sender not in senders, f"sender identity registered more than once: {sender}")
        senders.add(sender)
        _require("@" not in facade["sender"]["display_name"], f"email-like sender forbidden: {facade_id}")
        _require(set(facade["paths"]) == PATH_KEYS, f"paths are not exact: {facade_id}")
        _require(set(facade["callbacks"]) == CALLBACK_KEYS, f"callbacks are not exact: {facade_id}")
        for key, value in facade["paths"].items():
            _relative_path(value, f"{facade_id}.paths.{key}")
        for key, value in facade["callbacks"].items():
            _relative_path(value, f"{facade_id}.callbacks.{key}")
        locale = facade["locale"]
        _require(set(locale) == {"default", "allowed"} and isinstance(locale["allowed"], list), f"locale is not bounded: {facade_id}")
        _require(locale["allowed"] and locale["default"] in locale["allowed"] and len(locale["allowed"]) == len(set(locale["allowed"])), f"locale allowlist invalid: {facade_id}")
        _require(all(re.fullmatch(r"[a-z]{2}-[A-Z]{2}", item or "") for item in locale["allowed"]), f"invalid locale: {facade_id}")
        capabilities = facade["presenter_capabilities"]
        _require(isinstance(capabilities, list) and "bounded_authority_proxy" in capabilities, f"bounded proxy capability missing: {facade_id}")
        _require(len(capabilities) == len(set(capabilities)) and set(capabilities) <= BOUNDED_CAPABILITIES, f"unknown presenter capability: {facade_id}")
        _require(facade["rate_policy_ref"] == "facade_standard_v1", f"unknown rate policy: {facade_id}")
        _require(facade["abuse_policy_ref"] == "authority_facade_abuse_v1", f"unknown abuse policy: {facade_id}")

    raw = json.dumps(registry, sort_keys=True)
    _require("*" not in raw, "wildcards are forbidden")
    _require(EMAIL.search(raw) is None, "email addresses are forbidden in the public descriptor")
    _require(SECRET.search(raw) is None, "secret-like values are forbidden in the public descriptor")
    registry["counts"] = {
        "facades": len(facades),
        "exact_origins": len(origins),
        "product_bindings": sum(len(row["products"]) for row in facades),
        "sender_identities": len(senders),
        "callback_handles": sum(len(row["callbacks"]) for row in facades),
        "proxy_routes": len(expected_routes),
    }
    return registry


def build() -> dict:
    source = yaml.safe_load(SOURCE.read_text(encoding="utf-8"))
    _require(isinstance(source, dict), "registry root must be an object")
    return validate(source)


def render_json(registry: dict) -> str:
    return json.dumps(registry, indent=2, sort_keys=True, ensure_ascii=False) + "\n"


def render_php(registry: dict) -> str:
    compact = json.dumps(registry, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return """<?php
// Generated by scripts/generate-spec152e-facade-registry.py; do not hand-edit.
declare(strict_types=1);
return json_decode(<<<'FOCUSA_FACADE_REGISTRY_JSON'
%s
FOCUSA_FACADE_REGISTRY_JSON, true, 512, JSON_THROW_ON_ERROR);
""" % compact


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="validate and fail if generated outputs are stale")
    args = parser.parse_args()
    registry = build()
    outputs = {JSON_OUTPUT: render_json(registry), PHP_OUTPUT: render_php(registry)}
    if args.check:
        stale = [str(path.relative_to(ROOT)) for path, content in outputs.items() if not path.exists() or path.read_text(encoding="utf-8") != content]
        if stale:
            raise SystemExit("stale Spec 152E facade registry outputs: " + ", ".join(stale))
    else:
        for path, content in outputs.items():
            path.write_text(content, encoding="utf-8")
    print(json.dumps({"schema": "focusa.spec152e.facade_registry_validation.v1", **registry["counts"], "result": "passed"}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
