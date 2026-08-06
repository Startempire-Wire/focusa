#!/usr/bin/env python3
"""Generate OpenAPI, TypeScript, capability, UI, and compatibility artifacts for Spec 135."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/contracts/spec135/mission-canvas-v1"
REGISTRY_PATH = OUT / "operation-registry.json"
BUNDLE_PATH = ROOT / "schemas/spec135/mission-canvas/composition-bundle.v1.schema.json"


def load() -> tuple[dict[str, Any], dict[str, Any]]:
    return json.loads(REGISTRY_PATH.read_text()), json.loads(BUNDLE_PATH.read_text())


def json_text(value: Any) -> str:
    return json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n"


def schema_component_ref(name: str, *, request: bool = False) -> dict[str, Any]:
    base = name.removesuffix("[]")
    if base.startswith("focusa."):
        if request:
            # Operation-specific request schemas remain open until their owning
            # API publishes a named DTO, but the canonical Workstream owner is
            # still explicit instead of being hidden in a legacy scope blob.
            return {
                "type": "object",
                "additionalProperties": True,
                "required": ["workstream"],
                "properties": {
                    "workstream": {"$ref": "#/components/schemas/WorkstreamKey"},
                    "continuity_id": {"anyOf": [{"$ref": "#/components/schemas/ContinuityId"}, {"type": "null"}]},
                    "attachment": {"anyOf": [{"$ref": "#/components/schemas/AttachmentKey"}, {"type": "null"}]},
                    "workspace_binding_id": {"anyOf": [{"$ref": "#/components/schemas/WorkspaceBindingId"}, {"type": "null"}]},
                    "runtime_object": {"anyOf": [{"$ref": "#/components/schemas/RuntimeObjectRef"}, {"type": "null"}]},
                    "work_surface_id": {"anyOf": [{"$ref": "#/components/schemas/WorkSurfaceId"}, {"type": "null"}]},
                },
                "x-focusa-schema-ref": base,
            }
        return {"type": "object", "additionalProperties": True, "x-focusa-schema-ref": base}
    return {"$ref": f"../../../../schemas/spec135/mission-canvas/composition-bundle.v1.schema.json#/$defs/{base}"}


def openapi(registry: dict[str, Any], bundle: dict[str, Any]) -> dict[str, Any]:
    paths: dict[str, Any] = {}
    for entry in registry["operations"]:
        parameters = []
        for parameter in re.findall(r"\{([^}]+)\}", entry["path"]):
            parameters.append({"name": parameter, "in": "path", "required": True, "schema": {"type": "string"}})
        operation: dict[str, Any] = {
            "operationId": entry["operation_id"],
            "tags": ["Mission Canvas"],
            "summary": entry["operation_id"].removeprefix("focusa.mission_canvas.").replace("_", " ").replace(".", " ").title(),
            "x-focusa-availability": entry["availability"],
            "x-focusa-implementation-phase": entry["implementation_phase"],
            "x-focusa-permissions": entry["permissions_required"],
            "x-focusa-scope-keys": entry["authority_chain"],
            "x-focusa-scope-required": entry["scope_required"],
            "x-focusa-scope-optional": entry["scope_optional"],
            "x-focusa-idempotency-required": entry["requires_idempotency_key"],
            "x-focusa-if-match-revision-required": entry["requires_if_match_revision"],
            "x-focusa-receipt-required": entry["receipt_required"],
            "parameters": parameters,
            "responses": {
                "200": {
                    "description": "Contract response",
                    "content": {"application/json": {"schema": schema_component_ref(entry["response_schema_ref"])}},
                },
                "409": {"description": "Scope, revision, or ownership conflict"},
                "422": {"description": "Contract validation failure"},
            },
        }
        if entry["method"] != "GET":
            operation["requestBody"] = {
                "required": True,
                "content": {"application/json": {"schema": schema_component_ref(entry["request_schema_ref"], request=True)}},
            }
        paths.setdefault(entry["path"], {})[entry["method"].lower()] = operation
    components = {
        name: {"$ref": f"../../../../schemas/spec135/mission-canvas/composition-bundle.v1.schema.json#/$defs/{name}"}
        for name in sorted(bundle["$defs"])
    }
    return {
        "openapi": "3.0.3",
        "info": {"title": "Focusa Mission Canvas API", "version": "1.0.0"},
        "servers": [{"url": "/"}],
        "paths": paths,
        "components": {
            "securitySchemes": {"bearerAuth": {"type": "http", "scheme": "bearer"}},
            "schemas": components,
        },
        "security": [{"bearerAuth": []}],
        "x-focusa-contract-availability": "contract_defined",
    }


def ts_type(schema: dict[str, Any]) -> str:
    if "$ref" in schema:
        return schema["$ref"].split("/")[-1]
    if "const" in schema:
        return json.dumps(schema["const"])
    if "enum" in schema:
        return " | ".join(json.dumps(item) for item in schema["enum"])
    if "anyOf" in schema:
        return " | ".join(ts_type(item) for item in schema["anyOf"])
    if "oneOf" in schema:
        return " | ".join(ts_type(item) for item in schema["oneOf"])
    kind = schema.get("type")
    if isinstance(kind, list):
        return " | ".join(ts_type({**schema, "type": item}) for item in kind)
    if kind == "string":
        return "string"
    if kind in {"integer", "number"}:
        return "number"
    if kind == "boolean":
        return "boolean"
    if kind == "null":
        return "null"
    if kind == "array":
        return f"Array<{ts_type(schema.get('items', {}))}>"
    if kind == "object" or "properties" in schema:
        required = set(schema.get("required", []))
        fields = []
        for name, child in sorted(schema.get("properties", {}).items()):
            optional = "" if name in required else "?"
            fields.append(f"  {json.dumps(name)}{optional}: {ts_type(child)};")
        return "{\n" + "\n".join(fields) + "\n}"
    return "unknown"


def typescript_types(bundle: dict[str, Any]) -> str:
    lines = ["// Generated by generate-spec135-mission-canvas-derived-contracts.py. Do not edit.", ""]
    for name, schema in sorted(bundle["$defs"].items()):
        lines.append(f"export type {name} = {ts_type(schema)};")
        lines.append("")
    return "\n".join(lines)


def method_name(operation_id: str) -> str:
    parts = operation_id.removeprefix("focusa.mission_canvas.").split(".")
    return parts[0] + "".join(part[:1].upper() + part[1:] for part in parts[1:])


def response_type(schema_ref: str) -> str:
    if schema_ref.endswith("[]"):
        base = schema_ref[:-2]
        return f"Array<{base}>" if not base.startswith("focusa.") else "Array<unknown>"
    return "unknown" if schema_ref.startswith("focusa.") else schema_ref


def typescript_client(registry: dict[str, Any]) -> str:
    used = sorted({entry["response_schema_ref"].removesuffix("[]") for entry in registry["operations"] if not entry["response_schema_ref"].startswith("focusa.")})
    imports = sorted(set(used) | {"WorkstreamAuthorityContext"})
    lines = [
        "// Generated by generate-spec135-mission-canvas-derived-contracts.py. Do not edit.",
        f"import type {{ {', '.join(imports)} }} from './mission-canvas-types.generated';",
        "",
        "/** Every generated operation input is Workstream-bound.  Operation-specific",
        " * fields are generated from their published schema/registry entry; they do",
        " * not replace the canonical identity context. */",
        "export type MissionCanvasOperationInput = WorkstreamAuthorityContext & Record<string, unknown>;",
        "",
        "export interface MissionCanvasTransport {",
        "  request<T>(operationId: string, input: MissionCanvasOperationInput): Promise<T>;",
        "}",
        "",
        "export class MissionCanvasClient {",
        "  constructor(private readonly transport: MissionCanvasTransport) {}",
    ]
    for entry in registry["operations"]:
        lines.extend(
            [
                "",
                f"  {method_name(entry['operation_id'])}(input: MissionCanvasOperationInput): Promise<{response_type(entry['response_schema_ref'])}> {{",
                f"    return this.transport.request<{response_type(entry['response_schema_ref'])}>({json.dumps(entry['operation_id'])}, input);",
                "  }",
            ]
        )
    lines.extend(["}", ""])
    return "\n".join(lines)


def typescript_validators(bundle: dict[str, Any]) -> str:
    required = {name: schema.get("required", []) for name, schema in bundle["$defs"].items() if schema.get("required")}
    allowed = {name: sorted(schema.get("properties", {})) for name, schema in bundle["$defs"].items() if schema.get("properties") and schema.get("additionalProperties") is False}
    authority_schemas = sorted(
        name for name, schema in bundle["$defs"].items() if "workstream" in schema.get("required", [])
    )
    return "\n".join(
        [
            "// Generated structural and identity validators; JSON Schema remains canonical. Do not edit.",
            f"const REQUIRED: Record<string, readonly string[]> = {json.dumps(required, sort_keys=True)};",
            f"const ALLOWED: Record<string, readonly string[]> = {json.dumps(allowed, sort_keys=True)};",
            f"const AUTHORITY_SCHEMAS = new Set({json.dumps(authority_schemas)});",
            "",
            "export interface ValidationResult { valid: boolean; errors: string[]; }",
            "",
            "function record(value: unknown): Record<string, unknown> | undefined {",
            "  return typeof value === 'object' && value !== null && !Array.isArray(value) ? value as Record<string, unknown> : undefined;",
            "}",
            "",
            "function stable(value: unknown): string {",
            "  if (Array.isArray(value)) return `[${value.map(stable).join(',')}]`;",
            "  const object = record(value);",
            "  if (!object) return JSON.stringify(value) ?? String(value);",
            "  return `{${Object.keys(object).sort().map((key) => `${JSON.stringify(key)}:${stable(object[key])}`).join(',')}}`;",
            "}",
            "",
            "export function sameWorkstreamKey(left: unknown, right: unknown): boolean {",
            "  return stable(left) === stable(right);",
            "}",
            "",
            "export function sameAttachmentKey(left: unknown, right: unknown): boolean {",
            "  return stable(left) === stable(right);",
            "}",
            "",
            "export function sameWorkstreamAuthorityContext(left: unknown, right: unknown): boolean {",
            "  const a = record(left);",
            "  const b = record(right);",
            "  if (!a || !b || !sameWorkstreamKey(a.workstream, b.workstream)) return false;",
            "  return stable(a.continuity_id ?? null) === stable(b.continuity_id ?? null)",
            "    && sameAttachmentKey(a.attachment ?? null, b.attachment ?? null)",
            "    && stable(a.workspace_binding_id ?? null) === stable(b.workspace_binding_id ?? null)",
            "    && stable(a.runtime_object ?? null) === stable(b.runtime_object ?? null)",
            "    && stable(a.work_surface_id ?? null) === stable(b.work_surface_id ?? null);",
            "}",
            "",
            "function validateWorkstreamKey(value: unknown, errors: string[], path: string): void {",
            "  const key = record(value);",
            "  const scope = record(key?.scope);",
            "  const scopeKey = record(scope?.scope_key);",
            "  if (!key) { errors.push(`invalid:${path}.workstream`); return; }",
            "  if (typeof key.workstream_id !== 'string' || key.workstream_id.length === 0) errors.push(`missing:${path}.workstream_id`);",
            "  if (!scope || (scope.scope_kind !== 'project' && scope.scope_kind !== 'host')) errors.push(`invalid:${path}.scope`);",
            "  if (!scopeKey || typeof scopeKey.scope_id !== 'string' || typeof scopeKey.root_path !== 'string' || typeof scopeKey.fingerprint !== 'string') errors.push(`invalid:${path}.scope_key`);",
            "}",
            "",
            "function validateAuthority(value: Record<string, unknown>, errors: string[]): void {",
            "  validateWorkstreamKey(value.workstream, errors, 'workstream');",
            "  const attachment = value.attachment === null ? undefined : record(value.attachment);",
            "  if (value.attachment !== undefined && value.attachment !== null && !attachment) errors.push('invalid:attachment');",
            "  if (attachment) {",
            "    validateWorkstreamKey(attachment.workstream, errors, 'attachment');",
            "    for (const field of ['instance_id', 'session_id', 'attachment_id', 'workspace_binding_id']) if (typeof attachment[field] !== 'string' || attachment[field].length === 0) errors.push(`missing:attachment.${field}`);",
            "    if (attachment.continuity_id !== undefined && attachment.continuity_id !== null && (typeof attachment.continuity_id !== 'string' || attachment.continuity_id.length === 0)) errors.push('invalid:attachment.continuity_id');",
            "    if (!sameWorkstreamKey(attachment.workstream, value.workstream)) errors.push('foreign:attachment.workstream');",
            "    if (value.continuity_id !== undefined && value.continuity_id !== null && attachment.continuity_id !== undefined && attachment.continuity_id !== null && value.continuity_id !== attachment.continuity_id) errors.push('mismatch:continuity_id');",
            "    if (value.workspace_binding_id !== undefined && value.workspace_binding_id !== null && value.workspace_binding_id !== attachment.workspace_binding_id) errors.push('mismatch:workspace_binding_id');",
            "  }",
            "  if (value.continuity_id !== undefined && value.continuity_id !== null && (typeof value.continuity_id !== 'string' || value.continuity_id.length === 0)) errors.push('invalid:continuity_id');",
            "  if (value.workspace_binding_id !== undefined && value.workspace_binding_id !== null && (typeof value.workspace_binding_id !== 'string' || value.workspace_binding_id.length === 0)) errors.push('invalid:workspace_binding_id');",
            "  if (value.work_surface_id !== undefined && value.work_surface_id !== null && (typeof value.work_surface_id !== 'string' || value.work_surface_id.length === 0)) errors.push('invalid:work_surface_id');",
            "  const runtime = value.runtime_object === null ? undefined : record(value.runtime_object);",
            "  if (value.runtime_object !== undefined && value.runtime_object !== null && (!runtime || typeof runtime.runtime_kind !== 'string' || typeof runtime.runtime_id !== 'string')) errors.push('invalid:runtime_object');",
            "}",
            "",
            "export function validateMissionCanvasContract(schemaName: string, value: unknown): ValidationResult {",
            "  const errors: string[] = [];",
            "  const object = record(value);",
            "  if (!object) return { valid: false, errors: ['expected object'] };",
            "  for (const field of REQUIRED[schemaName] ?? []) if (!(field in object)) errors.push(`missing:${field}`);",
            "  const allowed = ALLOWED[schemaName];",
            "  if (allowed) for (const field of Object.keys(object)) if (!allowed.includes(field)) errors.push(`unknown:${field}`);",
            "  if (schemaName === 'WorkstreamKey') validateWorkstreamKey(object, errors, '');",
            "  if (schemaName === 'AttachmentKey') {",
            "    validateWorkstreamKey(object.workstream, errors, 'attachment');",
            "    for (const field of ['instance_id', 'session_id', 'attachment_id', 'workspace_binding_id']) if (typeof object[field] !== 'string' || object[field].length === 0) errors.push(`missing:attachment.${field}`);",
            "  }",
            "  if (AUTHORITY_SCHEMAS.has(schemaName)) validateAuthority(object, errors);",
            "  return { valid: errors.length === 0, errors };",
            "}",
            "",
        ]
    )


def capability_snapshot(registry: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema": "focusa.mission_canvas.capability_snapshot.v1",
        "contract_version": registry["registry_version"],
        "runtime_promoted": any(entry["availability"] == "available" for entry in registry["operations"]),
        "all_operations_promoted": all(entry["availability"] == "available" for entry in registry["operations"]),
        "operations": [
            {
                "operation_id": entry["operation_id"],
                "status": entry["availability"],
                "reason": (
                    "P03 runtime handler promoted"
                    if entry["availability"] == "available"
                    else "Contract defined; runtime handler not yet promoted"
                ),
                "required_permissions": entry["permissions_required"],
            }
            for entry in registry["operations"]
        ],
    }


def ui_bindings(registry: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema": "focusa.mission_canvas.ui_action_bindings.v1",
        "bindings": [
            {
                "action_id": "action:" + entry["operation_id"],
                "operation_id": entry["operation_id"],
                "enabled": entry["availability"] == "available",
                "disabled_reason": None if entry["availability"] == "available" else "runtime_not_promoted",
                "confirmation": entry["confirmation"],
            }
            for entry in registry["operations"]
            if entry["generated_ui_eligible"]
        ],
    }


def client_parity_matrix(registry: dict[str, Any]) -> dict[str, Any]:
    clients = ["core_api", "pi_extension", "rich_host_typescript", "cli", "uiai_engine_cockpit", "menubar"]
    rows = []
    for operation in registry["operations"]:
        for client in clients:
            if client in {"core_api", "pi_extension", "rich_host_typescript"}:
                support = "full"
            elif client == "uiai_engine_cockpit" and operation["mode"] in {"read", "stream"}:
                support = "read_only"
            else:
                support = "unsupported"
            rows.append({
                "operation_id": operation["operation_id"],
                "client": client,
                "support": support,
                "schema_ref": operation["response_schema_ref"],
                "evidence_ref": "tests/spec135_mission_canvas_generated_parity_test.py",
            })
    return {
        "schema": "focusa.mission_canvas.client_parity_matrix.v1",
        "operation_count": len(registry["operations"]),
        "client_count": len(clients),
        "rows": rows,
    }


def implementation_proof_matrix(registry: dict[str, Any], bundle: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema": "focusa.mission_canvas.implementation_proof_matrix.v1",
        "layers": [
            {"layer":"contracts","implementation":"schemas/spec135/mission-canvas","proof":"tests/spec135_resolved_projection_contract_test.py"},
            {"layer":"core","implementation":"crates/focusa-core/src/mission_canvas","proof":"cargo test -p focusa-core mission_canvas --lib"},
            {"layer":"api","implementation":"crates/focusa-api/src/routes/mission_canvas.rs","proof":"cargo test -p focusa-api mission_canvas::tests"},
            {"layer":"pi_terminal","implementation":"apps/pi-extension/src/mission-canvas-view.ts + apps/pi-extension/src/mission-canvas-shell.ts","proof":"npm run test:mission-canvas"},
            {"layer":"uiai_fixture","implementation":"apps/pi-extension/tests/mission-canvas-uiai-server.mjs","proof":"node tests/uiai-eval-harness.test.mjs"},
        ],
        "operation_ids": [entry["operation_id"] for entry in registry["operations"]],
        "schema_definition_count": len(bundle["$defs"]),
        "known_blockers": [],
    }


def sha256(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def outputs() -> dict[Path, str]:
    registry, bundle = load()
    openapi_text = json_text(openapi(registry, bundle))
    types_text = typescript_types(bundle)
    client_text = typescript_client(registry)
    validators_text = typescript_validators(bundle)
    capability_text = json_text(capability_snapshot(registry))
    ui_text = json_text(ui_bindings(registry))
    client_parity_text = json_text(client_parity_matrix(registry))
    proof_matrix_text = json_text(implementation_proof_matrix(registry, bundle))
    base: dict[Path, str] = {
        OUT / "openapi-3.0.3.json": openapi_text,
        OUT / "typescript/mission-canvas-types.generated.ts": types_text,
        OUT / "typescript/mission-canvas-client.generated.ts": client_text,
        OUT / "typescript/mission-canvas-validators.generated.ts": validators_text,
        OUT / "capability-snapshot.json": capability_text,
        OUT / "ui-action-bindings.json": ui_text,
        OUT / "client-parity-matrix.json": client_parity_text,
        OUT / "implementation-proof-matrix.json": proof_matrix_text,
    }
    lock = {
        "schema": "focusa.mission_canvas.compatibility_lock.v1",
        "protocol_version": "1.0.0",
        "schema_bundle_sha256": sha256(BUNDLE_PATH.read_text()),
        "operation_registry_sha256": sha256(REGISTRY_PATH.read_text()),
        "derived_artifacts": {str(path.relative_to(OUT)): "sha256:" + sha256(text) for path, text in sorted(base.items(), key=lambda item: str(item[0]))},
    }
    lock_text = json_text(lock)
    handshake = {
        "schema": "focusa.mission_canvas.protocol_handshake.v1",
        "protocol_version": "1.0.0",
        "minimum_client_version": "1.0.0",
        "schema_bundle_digest": "sha256:" + lock["schema_bundle_sha256"],
        "operation_registry_digest": "sha256:" + lock["operation_registry_sha256"],
        "required_scope_keys": ["workstream"],
        "authority_chain": registry["operations"][0]["authority_chain"],
        "runtime_promotion_required": not all(
            entry["availability"] == "available" for entry in registry["operations"]
        ),
        "compatibility_lock_ref": "compatibility-lock.json",
    }
    base[OUT / "compatibility-lock.json"] = lock_text
    base[OUT / "protocol-handshake.json"] = json_text(handshake)
    return base


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = outputs()
    if args.check:
        for path, expected in generated.items():
            assert path.exists(), f"missing generated artifact: {path}"
            assert path.read_text() == expected, f"stale generated artifact: {path}"
        print(f"Spec 135 Mission Canvas derived contracts: PASS ({len(generated)} artifacts)")
        return
    for path, text in generated.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        print(f"Generated {path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
