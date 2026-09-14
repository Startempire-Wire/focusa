#!/usr/bin/env python3
"""Validate the generated, typed Spec 152E authority call-stack contracts."""

import importlib.util
import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/contracts/spec152e-activation-call-stack.v1.yaml"
PUBLIC = ROOT / "docs/contracts/spec152e-activation-public-openapi.v1.json"
INTERNAL = ROOT / "docs/contracts/spec152e-activation-internal.v1.json"
ERRORS = ROOT / "docs/contracts/spec152e-activation-errors.v1.json"
GENERATOR = ROOT / "scripts/generate-spec152e-activation-contracts.py"

spec = importlib.util.spec_from_file_location("spec152e_activation_generator", GENERATOR)
module = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(module)

source = yaml.safe_load(SOURCE.read_text(encoding="utf-8"))
public = json.loads(PUBLIC.read_text(encoding="utf-8"))
internal = json.loads(INTERNAL.read_text(encoding="utf-8"))
errors = json.loads(ERRORS.read_text(encoding="utf-8"))
expected_public, expected_internal, expected_errors = module.build()
assert (public, internal, errors) == (expected_public, expected_internal, expected_errors)
assert all(module.render(value) == path.read_text(encoding="utf-8") for path, value in ((PUBLIC, public), (INTERNAL, internal), (ERRORS, errors)))

expected_surfaces = {
    ("POST", "/v1/activation/start"),
    ("POST", "/v1/activation/verify"),
    ("GET", "/v1/activation/offers"),
    ("POST", "/v1/activation/select-offer"),
    ("POST", "/v1/activation/checkout"),
    ("POST", "/v1/activation/existing-license"),
    ("POST", "/v1/activation/poll"),
    ("POST", "/v1/lease/refresh"),
    ("GET", "/v1/nodes"),
    ("POST", "/v1/nodes/deactivate"),
    ("GET", "/v1/account/manage-link"),
}
operations = internal["operations"]
assert {(op["method"], op["path"]) for op in operations} == expected_surfaces
assert len({op["id"] for op in operations}) == len(operations)
assert internal["authority"]["facade"] == "registered_authenticated_bounded_proxy_only"
assert internal["authority"]["spec158"] == "excluded"
assert public["x-focusa-facade-authority"] == "proxy_only"
assert public["x-focusa-spec158"] == "excluded"

stage_ids = [stage["id"] for stage in internal["stages"]]
assert stage_ids == [
    "facade_request_binding", "correlation_and_idempotency", "operation_handler",
    "identity_service", "edd_service", "device_service", "lease_service",
    "transactional_storage", "presenter_projection",
]
assert all(stage["owner"] and stage["output"] for stage in internal["stages"])
for op in operations:
    assert op["handler"] and op["services"] and op["storage"] and op["success_states"] and op["failures"]
    route = public["paths"][op["path"]][op["method"].lower()]
    assert route["operationId"] == op["id"]
    assert route["x-focusa-handler"] == op["handler"]
    assert route["x-focusa-failure-codes"] == op["failures"]
    parameters = {(row["name"], row["in"]): row for row in route["parameters"]}
    assert parameters[("X-Request-Id", "header")]["required"] is True
    if op["mutation"]:
        assert parameters[("Idempotency-Key", "header")]["required"] is True
        assert route["requestBody"]["required"] is True
    else:
        assert "requestBody" not in route

error_rows = errors["errors"]
error_codes = {row["code"] for row in error_rows}
assert error_codes == set(module.ERROR_DEFINITIONS)
assert error_codes == set(public["components"]["schemas"]["Error"]["properties"]["code"]["enum"])
assert {code for op in operations for code in op["failures"]} <= error_codes
assert len(error_codes) == len(error_rows)
assert all(row["public_message"] and row["safe_next_action"] for row in error_rows)

forbidden = set(internal["canonical_output"]["forbidden"])
output_properties = set(public["components"]["schemas"]["ActivationEnvelope"]["properties"])
assert forbidden.isdisjoint(output_properties)
assert set(internal["request_context"]["forbidden_caller_fields"]).isdisjoint({field for op in operations for field in op["input"]})
raw = "\n".join(path.read_text(encoding="utf-8") for path in (SOURCE, PUBLIC, INTERNAL, ERRORS))
assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)
assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw)
assert "full_license_key" not in output_properties

print(json.dumps({"schema": "focusa.spec152e.api_contract_validation.v1", "operations": len(operations), "errors": len(error_codes), "result": "passed"}, sort_keys=True))
