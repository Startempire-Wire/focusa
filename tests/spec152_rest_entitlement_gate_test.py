#!/usr/bin/env python3
"""Build-independent acceptance for exact REST feature/limit preflight gates."""

import importlib.util
import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
GENERATOR = ROOT / "scripts/generate-spec152-route-entitlement-table.py"
GENERATED = ROOT / "crates/focusa-api/src/middleware/entitlement_routes.rs"
MIDDLEWARE = ROOT / "crates/focusa-api/src/middleware/entitlement.rs"
SERVER = ROOT / "crates/focusa-api/src/server.rs"

spec = importlib.util.spec_from_file_location("route_entitlements", GENERATOR)
module = importlib.util.module_from_spec(spec); assert spec.loader; spec.loader.exec_module(module)
assert GENERATED.read_text() == module.render(), "generated route entitlement table is stale"
requirements = module.requirements()
assert len(requirements) >= 100, "too few descriptor-bound routes are governed"
assert len({path for path, _, _ in requirements}) == len(requirements)
requirement_map = {path: (feature, bucket) for path, feature, bucket in requirements}
assert requirement_map["/v1/update/apply"][0] == "focusa.update.apply"
assert requirement_map["/v1/update/scheduler"][0] == "focusa.update.unattended"
assert requirement_map["/v1/export/run"][0] == "focusa.export.packaged"
assert "/v1/update/rollback" not in requirement_map

registry = yaml.safe_load((ROOT / "docs/contracts/spec152-feature-registry.v1.yaml").read_text())
features = {item["key"]: item["limit_bucket"] for item in registry["features"]}
for path, feature, bucket in requirements:
    assert feature in features, f"route {path} uses unknown feature {feature}"
    assert bucket == features[feature], f"route {path} limit bucket drifted"

middleware = MIDDLEWARE.read_text()
server = SERVER.read_text()
for marker in [
    "route_entitlement_denial(&state.license_guard,",
    "ENTITLEMENT_FEATURE_REQUIRED",
    "ENTITLEMENT_LIMIT_EXHAUSTED",
    "ENTITLEMENT_ROUTE_UNCLASSIFIED",
    ".features",
    ".limits",
]:
    assert marker in middleware, f"middleware missing {marker}"
assert middleware.index("route_entitlement_denial") < middleware.index("next.run(request).await", middleware.index("route_entitlement_denial"))
assert 'path.starts_with("/v1/device/pair/")' not in middleware, "pairing still bypasses entitlement"
assert 'path.starts_with("/v1/connect/")' not in middleware, "connection setup still bypasses entitlement"
assert "middleware::entitlement::entitlement_gate_layer" in server
assert re.search(r'route_scope_layer\).*?entitlement_gate_layer', server, re.DOTALL), "scope and entitlement middleware ordering drifted"

print(json.dumps({
    "schema": "focusa.rest_entitlement_gate_validation.v1",
    "exact_routes": len(requirements),
    "feature_gate": "pre_handler",
    "limit_gate": "pre_handler",
    "pairing_bypass": False,
    "result": "passed",
}, sort_keys=True))
