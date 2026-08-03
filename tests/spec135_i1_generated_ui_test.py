#!/usr/bin/env python3
"""Spec 135I-1 generated UI protocol and permanent renderer proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md").read_text()
SCHEMA = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.generated_ui_component.v1.json").read_text()
)
CONTRACT = json.loads(
    (ROOT / "docs/contracts/spec135-generated-ui-protocol.v1.json").read_text()
)
CATALOG = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/a2ui-catalog.json").read_text()
)

assert SCHEMA["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert SCHEMA["title"] == "Focusa Generated UI Component v1"
for field in ("component_id", "component_kind", "renderer", "bound_operation_id", "bound_scope", "fallback_summary", "version", "versioning", "validation_status"):
    assert field in SCHEMA["required"], field

# Permanent renderer is Lit, not a second Svelte renderer
assert SCHEMA["properties"]["renderer"]["enum"][0] == "lit"
assert CONTRACT["permanent_renderer"] == "lit"
assert "MUST NOT build a complete second Svelte A2UI renderer" in CONTRACT["permanent_renderer_law"]

# Governed operation binding enforcement
gov = CONTRACT["governed_operation_binding"]
for law in ("Every action resolves to a generated typed Focusa operation and exact scope",
            "Generated components can invoke only registered governed operations",
            "Unsupported actions render disabled",
            "Preview required for any component bound to a side-effect operation"):
    assert any(law in line for line in gov), law

# Malformed UI fails safely
vf = CONTRACT["validation_and_fallback"]
for fallback in ("Malformed manifests fail closed with validation_status=malformed",
                 "Fallback component renders bounded summary, never raw stack trace",
                 "fallback_summary is plain language"):
    assert any(fallback in line for line in vf), fallback

# AG-UI adapter MUST NOT own canonical state
assert any("AG-UI adapter MUST NOT own canonical state" in l for l in CONTRACT["versioning"])

# a2ui catalog exists and uses lit renderer
assert CATALOG["schema"] == "focusa.a2ui_catalog.v1"
assert CATALOG["renderer"] == "lit"
assert CATALOG["protocol_version"] == "v0.9"

assert "permanent web renderer" in SPEC or "permanent Lit renderer" in SPEC
assert "A2UI v0.9.1" in SPEC or "A2UI" in SPEC
assert "@a2ui/lit/v0_9" in SPEC
assert "The maintained Lit renderer is the permanent A2UI renderer" in SPEC
assert "AG-UI is a required compatibility adapter" in SPEC or "AG-UI is a required external compatibility adapter" in SPEC

print("Spec 135 I1 generated UI protocol and permanent renderer: PASS")