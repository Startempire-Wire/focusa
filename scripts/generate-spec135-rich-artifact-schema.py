#!/usr/bin/env python3
"""Spec 135C-1: generate the durable rich Workspace Artifact descriptor
JSON Schema and the artifact-kind registry with required Pi renderers and
fallbacks. All artifacts carry canonical before/after/evidence refs."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"
REGISTRY_PATH = ROOT / "docs/contracts/spec135-rich-artifact-renderer-registry.v1.json"
SCHEMA_PATH = SCHEMA_DIR / "focusa.workspace_artifact_descriptor.v1.json"

ARTIFACT_KINDS = [
    {"kind": "image", "primary": "image viewer with zoom, metadata, source, and evidence", "fallback": "artifact card + Open action"},
    {"kind": "markdown", "primary": "cited research/document reader", "fallback": "bounded text + handle"},
    {"kind": "dataset", "primary": "sortable/filterable table", "fallback": "schema/row summary + download/open"},
    {"kind": "diff", "primary": "workspace-specific change viewer", "fallback": "unified text diff"},
    {"kind": "browser_snapshot", "primary": "structured accessibility tree and refs", "fallback": "bounded JSON/text tree"},
    {"kind": "diagnostics", "primary": "console/network/error inspector", "fallback": "summarized findings + refs"},
    {"kind": "chart", "primary": "interactive chart where supported", "fallback": "table and static summary"},
    {"kind": "document", "primary": "document/PDF reader", "fallback": "extracted text + source page refs"},
    {"kind": "media", "primary": "bounded media viewer", "fallback": "metadata + external/open action"},
    {"kind": "fpv_session", "primary": "live UIAI FPV Work Surface/share", "fallback": "session status + share/open action"},
]

SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.workspace_artifact_descriptor.v1.json",
    "title": "Focusa Workspace Artifact Descriptor v1",
    "description": "Durable rich artifact descriptor produced by UIAI Engine and rendered by Mission Canvas / Pi clients. No client may silently discard an artifact because it cannot render the preferred format; fallback is required.",
    "type": "object",
    "required": [
        "schema", "artifact_id", "artifact_kind", "title",
        "before_ref", "after_ref", "evidence_refs",
        "project_root", "continuity_id", "session_origin",
        "freshness", "authority", "render_safe"
    ],
    "properties": {
        "schema": {"const": "focusa.workspace_artifact_descriptor.v1"},
        "artifact_id": {"type": "string", "minLength": 1},
        "artifact_kind": {
            "type": "string",
            "enum": [k["kind"] for k in ARTIFACT_KINDS],
        },
        "title": {"type": "string", "minLength": 1},
        "before_ref": {"type": "string"},
        "after_ref": {"type": "string"},
        "evidence_refs": {
            "type": "array",
            "items": {"type": "string", "minLength": 1},
        },
        "summary": {"type": "string"},
        "changes": {"type": "array", "items": {"type": "string"}},
        "citations": {
            "type": "array",
            "items": {
                "type": "object",
                "required": ["citation_ref", "source_origin"],
                "properties": {
                    "citation_ref": {"type": "string", "minLength": 1},
                    "source_origin": {"type": "string", "minLength": 1},
                    "authoritative": {"type": "boolean"},
                },
            },
        },
        "provenance": {
            "type": "object",
            "required": ["source_kind", "harvested_at"],
            "properties": {
                "source_kind": {"type": "string"},
                "harvested_at": {"type": "string", "minLength": 1},
                "uiai_session_ref": {"type": "string"},
                "browser_context_ref": {"type": "string"},
                "operator_id": {"type": "string"},
            },
        },
        "project_root": {"type": "string", "minLength": 1},
        "continuity_id": {"type": "string", "minLength": 1},
        "session_origin": {"type": "string", "minLength": 1},
        "attachment_id": {"type": "string"},
        "freshness": {"type": "string", "minLength": 1},
        "authority": {"type": "string", "minLength": 1},
        "vertical_dispatch": {
            "type": "string",
            "enum": ["General", "Software", "Legal", "Markets", "Research", "Custom"],
        },
        "render_safe": {"type": "boolean"},
        "redacted": {"type": "boolean"},
        "artifact_handle": {"type": "string"},
        "external_open_ref": {"type": "string"},
    },
    "additionalProperties": False,
}

REGISTRY = {
    "schema": "focusa.spec135.rich_artifact_renderer_registry.v1",
    "spec_ref": "docs/135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md#5-artifact-kinds-and-required-renderers",
    "canonical_state_law": (
        "No client owns or duplicates canonical artifact state. UIAI Engine "
        "produces descriptors; Focusa reducers own linkage and authority. "
        "Mission Canvas clients render presentation safely from canonical "
        "before/after/evidence refs."
    ),
    "fallback_rule": "No client may silently discard an artifact because it cannot render the preferred format.",
    "artifact_kinds": ARTIFACT_KINDS,
    "all_required_sources": [
        "canonical before/after/evidence refs",
        "provenance with source kind and harvested timestamp",
        "session/attachment/browser-context origin",
        "freshness",
        "authority marker",
        "render_safe flag",
    ],
}


def main() -> None:
    SCHEMA_DIR.mkdir(parents=True, exist_ok=True)
    SCHEMA_PATH.write_text(json.dumps(SCHEMA, indent=2) + "\n")
    REGISTRY_PATH.write_text(json.dumps(REGISTRY, indent=2) + "\n")
    print(f"Spec 135C-1 rich artifact schema generated: {len(ARTIFACT_KINDS)} kinds")
    print(f"  schema: docs/contracts/spec135/generated-contract-v1/json-schema/focusa.workspace_artifact_descriptor.v1.json")
    print(f"  registry: docs/contracts/spec135-rich-artifact-renderer-registry.v1.json")


if __name__ == "__main__":
    main()