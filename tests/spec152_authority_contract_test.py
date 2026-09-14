#!/usr/bin/env python3
"""Validate the public-safe Spec 152 evaluation issuance contract and fixtures."""

from __future__ import annotations

import json
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec152-evaluation-issuance-contract.v1.schema.json"
OPENAPI = ROOT / "docs/contracts/spec152-evaluation-issuance-openapi.v1.yaml"
OWNERSHIP = ROOT / "docs/contracts/spec152-evaluation-issuance-ownership.v1.yaml"
FIXTURES = ROOT / "tests/fixtures"

schema = json.loads(CONTRACT.read_text(encoding="utf-8"))
Draft202012Validator.check_schema(schema)
checker = FormatChecker()
request_validator = Draft202012Validator(schema["$defs"]["request"], resolver=None, format_checker=checker)
response_validator = Draft202012Validator(schema["$defs"]["response"], resolver=None, format_checker=checker)

# Local refs require the complete root schema during validation.
request_validator = Draft202012Validator(
    {"$ref": "#/$defs/request", "$defs": schema["$defs"]}, format_checker=checker
)
response_validator = Draft202012Validator(
    {"$ref": "#/$defs/response", "$defs": schema["$defs"]}, format_checker=checker
)

def load(name: str) -> dict:
    return json.loads((FIXTURES / name).read_text(encoding="utf-8"))

valid_request = load("spec152-evaluation-issuance-valid.json")
valid_response = load("spec152-evaluation-issuance-response-valid.json")
request_validator.validate(valid_request)
response_validator.validate(valid_response)

for name in [
    "spec152-evaluation-issuance-invalid-raw-email.json",
    "spec152-evaluation-issuance-invalid-unverified.json",
]:
    errors = list(request_validator.iter_errors(load(name)))
    assert errors, f"invalid request fixture unexpectedly accepted: {name}"

invalid_response = load("spec152-evaluation-issuance-response-invalid-pending-lease.json")
assert list(response_validator.iter_errors(invalid_response)), "pending response cannot carry a lease"

request = schema["$defs"]["request"]
assert request["additionalProperties"] is False
assert "email" not in request["properties"]
assert request["properties"]["email_verification"]["properties"]["state"] == {"const": "verified"}
assert "marketing_consent" in request["required"]
assert request["properties"]["redaction_class"] == {"const": "public_safe_no_direct_identifier"}
assert schema["$defs"]["response"]["properties"]["state"]["enum"] == ["issued", "pending", "denied"]

openapi = yaml.safe_load(OPENAPI.read_text(encoding="utf-8"))
operation = openapi["paths"]["/v1/evaluations/issue"]["post"]
assert operation["operationId"] == "issueEvaluationLease"
assert {"200", "409", "422", "503"} <= set(operation["responses"])
assert operation["requestBody"]["content"]["application/json"]["schema"]["$ref"].endswith("#/$defs/request")

ownership = yaml.safe_load(OWNERSHIP.read_text(encoding="utf-8"))
assert ownership["authority"]["repository"] == "WPUIAI/wpuiai"
assert "signed lease issuance and revocation" in ownership["authority"]["owns"]
assert "self-issuing an evaluation lease" in ownership["focusa"]["forbidden"]
assert "inferring marketing consent from evaluation acceptance" in ownership["focusa"]["forbidden"]
assert ownership["redaction"]["class"] == "public_safe_no_direct_identifier"

serialized_valid = json.dumps(valid_request).lower()
assert "@" not in serialized_valid
assert "email_address" not in serialized_valid
assert valid_request["marketing_consent"]["status"] == "declined"

print("Spec152 authority evaluation contract: PASS")
