#!/usr/bin/env python3
"""Validate the governed, resumable Pi RPC AgentExecutionAdapter."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
registry = json.loads((BUNDLE / "operation-registry.json").read_text())
start_schema = json.loads((BUNDLE / "json-schema/focusa.agent_execution_start.request.v1.json").read_text())
result_schema = json.loads((BUNDLE / "json-schema/focusa.agent_execution_adapter_result.v1.json").read_text())
work_loop = (ROOT / "crates/focusa-api/src/routes/work_loop.rs").read_text()
server = (ROOT / "crates/focusa-api/src/server.rs").read_text()
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
go = (ROOT / "packages/generated/spec135/go/client.gen.go").read_text()

expected = {
    "/v1/work-loop/driver/start": "focusa.agent_execution.start",
    "/v1/work-loop/driver/prompt": "focusa.agent_execution.prompt",
    "/v1/work-loop/driver/abort": "focusa.agent_execution.abort",
    "/v1/work-loop/driver/stop": "focusa.agent_execution.stop",
}
for path, operation_id in expected.items():
    operation = openapi["paths"][path]["post"]
    assert operation["operationId"] == operation_id
    assert operation["x-focusa-idempotency"] is True
    assert operation["x-focusa-permissions"] == ["work-loop:write"]
    assert operation["x-focusa-result-envelope"] == "focusa.tool_result.v1"
    scope = {(item["name"], item["required"]) for item in operation["parameters"]}
    assert {("project_root", True), ("continuity_id", True)} <= scope

registry_ids = {item["operation_id"] for item in registry["operations"]}
assert set(expected.values()) <= registry_ids
assert "idempotency_key" in start_schema["required"]
for field in ("resume_session", "session_dir", "session_name", "workpoint_id", "idempotency_key"):
    assert field in start_schema["properties"]
assert {"schema", "status", "adapter", "session_id", "resumable", "authority", "tool_result"} <= set(result_schema["required"])

for marker in (
    'command.args(["--mode", "rpc"])', 'command.args(["--session", resume_session])',
    'command.args(["--session-dir", session_dir])', 'json!({"type":"abort"})',
    "terminate_pi_rpc_child", "WorkLoopScope", 'permissions.allows("work-loop:write")',
    "ensure_writer_claim", "agent_execution_tool_result", "idempotent_replay",
    "pi_rpc_execution_invocation_is_persisted_resumable_and_governed",
):
    assert marker in work_loop
assert 'args(["--mode", "rpc", "--no-session"])' not in work_loop
assert "pub idempotency_key: String" in server
assert 'operations["focusa.agent_execution.start"]' in ts
assert 'operations["focusa.agent_execution.abort"]' in ts
assert "func (c *Client) FocusaAgentExecutionStart(" in go
assert "func (c *Client) FocusaAgentExecutionStop(" in go

print("Spec 135 Pi RPC AgentExecutionAdapter: PASS (governed, resumable, idempotent, cancellation-aware)")
