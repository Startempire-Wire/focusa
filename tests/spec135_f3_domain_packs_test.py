#!/usr/bin/env python3
"""Spec 135F-3 complete domain packs proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md").read_text()
SCHEMA = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.domain_pack_manifest.v1.json").read_text()
)
CONTRACT = json.loads(
    (ROOT / "docs/contracts/spec135-domain-packs-complete.v1.json").read_text()
)

assert SCHEMA["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert SCHEMA["title"] == "Focusa Domain Pack Manifest v1"
for field in ("pack_id", "pack_version", "extends", "object_type_ids", "action_type_ids", "built_in"):
    assert field in SCHEMA["required"], field

assert CONTRACT["acceptance_criteria"] == "Each pack passes domain acceptance while sharing canonical ontology and authority."

pack_by_id = {p["pack_id"]: p for p in CONTRACT["packs"]}
required = CONTRACT["required_builtin_packs"]
for rid in required:
    pid = rid.rsplit("@", 1)[0]
    assert pid in pack_by_id, f"missing required pack: {pid}"

# All non-cognition packs extend cognition (shared canonical ontology)
for p in CONTRACT["packs"]:
    if p["pack_id"] == "focusa.core.cognition":
        continue
    assert "focusa.core.cognition@1" in p["extends"], f"{p['pack_id']} does not extend cognition"

# Each pack passes domain acceptance
for p in CONTRACT["packs"]:
    assert p["domain_acceptance"], f"{p['pack_id']} has no domain acceptance"

# Built-in packs have panels + renderers; Custom is declarative-only
built_in_content = [p for p in CONTRACT["packs"] if p["built_in"] and p["panel"]]
assert len(built_in_content) == 5, f"expected 5 built-in packs with panels, got {len(built_in_content)}"
custom = pack_by_id["focusa.custom"]
assert custom["custom"] is True
# Custom pack requires the 9 governance gates
assert len(custom["custom_pack_requirements"]) == 9
for req in ("schema validation", "operator approval", "conformance tests", "migration plan", "explicit fallback behavior"):
    assert req in custom["custom_pack_requirements"], req

# Domain-pack activation cannot change authority implicitly
assert "Domain-pack activation cannot change permission" in CONTRACT["shared_authority_law"]

for spec_text in (
    "Domain packs",
    "domain_pack_manifest.v1",
    "Required built-in packs",
    "focusa.core.cognition@1",
    "Composite projects may activate multiple compatible packs",
    "Custom packs require",
    "schema validation",
    "namespace ownership",
):
    assert spec_text in SPEC, spec_text

print("Spec 135 F3 complete domain packs: PASS")