#!/usr/bin/env python3
"""Generate deterministic public and internal Spec 152E activation contracts."""

import argparse
import json
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/contracts/spec152e-activation-call-stack.v1.yaml"
PUBLIC = ROOT / "docs/contracts/spec152e-activation-public-openapi.v1.json"
INTERNAL = ROOT / "docs/contracts/spec152e-activation-internal.v1.json"
ERRORS = ROOT / "docs/contracts/spec152e-activation-errors.v1.json"

ERROR_DEFINITIONS = {
    "EMAIL_REQUIRED": (400, False, "provide_email", "An email is required to continue."),
    "EMAIL_VERIFICATION_REQUIRED": (403, False, "verify_email", "Mailbox verification is required."),
    "EMAIL_VERIFICATION_EXPIRED": (410, False, "restart_verification", "The verification challenge expired."),
    "EMAIL_VERIFICATION_FAILED": (403, False, "retry_verification_within_budget", "Mailbox verification failed."),
    "EMAIL_DELIVERY_FAILED": (503, True, "retry_or_use_recovery", "Verification delivery is unavailable."),
    "ACCOUNT_EMAIL_MISMATCH": (403, False, "verify_account_email", "The verified account does not match."),
    "ACCOUNT_MERGE_REVIEW_REQUIRED": (409, False, "contact_support", "Account ownership requires review."),
    "FACADE_ORIGIN_DENIED": (403, False, "use_registered_facade", "The facade origin is not allowed."),
    "FACADE_PRODUCT_DENIED": (403, False, "select_supported_product", "The product is unavailable on this facade."),
    "PRODUCT_MAPPING_REQUIRED": (409, False, "wait_for_product_mapping", "The product is not mapped for issuance."),
    "EDD_CUSTOMER_RESOLUTION_FAILED": (503, True, "retry_or_use_recovery", "Customer setup is unavailable."),
    "EDD_CHECKOUT_REQUIRED": (409, False, "open_checkout", "Checkout is required."),
    "EDD_ORDER_PENDING": (202, True, "poll_after_retry_after", "The order is pending."),
    "EDD_ORDER_UNVERIFIED": (409, False, "verify_checkout_identity", "The order cannot yet be verified."),
    "EDD_LICENSE_PENDING": (202, True, "poll_after_retry_after", "License issuance is pending."),
    "EDD_LICENSE_UNUSABLE": (403, False, "recovery_only", "The license cannot authorize this activation."),
    "EVALUATION_NOT_ELIGIBLE": (403, False, "select_paid_or_limited_access", "The requested legacy Evaluation journey is unavailable."),
    "LICENSE_DELIVERY_PENDING": (202, True, "poll_after_retry_after", "Credential delivery is pending."),
    "LICENSE_DELIVERY_FAILED": (503, False, "authenticated_recovery", "Credential delivery requires recovery."),
    "LICENSE_ACCOUNT_MISMATCH": (403, False, "verify_license_owner", "The license does not match the verified account."),
    "NODE_LIMIT_EXHAUSTED": (409, False, "manage_nodes", "The node limit is exhausted."),
    "AUTHORITY_UNAVAILABLE": (503, True, "retry_or_use_recovery", "The licensing authority is unavailable."),
    "ENTITLEMENT_REQUIRED": (403, False, "activate_or_manage_license", "An authority-issued entitlement is required."),
    "ENTITLEMENT_FEATURE_REQUIRED": (403, False, "manage_license", "The requested feature is not granted."),
    "ENTITLEMENT_LIMIT_EXHAUSTED": (429, False, "manage_limit", "The entitlement limit is exhausted."),
    "REQUEST_ID_REQUIRED": (400, False, "send_new_request_id", "A request identifier is required."),
    "IDEMPOTENCY_KEY_REQUIRED": (400, False, "send_idempotency_key", "An idempotency key is required."),
    "IDEMPOTENCY_CONFLICT": (409, False, "use_original_request_or_new_key", "The idempotency key conflicts with another request."),
    "REQUEST_IN_PROGRESS": (409, True, "retry_same_idempotency_key", "The original request is still in progress."),
    "POLL_CREDENTIAL_REQUIRED": (401, False, "restart_or_recover_activation", "A poll credential is required."),
    "POLL_CREDENTIAL_EXPIRED": (401, False, "restart_or_recover_activation", "The poll credential expired."),
    "REFUNDED": (403, False, "recovery_only", "The order was refunded."),
    "REVOKED": (403, False, "recovery_only", "The entitlement was revoked."),
}


