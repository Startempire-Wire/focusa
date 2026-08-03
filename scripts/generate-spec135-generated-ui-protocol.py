#!/usr/bin/env python3
"""Spec 135I-1: generate the durable generated-UI protocol contract with
governed operation binding, permanent renderer, component registry,
validation, fallback, and versioning. Generated components can invoke only
registered governed operations; malformed UI fails safely."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"
SCHEMA_PATH = SCHEMA_DIR / "focusa.generated_ui_component.v1.json"
CONTRACT_PATH = ROOT / "docs/contracts/spec135-generated-ui-protocol.v1.json"

SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.generated_ui_component.v1.json",
    "title": "Focusa Generated UI Component v1",
    "description": "A validated generated UI component bound to a governed Focusa operation. Malformed manifests fail closed; unsupported actions render disabled.",
    "type": "object",
    "required": [
        "schema", "component_id", "component_kind", "renderer",
        "bound_operation_id", "bound_scope", "fallback_summary",
        "version", "versioning", "validation_status",
    ],
    "properties": {
        "schema": {"const": "focusa.generated_ui_component.v1"},
        "component_id": {"type": "string", "minLength": 1},
        "component_kind": {"type": "string", "enum": ["surface", "action", "panel", "card", "dialog", "form"]},
        "renderer": {"type": "string", "enum": ["lit", "svelte-custom-element", "pi-tui-text", "markdown-fallback"]},
        "bound_operation_id": {"type": "string", "minLength": 1},
        "bound_scope": {"type": "object", "required": ["project_root", "continuity_id"]},
        "bound_action_prototype": {},
        "preview_required": {"type": "boolean"},
        "fallback_summary": {"type": "string", "minLength": 1},
        "fallback_component_id": {"type": "string", "minLength": 1},
        "version": {"type": "string", "minLength": 1},
        "versioning": {"type": "object", "required": ["major", "minor", "patch", "lineage"], "properties": {"major": {"type": "integer"}, "minor": {"type": "integer"}, "patch": {"type": "integer"}, "lineage": {"type": "array"}}},
        "validation_status": {"type": "string", "enum": ["valid", "malformed", "fallback_required", "disabled"]},
        "governed_operation_law": {"const": "Generated components can invoke only registered governed operations."},
    },
    "additionalProperties": False,
}

contract = {
    "schema": "focusa.spec135.generated_ui_protocol.v1",
    "spec_ref": "docs/135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md",
    "acceptance_criteria": "Generated components can invoke only registered governed operations and malformed UI fails safely.",
    "permanent_renderer": "lit",
    "permanent_renderer_law": "The maintained Lit renderer is the permanent A2UI renderer. Focusa MUST NOT build a complete second Svelte A2UI renderer, duplicate the A2UI message processor, duplicate SurfaceModel, or duplicate A2UI data binding.",
    "component_registry_ref": "docs/contracts/spec135/generated-contract-v1/a2ui-catalog.json",
    "governed_operation_binding": [
        "Every action resolves to a generated typed Focusa operation and exact scope",
        "Generated components can invoke only registered governed operations",
        "Unsupported actions render disabled — never enabled and discoverable-as-broken",
        "Preview required for any component bound to a side-effect operation",
        "Bound scope uses project_root + continuity_id",
    ],
    "validation_and_fallback": [
        "Malformed manifests fail closed with validation_status=malformed",
        "Fallback component renders bounded summary, never raw stack trace or IDs",
        "fallback_summary is plain language — no raw diagnostics in the default flow",
        "Validation status is explicit: valid | malformed | fallback_required | disabled",
    ],
    "versioning": [
        "Every component carries version and major.minor.patch lineage",
        "Migration preserves history; unsupported versions route to fallback",
        "AG-UI adapter MUST NOT own canonical state, event history, approvals, tool authority, or persistence",
    ],
    "renderer_surface_kinds": ["surface", "action", "panel", "card", "dialog", "form"],
    "schema_path": "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.generated_ui_component.v1.json",
}


def main() -> None:
    SCHEMA_DIR.mkdir(parents=True, exist_ok=True)
    SCHEMA_PATH.write_text(json.dumps(SCHEMA, indent=2) + "\n")
    CONTRACT_PATH.write_text(json.dumps(contract, indent=2) + "\n")
    print("Spec 135I-1 generated UI protocol contract generated")


if __name__ == "__main__":
    main()