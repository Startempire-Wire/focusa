#!/usr/bin/env python3
"""SPEC135-U2 static contract proof for bounded targeted Workspace invalidation."""

import json
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
workspace_schema = json.loads(
    (BUNDLE / "json-schema/focusa.workspace_event.v1.json").read_text()
)
stream_request = json.loads(
    (BUNDLE / "json-schema/focusa.events_stream.request.v1.json").read_text()
)
stream_schema = json.loads(
    (BUNDLE / "json-schema/focusa.stream_event.v1.json").read_text()
)
eval_scenario_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_scenario.v1.schema.json").read_text()
)
eval_result_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_result.v1.schema.json").read_text()
)
eval_scenario = json.loads(
    (BUNDLE / "uiai-eval.u2-workspace-live-refresh.scenario.json").read_text()
)
eval_result = json.loads(
    (BUNDLE / "uiai-eval.u2-workspace-live-refresh.result.json").read_text()
)
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
dag = json.loads((ROOT / "docs/contracts/spec135-delivery-dag.v1.yaml").read_text())
types = (ROOT / "crates/focusa-core/src/types.rs").read_text()
route = (ROOT / "crates/focusa-api/src/routes/workspace_artifacts.rs").read_text()
sse = (ROOT / "crates/focusa-api/src/routes/sse.rs").read_text()
ui = (ROOT / "packages/a2ui-renderer/proof/workspace-live-refresh.ts").read_text()
ui_html = (
    ROOT / "packages/a2ui-renderer/proof/workspace-live-refresh.html"
).read_text()
e2e = (ROOT / "tests/spec135_workspace_live_refresh_e2e_test.py").read_text()
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()

for schema in (
    workspace_schema,
    stream_request,
    stream_schema,
    eval_scenario_schema,
    eval_result_schema,
):
    Draft202012Validator.check_schema(schema)
Draft202012Validator(eval_scenario_schema).validate(eval_scenario)
Draft202012Validator(eval_result_schema).validate(eval_result)
assert workspace_schema["properties"]["schema"]["const"] == "focusa.workspace_event.v1"
assert workspace_schema["properties"]["semantic_authority"]["const"] is False
assert workspace_schema["properties"]["invalidate"]["maxItems"] == 16
assert "workspace_artifact_linked" in workspace_schema["properties"]["event"]["enum"]
assert {
    "project_root",
    "continuity_id",
    "attachment_id",
    "session_id",
    "work_surface_id",
} <= set(stream_request["properties"])

parameters = {
    (item["in"], item["name"])
    for item in openapi["paths"]["/v1/events/stream"]["get"]["parameters"]
}
assert {
    ("query", "cursor"),
    ("header", "Last-Event-ID"),
    ("query", "project_root"),
    ("query", "continuity_id"),
    ("query", "attachment_id"),
    ("query", "session_id"),
    ("query", "work_surface_id"),
} <= parameters

for marker in (
    "WorkspaceEventRecord",
    "WorkspaceEventType",
    "WorkspaceArtifactLinked",
    "semantic_authority",
):
    assert marker in types, marker
for marker in (
    'schema: "focusa.workspace_event.v1"',
    "WorkspaceEventType::WorkspaceArtifactLinked",
    "mission_canvas.surface_detail",
    "workspace.artifacts:",
    "semantic_authority: false",
):
    assert marker in route, marker
for marker in (
    'get("workspace_event")',
    "never enters the SSE payload",
    "stream_scope_matches",
    "query.attachment_id",
    "query.session_id",
    "query.work_surface_id",
    "record.sequence <= cursor",
    "RecvError::Lagged",
):
    assert marker in sse, marker
assert "must-not-stream" in sse and "inline_preview" in sse

for marker in (
    "new EventSource",
    "processedEventIds",
    "lastCursor",
    "surfaceARenders",
    "surfaceBRenders",
    "polling_fallback",
    "waitForSurfaceRender(2)",
    "unrelated Work Surface was invalidated",
    "event.payload.semantic_authority !== false",
    "exact Work Surface artifact read",
):
    assert marker in ui, marker
assert "Targeted Workspace live refresh" in ui_html
for marker in (
    "reconnect replay",
    "unrelated suppression",
    "semantic_authority",
    "inline_preview",
    "stream_url(base, SCOPE_A, first_cursor)",
):
    assert marker in e2e, marker

alpha1 = next(
    item
    for item in dag["critical_path_contract"]["slices"]
    if item["slice_id"] == "SPEC135-ALPHA1"
)
assert ["SPEC135-F12", "SPEC135-U1", "SPEC135-U2", "SPEC135-ALPHA1"] in alpha1[
    "feeder_paths"
]
assert "focusa_workspace_event_v1" in ts
for field in (
    "project_root",
    "continuity_id",
    "attachment_id",
    "session_id",
    "work_surface_id",
):
    assert field in ts

print(
    "Spec 135 U2 Workspace live refresh: PASS (bounded event, exact filters, cursor replay, targeted generated UI)"
)