def public_schema(stack):
    error_codes = sorted(ERROR_DEFINITIONS)
    envelope = {
        "type": "object",
        "additionalProperties": False,
        "required": stack["canonical_output"]["required"],
        "properties": {
            "schema": {"const": "focusa.activation.response.v1"},
            "request_id": {"type": "string", "minLength": 8},
            "registration_id": {"type": "string", "minLength": 8},
            "state": {"type": "string", "enum": stack["presenter_states"]},
            "terminal": {"type": "boolean"},
            "retry": {"$ref": "#/components/schemas/Retry"},
            "next_action": {"type": "string"},
            "masked_email": {"type": "string", "pattern": "^[^@]*\\*[^@]*@[^@]+$"},
            "safe_url": {"type": "string", "format": "uri"},
            "verification_delivery_status": {"type": "string"},
            "one_time_key_envelope": {"type": "string", "contentEncoding": "base64"},
            "node_id": {"type": "string"},
            "lease_envelope": {"type": "string", "contentEncoding": "base64"},
            "error": {"$ref": "#/components/schemas/Error"},
        },
    }
    components = {
        "CorrelationHeaders": {"type": "object", "required": ["request_id"], "properties": {"request_id": {"type": "string"}, "idempotency_key": {"type": "string"}}},
        "MutationRequest": {"type": "object", "additionalProperties": False, "required": ["request_id", "idempotency_key", "registration_id"], "properties": {"request_id": {"type": "string"}, "idempotency_key": {"type": "string"}, "registration_id": {"type": "string"}, "payload": {"type": "object"}}},
        "ReadRequest": {"type": "object", "additionalProperties": False, "required": ["request_id"], "properties": {"request_id": {"type": "string"}}},
        "Retry": {"type": "object", "additionalProperties": False, "required": ["posture"], "properties": {"posture": {"enum": ["none", "safe_retry", "retry_same_idempotency_key", "restart", "recovery_only"]}, "retry_after_seconds": {"type": "integer", "minimum": 1, "maximum": stack["polling"]["maximum_retry_after_seconds"]}}},
        "Error": {"type": "object", "additionalProperties": False, "required": ["code", "message", "next_action"], "properties": {"code": {"type": "string", "enum": error_codes}, "message": {"type": "string"}, "next_action": {"type": "string"}}},
        "ActivationEnvelope": envelope,
    }
    paths = {}
    for op in stack["operations"]:
        method = op["method"].lower()
        operation = {
            "operationId": op["id"],
            "x-focusa-authority": "WPUIAI.com EDD",
            "x-focusa-handler": op["handler"],
            "x-focusa-idempotency": op["mutation"],
            "x-focusa-failure-codes": op["failures"],
            "parameters": [{"name": "X-Request-Id", "in": "header", "required": True, "schema": {"type": "string"}}],
            "responses": {"200": {"description": "Redacted activation state", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ActivationEnvelope"}}}}, "default": {"description": "Stable public-safe failure", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ActivationEnvelope"}}}}},
        }
        if op["mutation"]:
            operation["parameters"].append({"name": "Idempotency-Key", "in": "header", "required": True, "schema": {"type": "string"}})
            operation["requestBody"] = {"required": True, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/MutationRequest"}}}}
        paths.setdefault(op["path"], {})[method] = operation
    return {"openapi": "3.1.0", "info": {"title": "Spec 152E Universal Activation Facade API", "version": "1.0.0"}, "jsonSchemaDialect": "https://json-schema.org/draft/2020-12/schema", "servers": [{"url": "https://{registeredFacade}", "variables": {"registeredFacade": {"default": "facade.invalid"}}}], "paths": paths, "components": {"schemas": components}, "x-focusa-facade-authority": "proxy_only", "x-focusa-spec158": "excluded"}


def build():
    stack = yaml.safe_load(SOURCE.read_text(encoding="utf-8"))
    used = sorted({code for op in stack["operations"] for code in op["failures"]})
    missing = set(used) - set(ERROR_DEFINITIONS)
    if missing:
        raise ValueError(f"missing error definitions: {sorted(missing)}")
    errors = {"schema": "focusa.spec152e.activation_errors.v1", "contract_version": 1, "owner": "WPUIAI/wpuiai", "rules": {"codes_are_stable": True, "messages_are_public_safe": True, "presenters_must_not_rewrite": True, "unknown_codes_fail_closed": True}, "errors": [{"code": code, "http_status": ERROR_DEFINITIONS[code][0], "retryable": ERROR_DEFINITIONS[code][1], "safe_next_action": ERROR_DEFINITIONS[code][2], "public_message": ERROR_DEFINITIONS[code][3]} for code in sorted(ERROR_DEFINITIONS)]}
    internal = {"schema": "focusa.spec152e.activation_internal.v1", "contract_version": 1, "authority": stack["authority"], "invariants": stack["invariants"], "request_context": stack["request_context"], "registration_states": stack["registration_states"], "presenter_states": stack["presenter_states"], "polling": stack["polling"], "canonical_output": stack["canonical_output"], "stages": stack["stages"], "operations": stack["operations"], "error_registry": str(ERRORS.relative_to(ROOT))}
    return public_schema(stack), internal, errors


def render(value):
    return json.dumps(value, indent=2, sort_keys=True) + "\n"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    values = zip((PUBLIC, INTERNAL, ERRORS), build())
    stale = []
    for path, value in values:
        expected = render(value)
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8") != expected:
                stale.append(str(path.relative_to(ROOT)))
        else:
            path.write_text(expected, encoding="utf-8")
    if stale:
        raise SystemExit("stale generated contracts: " + ", ".join(stale))
    print("Spec 152E activation contracts are current" if args.check else "Generated Spec 152E activation contracts")


if __name__ == "__main__":
    main()
