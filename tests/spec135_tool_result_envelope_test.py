#!/usr/bin/env python3
"""Validate the canonical Spec 135 ToolResult success/error/recovery envelope."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
schema = json.loads((BUNDLE / "json-schema/focusa.tool_result.v1.json").read_text())
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
core = (ROOT / "crates/focusa-core/src/tool_result.rs").read_text()
middleware = (ROOT / "crates/focusa-api/src/middleware/error_envelope.rs").read_text()
pi = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()

assert schema["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert schema["x-focusa-schema-id"] == "focusa.tool_result.v1"
assert schema["additionalProperties"] is False
required = {
    "schema",
    "ok",
    "status",
    "canonical",
    "degraded",
    "summary",
    "retry",
    "side_effects",
    "evidence_refs",
    "next_tools",
}
assert required <= set(schema["required"])
assert set(schema["properties"]["status"]["enum"]) == {
    "accepted",
    "completed",
    "no_op",
    "blocked",
    "validation_rejected",
    "degraded",
    "offline",
    "error",
}
assert (
    "do_not_retry_unchanged"
    in schema["properties"]["retry"]["properties"]["posture"]["enum"]
)

component = openapi["components"]["schemas"]["focusa_tool_result_v1"]
assert component["required"] == schema["required"]
for path_item in openapi["paths"].values():
    for method, operation in path_item.items():
        if method not in {"get", "post", "put", "patch", "delete"}:
            continue
        assert operation["x-focusa-result-envelope"] == "focusa.tool_result.v1"
        default_ref = operation["responses"]["default"]["content"]["application/json"][
            "schema"
        ]["$ref"]
        assert default_ref == "#/components/schemas/focusa_tool_result_v1"

for marker in (
    "pub struct ToolResultV1",
    "pub enum FailureClass",
    "pub enum RetryPosture",
    "pub fn success",
    "pub fn failure",
):
    assert marker in core
assert "canonical_tool_result(status" in middleware
assert 'schema: "focusa.tool_result.v1"' in pi
assert 'schema: "focusa.tool_result.v1",' in pi
assert "focusa_tool_result_v1:" in ts

print("Spec 135 ToolResult envelope: PASS (core, API, Pi, TypeScript, portable contracts)")
