#!/usr/bin/env python3
"""Generate/check Spec 135B Project Context Artifact OpenAPI contracts."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OPENAPI = ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json"


def string(default: str | None = None, **extra):
    value = {"type": "string", **extra}
    if default is not None:
        value["default"] = default
    return value


def object_schema(properties, required=()):
    return {
        "additionalProperties": False,
        "properties": properties,
        "required": list(required),
        "type": "object",
    }


def artifact_schema():
    scope = object_schema(
        {"project_root": string(), "continuity_id": string()},
        ("project_root", "continuity_id"),
    )
    provenance = object_schema(
        {
            "connector_id": string(),
            "account_ref": string(),
            "author": string(),
            "source_url": string(),
            "page_or_message_ref": string(),
        },
        ("connector_id", "account_ref", "author", "source_url", "page_or_message_ref"),
    )
    classification = object_schema(
        {
            "sensitivity": string(),
            "confidentiality": string(),
            "retention_class": string(),
            "freshness_status": string(),
        },
        ("sensitivity", "confidentiality", "retention_class", "freshness_status"),
    )
    refs = {"items": string(), "maxItems": 32, "type": "array"}
    extraction = object_schema(
        {
            "status": string(),
            "diagnostic_refs": copy.deepcopy(refs),
            "extracted_claim_ids": copy.deepcopy(refs),
            "entity_refs": copy.deepcopy(refs),
            "date_refs": copy.deepcopy(refs),
            "task_refs": copy.deepcopy(refs),
            "contradiction_refs": copy.deepcopy(refs),
        },
        (
            "status",
            "diagnostic_refs",
            "extracted_claim_ids",
            "entity_refs",
            "date_refs",
            "task_refs",
            "contradiction_refs",
        ),
    )
    semantic = object_schema(
        {
            "domain_pack_refs": copy.deepcopy(refs),
            "candidate_object_refs": copy.deepcopy(refs),
            "candidate_link_refs": copy.deepcopy(refs),
            "verification_policy_refs": copy.deepcopy(refs),
        },
        (
            "domain_pack_refs",
            "candidate_object_refs",
            "candidate_link_refs",
            "verification_policy_refs",
        ),
    )
    return object_schema(
        {
            "schema": {"const": "focusa.project_context_artifact.v1", "type": "string"},
            "artifact_id": string(),
            "source_kind": string(),
            "source_ref": string(),
            "source_revision": string(),
            "title": string(),
            "mime_type": string(),
            "content_handle": string(),
            "content_sha256": string(pattern="^[a-f0-9]{64}$"),
            "created_at": string(format="date-time"),
            "observed_at": string(format="date-time"),
            "scope": scope,
            "provenance": provenance,
            "classification": classification,
            "extraction": extraction,
            "semantic": semantic,
            "duplicate_of_artifact_ref": {"nullable": True, "type": "string"},
        },
        (
            "schema",
            "artifact_id",
            "source_kind",
            "source_ref",
            "source_revision",
            "title",
            "mime_type",
            "content_handle",
            "content_sha256",
            "created_at",
            "observed_at",
            "scope",
            "provenance",
            "classification",
            "extraction",
            "semantic",
        ),
    )


def health_schema():
    refs = {"items": string(), "maxItems": 32, "type": "array"}
    return object_schema(
        {
            "status": string(),
            "adapter_id": string(),
            "message": string(),
            "read_write_posture": string(),
            "oauth_scopes": refs,
            "incremental_sync_method": string(),
            "cursor_state": {"nullable": True, "type": "string"},
            "rate_limit_posture": string(),
            "revocation_behavior": string(),
            "recovery_action": {"nullable": True, "type": "string"},
            "last_successful_sync": {"format": "date-time", "nullable": True, "type": "string"},
        },
        (
            "status",
            "adapter_id",
            "message",
            "read_write_posture",
            "oauth_scopes",
            "incremental_sync_method",
            "rate_limit_posture",
            "revocation_behavior",
        ),
    )


def generated(current):
    result = copy.deepcopy(current)
    schemas = result["components"]["schemas"]
    request = schemas["focusa_context_source_ingest_request_v1"]
    request["properties"]["source_kind"]["enum"] = [
        "markdown",
        "code",
        "pdf",
        "file",
        "web",
        "research",
        "connected",
        "focusa_native",
    ]
    optional_strings = (
        "connector_id",
        "account_ref",
        "author",
        "source_url",
        "page_or_message_ref",
        "sensitivity",
        "confidentiality",
        "retention_class",
        "freshness_status",
        "sync_cursor",
        "incremental_sync_method",
        "rate_limit_posture",
        "recovery_action",
    )
    for name in optional_strings:
        request["properties"][name] = string(maxLength=2048)
    for name in ("domain_pack_refs", "verification_policy_refs", "oauth_scopes"):
        request["properties"][name] = {
            "items": string(maxLength=256),
            "maxItems": 32,
            "type": "array",
        }

    artifact = artifact_schema()
    schemas["focusa_project_context_artifact_v1"] = artifact
    source = copy.deepcopy(
        schemas.get("focusa_context_source_record_v1")
        or schemas["focusa_context_source_ingest_result_v1"]["properties"]["source"]
    )
    source["properties"]["health"] = health_schema()
    source["properties"]["artifact"] = {
        "$ref": "#/components/schemas/focusa_project_context_artifact_v1"
    }
    required = source.setdefault("required", [])
    for name in ("health", "artifact"):
        if name not in required:
            required.append(name)
    schemas["focusa_context_source_record_v1"] = source
    source_ref = {"$ref": "#/components/schemas/focusa_context_source_record_v1"}
    for result_name in (
        "focusa_context_source_commit_result_v1",
        "focusa_context_source_ingest_result_v1",
    ):
        schemas[result_name]["properties"]["source"] = copy.deepcopy(source_ref)
    schemas["focusa_context_source_list_v1"]["properties"]["sources"]["items"] = copy.deepcopy(
        source_ref
    )
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    current = json.loads(OPENAPI.read_text())
    expected = generated(current)
    rendered = json.dumps(expected, indent=2) + "\n"
    existing = json.dumps(current, indent=2) + "\n"
    if args.write:
        OPENAPI.write_text(rendered)
    if args.check and rendered != existing:
        raise SystemExit("Spec 135B Context artifact OpenAPI contract is stale; run with --write")
    print(json.dumps({"status": "passed", "mode": "write" if args.write else "check"}))


if __name__ == "__main__":
    main()
