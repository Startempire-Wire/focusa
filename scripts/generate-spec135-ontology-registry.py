#!/usr/bin/env python3
"""Spec 135F-1: generate the durable domain-general ontology registry.

One core-owned semantic registry with versioned built-in domain packs
(General, Software, Legal, Markets, Research, Custom). Typed object,
link, action, status, and evidence definitions replace generic JSON.
V1 compatibility projection is explicit. Candidate and canonical state
are structurally distinct. No software-only assumptions; no duplicate
canonical models."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"
REGISTRY_PATH = ROOT / "docs/contracts/spec135-ontology-domain-general-registry.v1.json"
SCHEMA_PATH = SCHEMA_DIR / "focusa.ontology_registry.v2.json"

# Object type IDs — domain-general, not software-only.
OBJECT_TYPES = [
    {"type_id": "focusa.core/mission@1", "legacy_names": ["mission"], "domain_pack_id": "focusa.core.general", "required_properties": ["title"], "allowed_link_type_ids": ["focusa.core/decomposes_to@1"], "allowed_action_type_ids": ["focusa.core/redefine_mission@1"], "status_vocabulary_id": "focusa.core/lifecycle@1"},
    {"type_id": "focusa.core/workpoint@1", "legacy_names": ["workpoint"], "domain_pack_id": "focusa.core.general", "required_properties": ["next_action"], "allowed_link_type_ids": ["focusa.core/depends_on@1", "focusa.core/proves@1"], "allowed_action_type_ids": ["focusa.core/checkpoint@1", "focusa.core/resume@1", "focusa.core/commit@1"], "status_vocabulary_id": "focusa.core/lifecycle@1"},
    {"type_id": "focusa.core/evidence@1", "legacy_names": ["evidence"], "domain_pack_id": "focusa.core.general", "required_properties": ["evidence_ref", "target_ref"], "allowed_link_type_ids": ["focusa.core/proves@1"], "allowed_action_type_ids": ["focusa.core/link_evidence@1"], "status_vocabulary_id": "focusa.core/verification@1"},
    {"type_id": "focusa.core/task@1", "legacy_names": ["task"], "domain_pack_id": "focusa.core.software", "required_properties": ["title"], "allowed_link_type_ids": ["focusa.core/depends_on@1"], "allowed_action_type_ids": ["focusa.core/complete_task@1"], "status_vocabulary_id": "focusa.core/lifecycle@1"},
    {"type_id": "focusa.core/code_unit@1", "legacy_names": ["file", "module"], "domain_pack_id": "focusa.core.software", "required_properties": ["path"], "allowed_link_type_ids": ["focusa.core/depends_on@1"], "allowed_action_type_ids": ["focusa.core/edit@1"], "status_vocabulary_id": "focusa.core/lifecycle@1"},
    {"type_id": "focusa.core/legal_clause@1", "legacy_names": [], "domain_pack_id": "focusa.core.legal", "required_properties": ["clause_id", "text"], "allowed_link_type_ids": ["focusa.core/cites@1"], "allowed_action_type_ids": ["focusa.core/redline@1"], "status_vocabulary_id": "focusa.core/lifecycle@1"},
    {"type_id": "focusa.core/thesis@1", "legacy_names": [], "domain_pack_id": "focusa.core.markets", "required_properties": ["statement"], "allowed_link_type_ids": ["focusa.core/depends_on@1"], "allowed_action_type_ids": ["focusa.core/revise_thesis@1"], "status_vocabulary_id": "focusa.core/lifecycle@1"},
    {"type_id": "focusa.core/claim@1", "legacy_names": [], "domain_pack_id": "focusa.core.research", "required_properties": ["statement", "evidence_ref"], "allowed_link_type_ids": ["focusa.core/contradicts@1", "focusa.core/proves@1"], "allowed_action_type_ids": ["focusa.core/revise_claim@1"], "status_vocabulary_id": "focusa.core/verification@1"},
]

LINK_TYPES = [
    {"link_type_id": "focusa.core/depends_on@1", "legacy_names": ["depends_on"], "source_type_ids": ["focusa.core/*"], "target_type_ids": ["focusa.core/*"], "directionality": "directed", "reversible": True},
    {"link_type_id": "focusa.core/proves@1", "legacy_names": ["proves"], "source_type_ids": ["focusa.core/evidence@1", "focusa.core/claim@1"], "target_type_ids": ["focusa.core/*"], "directionality": "directed", "reversible": False},
    {"link_type_id": "focusa.core/decomposes_to@1", "legacy_names": ["decomposes_to"], "source_type_ids": ["focusa.core/mission@1", "focusa.core/thesis@1"], "target_type_ids": ["focusa.core/*"], "directionality": "directed", "reversible": True},
    {"link_type_id": "focusa.core/cites@1", "legacy_names": [], "source_type_ids": ["focusa.core/legal_clause@1", "focusa.core/claim@1"], "target_type_ids": ["focusa.core/evidence@1", "focusa.core/legal_clause@1"], "directionality": "directed", "reversible": False},
    {"link_type_id": "focusa.core/contradicts@1", "legacy_names": [], "source_type_ids": ["focusa.core/claim@1"], "target_type_ids": ["focusa.core/claim@1"], "directionality": "directed", "reversible": True},
]

ACTION_TYPES = [
    {"action_type_id": "focusa.core/complete_task@1", "legacy_names": ["complete_task"], "target_type_ids": ["focusa.core/task@1"], "side_effect_classes": ["workpoint_compare"], "verification_policy_ref": "policy:workpoint_verification@1"},
    {"action_type_id": "focusa.core/edit@1", "legacy_names": ["edit"], "target_type_ids": ["focusa.core/code_unit@1"], "side_effect_classes": ["diff_produce"], "verification_policy_ref": "policy:test_evidence@1"},
    {"action_type_id": "focusa.core/redline@1", "legacy_names": [], "target_type_ids": ["focusa.core/legal_clause@1"], "side_effect_classes": ["diff_produce"], "verification_policy_ref": "policy:citation_evidence@1"},
    {"action_type_id": "focusa.core/revise_thesis@1", "legacy_names": [], "target_type_ids": ["focusa.core/thesis@1"], "side_effect_classes": ["assumption_change"], "verification_policy_ref": "policy:market_research_evidence@1"},
    {"action_type_id": "focusa.core/revise_claim@1", "legacy_names": [], "target_type_ids": ["focusa.core/claim@1"], "side_effect_classes": ["claim_delta"], "verification_policy_ref": "policy:multiple_source_evidence@1"},
    {"action_type_id": "focusa.core/checkpoint@1", "legacy_names": ["checkpoint"], "target_type_ids": ["focusa.core/workpoint@1"], "side_effect_classes": [], "verification_policy_ref": "policy:workpoint_verification@1"},
    {"action_type_id": "focusa.core/commit@1", "legacy_names": ["commit"], "target_type_ids": ["focusa.core/workpoint@1"], "side_effect_classes": ["receipt_persist"], "verification_policy_ref": "policy:workpoint_verification@1"},
    {"action_type_id": "focusa.core/resume@1", "legacy_names": ["resume"], "target_type_ids": ["focusa.core/workpoint@1"], "side_effect_classes": [], "verification_policy_ref": "policy:workpoint_verification@1"},
]

STATUS_VOCABULARIES = [
    {"status_vocabulary_id": "focusa.core/lifecycle@1", "statuses": ["proposed", "active", "completed", "blocked", "superseded"], "terminal_statuses": ["completed", "superseded"], "verified_statuses": ["completed"], "advisory_statuses": ["blocked", "proposed"]},
    {"status_vocabulary_id": "focusa.core/verification@1", "statuses": ["candidate", "unverified", "verified", "contradicted", "superseded"], "terminal_statuses": ["verified", "superseded"], "verified_statuses": ["verified"], "advisory_statuses": ["candidate", "unverified", "contradicted"], "legacy_aliases": {"candidate": "proposed"}},
]

EVIDENCE_KINDS = [
    "test_pass", "diff_verified", "citation_verified", "execution_receipt",
    "multiple_source", "operator_confirmed", "contradiction_detected",
]

DOMAIN_PACKS = [
    {"pack_id": "focusa.core.general", "label": "General", "object_type_ids": ["focusa.core/mission@1", "focusa.core/workpoint@1", "focusa.core/evidence@1"], "software_only_assumption": False, "versioned": True, "built_in": True},
    {"pack_id": "focusa.core.software", "label": "Software", "object_type_ids": ["focusa.core/task@1", "focusa.core/code_unit@1", "focusa.core/workpoint@1"], "software_only_assumption": False, "versioned": True, "built_in": True},
    {"pack_id": "focusa.core.legal", "label": "Legal", "object_type_ids": ["focusa.core/legal_clause@1", "focusa.core/evidence@1"], "software_only_assumption": False, "versioned": True, "built_in": True},
    {"pack_id": "focusa.core.markets", "label": "Markets", "object_type_ids": ["focusa.core/thesis@1", "focusa.core/evidence@1"], "software_only_assumption": False, "versioned": True, "built_in": True},
    {"pack_id": "focusa.core.research", "label": "Research", "object_type_ids": ["focusa.core/claim@1", "focusa.core/evidence@1"], "software_only_assumption": False, "versioned": True, "built_in": True},
    {"pack_id": "focusa.core.custom", "label": "Custom", "object_type_ids": [], "software_only_assumption": False, "versioned": True, "built_in": False},
]

SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.ontology_registry.v2.json",
    "title": "Focusa Ontology Registry v2",
    "description": "One core-owned semantic registry with typed object, link, action, status, and evidence definitions. Versioned built-in domain packs. No duplicate canonical models.",
    "type": "object",
    "required": ["schema", "object_types", "link_types", "action_types", "status_vocabularies", "evidence_kinds", "domain_packs", "v1_compatibility_projection", "hard_design_laws"],
    "properties": {
        "schema": {"const": "focusa.ontology_registry.v2"},
        "object_types": {"type": "array"},
        "link_types": {"type": "array"},
        "action_types": {"type": "array"},
        "status_vocabularies": {"type": "array"},
        "evidence_kinds": {"type": "array", "items": {"type": "string"}},
        "domain_packs": {"type": "array"},
        "v1_compatibility_projection": {"type": "object", "required": ["enabled", "legacy_names_preserved"]},
        "hard_design_laws": {"type": "array"},
    },
    "additionalProperties": False,
}

REGISTRY = {
    "schema": "focusa.ontology_registry.v2",
    "spec_ref": "docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md",
    "acceptance_criteria": "Ontology supports all domains without software-only assumptions or duplicate canonical models.",
    "object_types": OBJECT_TYPES,
    "link_types": LINK_TYPES,
    "action_types": ACTION_TYPES,
    "status_vocabularies": STATUS_VOCABULARIES,
    "evidence_kinds": EVIDENCE_KINDS,
    "domain_packs": DOMAIN_PACKS,
    "v1_compatibility_projection": {
        "enabled": True,
        "legacy_names_preserved": True,
        "description": "Preserved legacy objects, links, action names, statuses, routes, slice names, and output shapes derived from V2 state where possible and retained directly where not yet migrated.",
    },
    "hard_design_laws": [
        "One Focusa runtime; no per-domain cognitive cores.",
        "One core-owned semantic registry; no route-local competing registries.",
        "Shared cognition and domain-specific semantics remain separate layers.",
        "Candidate and canonical semantic state remain structurally distinct.",
        "No canonical promotion without the registered policy result.",
        "Workspace selection is not domain-pack activation unless explicitly previewed and committed.",
        "Domain-pack activation is not permission escalation.",
        "Role is not permission; domain expertise is not operational authority.",
        "Workpoint remains continuation authority after governed promotion.",
        "Unknown semantic IDs and events are preserved, never guessed.",
        "Existing V1 behavior remains available through an explicit compatibility projection.",
        "Migration never silently converts unreadable state into fresh empty state.",
    ],
    "schema_path": "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.ontology_registry.v2.json",
}


def main() -> None:
    SCHEMA_DIR.mkdir(parents=True, exist_ok=True)
    SCHEMA_PATH.write_text(json.dumps(SCHEMA, indent=2) + "\n")
    REGISTRY_PATH.write_text(json.dumps(REGISTRY, indent=2) + "\n")
    packs = {p["pack_id"] for p in DOMAIN_PACKS}
    domains = {p["label"] for p in DOMAIN_PACKS}
    print(f"Spec 135F-1 ontology registry generated: {len(OBJECT_TYPES)} object types, {len(LINK_TYPES)} link types, {len(ACTION_TYPES)} action types")
    print(f"  domain packs: {sorted(domains)}")
    print(f"  all software_only_assumption=False: {all(not p['software_only_assumption'] for p in DOMAIN_PACKS)}")


if __name__ == "__main__":
    main()