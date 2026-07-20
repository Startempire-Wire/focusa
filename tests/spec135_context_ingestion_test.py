#!/usr/bin/env python3
"""Validate C1 Context ingestion contracts, canonical state, Docling, clients, and generated UI."""
from __future__ import annotations

import json
from pathlib import Path
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
registry = json.loads((BUNDLE / "operation-registry.json").read_text())
bindings = json.loads((BUNDLE / "ui-action-bindings.fixture.json").read_text())
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
proof_contract = json.loads((BUNDLE / "spec135-c1-context-ingestion-proof.json").read_text())
scenario_schema = json.loads((BUNDLE / "uiai.focusa_ui_eval_scenario.v1.schema.json").read_text())
result_schema = json.loads((BUNDLE / "uiai.focusa_ui_eval_result.v1.schema.json").read_text())
scenario = json.loads((BUNDLE / "uiai-eval.c1-context-ingestion.scenario.json").read_text())
result = json.loads((BUNDLE / "uiai-eval.c1-context-ingestion.result.json").read_text())
operations = {item["operation_id"]: item for item in registry["operations"]}
source_types = (ROOT / "crates/focusa-core/src/types.rs").read_text()
reducer = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
route = (ROOT / "crates/focusa-api/src/routes/context_sources.rs").read_text()
proof = (ROOT / "packages/a2ui-renderer/proof/context-ingest.ts").read_text()
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
go = (ROOT / "packages/generated/spec135/go/client.gen.go").read_text()

assert registry["operation_count"] == 59
assert bindings["binding_count"] == 59
for operation_id, method, path in (
    ("focusa.context.source.ingest", "POST", "/v1/context/sources/ingest"),
    ("focusa.context.adapter.docling.health", "GET", "/v1/context/adapters/docling/health"),
):
    operation = operations[operation_id]
    assert operation["method"] == method and operation["path"] == path and operation["canonical"]
    assert operation["scope"]["required_keys"] == ["project_root", "continuity_id", "attachment_id"]
    assert next(binding for binding in bindings["bindings"] if binding["action_id"] == operation_id)

operation = operations["focusa.context.source.ingest"]
assert operation["control"]["idempotency_required"]
assert operation["control"]["optimistic_concurrency_required"]
assert operation["control"]["receipt_required"]
assert operation["control"]["permission_scopes"] == ["context:write"]
assert operation["ui"]["allowed_in_generated_ui"]

for schema_id in (
    "focusa.context_source_ingest.request.v1", "focusa.context_source_ingest_result.v1",
    "focusa.context_adapter_health.request.v1", "focusa.context_adapter_health.v1",
):
    schema = json.loads((BUNDLE / "json-schema" / f"{schema_id}.json").read_text())
    Draft202012Validator.check_schema(schema)
    assert schema["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert openapi["paths"]["/v1/context/sources/ingest"]["post"]["operationId"] == "focusa.context.source.ingest"
assert openapi["paths"]["/v1/context/adapters/docling/health"]["get"]["operationId"] == "focusa.context.adapter.docling.health"

for marker in (
    "pub struct ContextSourceHealth", "pub source_locator: String", "pub source_revision: String",
    "pub mime_type: String", "pub adapter_id: String", "pub ingestion_status: String",
    "pub extraction_diagnostics: Vec<String>", "ContextSourceIngested",
):
    assert marker in source_types
for marker in (
    "Context source revision mismatch", "duplicate Context source ingestion",
    "state.context_sources[index] = source", "new Context source must start at revision 1",
):
    assert marker in reducer
for marker in (
    '"markdown" | "code" | "pdf"', 'format!("{base_url}/v1/convert/file")',
    '.part("files", part)', '.text("to_formats", "md")', 'bytes.starts_with(b"%PDF-")',
    "FOCUSA_DOCLING_BASE_URL", "Action::EmitEvent", "FocusaEvent::ContextSourceIngested",
    "expected_state_version", "idempotency_key", "source_revision", "evidence_ref", "receipt_ref",
):
    assert marker in route
for marker in (
    'operationId = "focusa.context.source.ingest"', 'healthOperationId = "focusa.context.adapter.docling.health"',
    'client.POST("/v1/context/sources/ingest"', 'client.GET("/v1/context/adapters/docling/health"',
    "FocusaSourceConnectorCard", "FocusaProgressStepper", "FocusaRecoveryCard",
    "content_base64: bytesToBase64(minimalPdf", "Evidence", "receipts=",
):
    assert marker in proof
assert '"/v1/context/sources/ingest"' in ts and "FocusaContextSourceIngest" in go
assert '"/v1/context/adapters/docling/health"' in ts and "FocusaContextAdapterDoclingHealth" in go
assert "playwright" not in proof.lower()
Draft202012Validator(scenario_schema).validate(scenario)
Draft202012Validator(result_schema).validate(result)
assert result["status"] == "passed" and all(step["status"] == "passed" for step in result["step_results"])
assert proof_contract["status"] == "verified"
assert proof_contract["critical_path"] == ["SPEC135-F12", "SPEC135-C1", "SPEC135-C2", "SPEC135-C3", "SPEC135-ALPHA1"]
assert proof_contract["runtime_proof"]["canonical_source_count"] == 3
assert proof_contract["runtime_proof"]["pdf_error_count"] == 0
assert proof_contract["docling_proof_environment"]["license"] == "MIT"
assert set(result["receipt_refs"]) == set(proof_contract["receipt_refs"])

print("Spec 135 C1 Context ingestion: PASS (Markdown/code/PDF, Docling health, incremental reducer, generated clients/UI)")
