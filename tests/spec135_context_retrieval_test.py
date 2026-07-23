#!/usr/bin/env python3
"""SPEC135-C2 contract/static proof for bounded hybrid Context retrieval."""

import json
from pathlib import Path
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
core = (ROOT / "crates/focusa-core/src/runtime/context_retrieval.rs").read_text()
api = (ROOT / "crates/focusa-api/src/routes/context_sources.rs").read_text()
cargo = (ROOT / "crates/focusa-core/Cargo.toml").read_text()
registry = json.loads((BUNDLE / "operation-registry.json").read_text())
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
bindings = json.loads((BUNDLE / "ui-action-bindings.fixture.json").read_text())
request_schema = json.loads(
    (BUNDLE / "json-schema/focusa.context_retrieve.request.v1.json").read_text()
)
response_schema = json.loads(
    (BUNDLE / "json-schema/focusa.context_retrieve_response.v1.json").read_text()
)
scenario_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_scenario.v1.schema.json").read_text()
)
result_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_result.v1.schema.json").read_text()
)
scenario = json.loads(
    (BUNDLE / "uiai-eval.c2-context-retrieval.scenario.json").read_text()
)
result = json.loads((BUNDLE / "uiai-eval.c2-context-retrieval.result.json").read_text())
proof = json.loads((BUNDLE / "spec135-c2-context-retrieval-proof.json").read_text())
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
go = (ROOT / "packages/generated/spec135/go/client.gen.go").read_text()

assert 'sqlite-vec = "=0.1.7"' in cargo
assert 'context-vector-fastembed = ["dep:fastembed"]' in cargo
assert 'fastembed = { version = "=4.9.1", optional = true' in cargo
for marker in [
    "CREATE VIRTUAL TABLE IF NOT EXISTS context_chunks_fts USING fts5",
    "CREATE VIRTUAL TABLE IF NOT EXISTS context_embeddings USING vec0",
    "FOCUSA_CONTEXT_VECTOR_MODE",
    "fastembed provider is not built",
    "ContextCitation",
    "ContextContradictionCandidate",
    "reciprocal_rank_fusion",
    "MAX_CHUNKS_PER_SOURCE",
    "request.limit.clamp(1, 50)",
]:
    assert marker in core, marker
for marker in [
    "/v1/context/retrieve",
    "spawn_blocking",
    "canonical_sources",
    "receipt:context-retrieval",
]:
    assert marker in api, marker

operations = {
    operation["operation_id"]: operation for operation in registry["operations"]
}
op = operations["focusa.context.retrieve"]
assert op["path"] == "/v1/context/retrieve" and op["method"] == "POST"
assert op["contracts"]["input_schema_ref"] == "focusa.context_retrieve.request.v1"
assert op["contracts"]["output_schema_ref"] == "focusa.context_retrieve_response.v1"
assert (
    op["scope"]["project_scoped"]
    and op["scope"]["workstream_scoped"]
    and op["scope"]["attachment_scoped"]
)
assert any(
    binding["action_id"] == "focusa.context.retrieve"
    for binding in bindings["bindings"]
)
assert "/v1/context/retrieve" in openapi["paths"]
assert (
    openapi["paths"]["/v1/context/retrieve"]["post"]["operationId"]
    == "focusa.context.retrieve"
)

Draft202012Validator.check_schema(request_schema)
Draft202012Validator.check_schema(response_schema)
props = request_schema["properties"]
assert props["limit"]["maximum"] == 50 and props["query"]["maxLength"] == 2048
result_props = response_schema["properties"]["result"]["properties"]
assert result_props["hits"]["maxItems"] == 50
assert result_props["hits"]["items"]["properties"]["citation"]
assert (
    result_props["contradictions"]["items"]["properties"]["status"]["const"]
    == "candidate"
)
assert "focusa_context_retrieve_request_v1" in ts
assert "FocusaContextRetrieve" in go
Draft202012Validator(scenario_schema).validate(scenario)
Draft202012Validator(result_schema).validate(result)
assert result["status"] == "passed" and all(
    step["status"] == "passed" for step in result["step_results"]
)
assert proof["status"] == "verified" and proof["requirement_id"] == "SPEC135-C2"
assert proof["runtime_proof"]["embedding_dimensions"] == 384
assert (
    proof["runtime_proof"]["vector_absence_behavior"]
    == "deterministic lexical fallback"
)
assert "test:focusa-core:context-vector-fastembed" in proof["evidence_refs"]

print(
    f"Spec 135 C2 Context retrieval: PASS ({len(operations)} operations; FTS5 + sqlite-vec + optional fastembed; cited/scoped/bounded)"
)
