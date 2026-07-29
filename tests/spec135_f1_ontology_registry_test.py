#!/usr/bin/env python3
"""Spec 135F-1 domain-general ontology core proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md").read_text()
SCHEMA = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.ontology_registry.v2.json").read_text()
)
REGISTRY = json.loads(
    (ROOT / "docs/contracts/spec135-ontology-domain-general-registry.v1.json").read_text()
)

assert SCHEMA["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert SCHEMA["title"] == "Focusa Ontology Registry v2"
for field in ("object_types", "link_types", "action_types", "status_vocabularies", "evidence_kinds", "domain_packs", "v1_compatibility_projection", "hard_design_laws"):
    assert field in SCHEMA["required"], field

assert REGISTRY["schema"] == "focusa.ontology_registry.v2"
assert REGISTRY["acceptance_criteria"] == "Ontology supports all domains without software-only assumptions or duplicate canonical models."

# All 6 required domain packs present
pack_labels = {p["label"] for p in REGISTRY["domain_packs"]}
required_packs = {"General", "Software", "Legal", "Markets", "Research", "Custom"}
assert pack_labels == required_packs, pack_labels

# NO software-only assumptions — every pack declares this
for pack in REGISTRY["domain_packs"]:
    assert pack["software_only_assumption"] is False, f"{pack['label']} has software-only assumption"

# NO duplicate canonical models — all object type IDs unique
obj_ids = [ot["type_id"] for ot in REGISTRY["object_types"]]
assert len(obj_ids) == len(set(obj_ids)), "duplicate object type IDs"
link_ids = [lt["link_type_id"] for lt in REGISTRY["link_types"]]
assert len(link_ids) == len(set(link_ids)), "duplicate link type IDs"
action_ids = [at["action_type_id"] for at in REGISTRY["action_types"]]
assert len(action_ids) == len(set(action_ids)), "duplicate action type IDs"

# Typed definitions — not generic JSON
assert all("type_id" in ot and "domain_pack_id" in ot for ot in REGISTRY["object_types"])
assert all("link_type_id" in lt and "source_type_ids" in lt for lt in REGISTRY["link_types"])
assert all("action_type_id" in at and "target_type_ids" in at for at in REGISTRY["action_types"])
assert all("status_vocabulary_id" in sv for sv in REGISTRY["status_vocabularies"])

# Versioned built-in packs
assert all(p["versioned"] for p in REGISTRY["domain_packs"])
built_in = [p for p in REGISTRY["domain_packs"] if p["built_in"]]
assert len(built_in) == 5, "5 built-in packs expected"

# V1 compatibility projection explicit
assert REGISTRY["v1_compatibility_projection"]["enabled"] is True
assert REGISTRY["v1_compatibility_projection"]["legacy_names_preserved"] is True

# Candidate vs canonical structural separation law present
laws = REGISTRY["hard_design_laws"]
assert any("Candidate and canonical semantic state remain structurally distinct" in l for l in laws)
assert any("One core-owned semantic registry" in l for l in laws)
assert any("V1 behavior remains available through an explicit compatibility projection" in l for l in laws)

# Evidence kinds are domain-general (not test-only)
evidence = set(REGISTRY["evidence_kinds"])
assert "operator_confirmed" in evidence and "citation_verified" in evidence
assert "multiple_source" in evidence

for spec_text in (
    "One core-owned semantic registry",
    "domain-general",
    "competing registries",
    "software and non-software domains",
    "V1 Compatibility Projection",
    "Object type definition",
    "Link type definition",
    "Action type definition",
    "Status and lifecycle definition",
):
    assert spec_text in SPEC, spec_text

print("Spec 135 F1 domain-general ontology core: PASS")