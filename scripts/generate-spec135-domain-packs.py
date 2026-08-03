#!/usr/bin/env python3
"""Spec 135F-3: generate the complete domain pack manifests with schemas,
panels, renderers, operations, and migrations. Each pack shares canonical
ontology and authority."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"
MANIFEST_SCHEMA = SCHEMA_DIR / "focusa.domain_pack_manifest.v1.json"
CONTRACT_PATH = ROOT / "docs/contracts/spec135-domain-packs-complete.v1.json"

PACKS = [
    {
        "pack_id": "focusa.core.cognition",
        "label": "Cognition",
        "pack_version": 1, "compatibility_version": 1, "minimum_core_version": "0.9.141",
        "extends": [],
        "object_type_ids": ["focusa.core/mission@1", "focusa.core/workpoint@1", "focusa.core/evidence@1"],
        "link_type_ids": ["focusa.core/depends_on@1", "focusa.core/proves@1", "focusa.core/decomposes_to@1"],
        "action_type_ids": ["focusa.core/checkpoint@1", "focusa.core/resume@1", "focusa.core/commit@1", "focusa.core/link_evidence@1"],
        "status_vocabulary_ids": ["focusa.core/lifecycle@1", "focusa.core/verification@1"],
        "verification_policy_ids": ["policy:workpoint_verification@1"],
        "promotion_policy_ids": ["policy:workpoint_promotion@1"],
        "slice_policy_ids": ["policy:bounded_slice@1"],
        "panel": None, "renderer_id": None,
        "migration_refs": [], "legacy_aliases": {}, "built_in": True, "compositional_base": True, "custom": False,
        "security_classification": "internal", "license": "proprietary",
        "domain_acceptance": "Cognition primitives (mission, workpoint, evidence) resolve under shared authority.",
    },
    {
        "pack_id": "focusa.general", "label": "General", "pack_version": 1, "compatibility_version": 1, "minimum_core_version": "0.9.141",
        "extends": ["focusa.core.cognition@1"],
        "object_type_ids": ["focusa.core/mission@1", "focusa.core/workpoint@1", "focusa.core/evidence@1"],
        "link_type_ids": ["focusa.core/depends_on@1", "focusa.core/decomposes_to@1"],
        "action_type_ids": ["focusa.core/commit@1"],
        "status_vocabulary_ids": ["focusa.core/lifecycle@1"],
        "panel": "General", "renderer_id": "general-artifact-card", "migration_refs": [], "legacy_aliases": {},
        "verification_policy_ids": ["policy:operator_confirmed@1"], "promotion_policy_ids": ["policy:workpoint_promotion@1"], "slice_policy_ids": ["policy:bounded_slice@1"],
        "built_in": True, "compositional_base": True, "custom": False, "security_classification": "internal", "license": "proprietary",
        "domain_acceptance": "General projects compose workpoints with no domain-specific assumptions.",
    },
    {
        "pack_id": "focusa.software", "label": "Software", "pack_version": 1, "compatibility_version": 1, "minimum_core_version": "0.9.141",
        "extends": ["focusa.core.cognition@1", "focusa.general@1"],
        "object_type_ids": ["focusa.core/task@1", "focusa.core/code_unit@1"],
        "link_type_ids": ["focusa.core/depends_on@1"],
        "action_type_ids": ["focusa.core/complete_task@1", "focusa.core/edit@1"],
        "status_vocabulary_ids": ["focusa.core/lifecycle@1"],
        "panel": "Software", "renderer_id": "software-unified-diff", "migration_refs": ["migration:software@1"], "legacy_aliases": {"task": "focusa.core/task@1", "file": "focusa.core/code_unit@1"},
        "verification_policy_ids": ["policy:test_evidence@1"], "promotion_policy_ids": ["policy:workpoint_promotion@1"], "slice_policy_ids": ["policy:bounded_slice@1"],
        "built_in": True, "compositional_base": False, "custom": False, "security_classification": "internal", "license": "proprietary",
        "domain_acceptance": "Software projects produce test-evidenced diffs under shared authority.",
    },
    {
        "pack_id": "focusa.legal", "label": "Legal", "pack_version": 1, "compatibility_version": 1, "minimum_core_version": "0.9.141",
        "extends": ["focusa.core.cognition@1", "focusa.general@1"],
        "object_type_ids": ["focusa.core/legal_clause@1"],
        "link_type_ids": ["focusa.core/cites@1"],
        "action_type_ids": ["focusa.core/redline@1"],
        "status_vocabulary_ids": ["focusa.core/lifecycle@1"],
        "panel": "Legal", "renderer_id": "legal-side-by-side-redline", "migration_refs": ["migration:legal@1"], "legacy_aliases": {},
        "verification_policy_ids": ["policy:citation_evidence@1"], "promotion_policy_ids": ["policy:workpoint_promotion@1"], "slice_policy_ids": ["policy:bounded_slice@1"],
        "built_in": True, "compositional_base": False, "custom": False, "security_classification": "internal", "license": "proprietary",
        "domain_acceptance": "Legal projects cite and redline with authority under shared canonical workpoint.",
    },
    {
        "pack_id": "focusa.markets", "label": "Markets", "pack_version": 1, "compatibility_version": 1, "minimum_core_version": "0.9.141",
        "extends": ["focusa.core.cognition@1", "focusa.general@1"],
        "object_type_ids": ["focusa.core/thesis@1"],
        "link_type_ids": ["focusa.core/depends_on@1", "focusa.core/decomposes_to@1"],
        "action_type_ids": ["focusa.core/revise_thesis@1"],
        "status_vocabulary_ids": ["focusa.core/lifecycle@1"],
        "panel": "Markets", "renderer_id": "markets-thesis-revision", "migration_refs": ["migration:markets@1"], "legacy_aliases": {},
        "verification_policy_ids": ["policy:market_research_evidence@1"], "promotion_policy_ids": ["policy:workpoint_promotion@1"], "slice_policy_ids": ["policy:bounded_slice@1"],
        "built_in": True, "compositional_base": False, "custom": False, "security_classification": "internal", "license": "proprietary",
        "domain_acceptance": "Markets projects revise theses with visible assumptions and invalidation rules.",
    },
    {
        "pack_id": "focusa.research", "label": "Research", "pack_version": 1, "compatibility_version": 1, "minimum_core_version": "0.9.141",
        "extends": ["focusa.core.cognition@1", "focusa.general@1"],
        "object_type_ids": ["focusa.core/claim@1"],
        "link_type_ids": ["focusa.core/proves@1", "focusa.core/contradicts@1", "focusa.core/cites@1"],
        "action_type_ids": ["focusa.core/revise_claim@1"],
        "status_vocabulary_ids": ["focusa.core/verification@1"],
        "panel": "Research", "renderer_id": "research-claim-delta", "migration_refs": ["migration:research@1"], "legacy_aliases": {},
        "verification_policy_ids": ["policy:multiple_source_evidence@1"], "promotion_policy_ids": ["policy:workpoint_promotion@1"], "slice_policy_ids": ["policy:bounded_slice@1"],
        "built_in": True, "compositional_base": False, "custom": False, "security_classification": "internal", "license": "proprietary",
        "domain_acceptance": "Research projects revise claims with multiple-source evidence and contradiction tracking.",
    },
    {
        "pack_id": "focusa.custom", "label": "Custom", "pack_version": 1, "compatibility_version": 1, "minimum_core_version": "0.9.141",
        "extends": ["focusa.core.cognition@1", "focusa.general@1"],
        "object_type_ids": [],
        "link_type_ids": [],
        "action_type_ids": [],
        "status_vocabulary_ids": ["focusa.core/lifecycle@1"],
        "panel": "Custom", "renderer_id": "custom-registered-projection", "migration_refs": [],
        "verification_policy_ids": ["policy:operator_confirmed@1"], "promotion_policy_ids": ["policy:workpoint_promotion@1"], "slice_policy_ids": ["policy:bounded_slice@1"],
        "legacy_aliases": {},
        "built_in": False, "compositional_base": False, "custom": True, "security_classification": "internal", "license": "proprietary",
        "custom_pack_requirements": ["schema validation", "namespace ownership", "compatibility declaration", "preview", "migration plan", "operator approval", "import/export classification", "conformance tests", "explicit fallback behavior"],
        "domain_acceptance": "Custom packs require schema validation, namespace ownership, compatibility, preview, migration plan, approval, classification, conformance, and fallback.",
    },
]

MANIFEST_SCHEMA_DEF = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.domain_pack_manifest.v1.json",
    "title": "Focusa Domain Pack Manifest v1",
    "description": "Versioned package extending the shared cognition pack with domain-specific definitions. Composite projects may activate multiple compatible packs.",
    "type": "object",
    "required": ["schema", "pack_id", "pack_version", "compatibility_version", "extends", "object_type_ids", "action_type_ids", "status_vocabulary_ids", "verification_policy_ids", "promotion_policy_ids", "slice_policy_ids", "built_in"],
    "properties": {
        "schema": {"const": "focusa.domain_pack_manifest.v1"},
        "pack_id": {"type": "string", "minLength": 1},
        "pack_version": {"type": "integer", "minimum": 1},
        "compatibility_version": {"type": "integer", "minimum": 1},
        "extends": {"type": "array", "items": {"type": "string"}},
        "object_type_ids": {"type": "array"},
        "link_type_ids": {"type": "array"},
        "action_type_ids": {"type": "array"},
        "status_vocabulary_ids": {"type": "array"},
        "verification_policy_ids": {"type": "array"},
        "promotion_policy_ids": {"type": "array"},
        "slice_policy_ids": {"type": "array"},
        "panel": {"type": ["string", "null"]},
        "renderer_id": {"type": ["string", "null"]},
        "migration_refs": {"type": "array"},
        "legacy_aliases": {"type": "object"},
        "built_in": {"type": "boolean"},
        "custom": {"type": "boolean"},
    },
    "additionalProperties": True,
}

contract = {
    "schema": "focusa.spec135.domain_packs_complete.v1",
    "spec_ref": "docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md",
    "acceptance_criteria": "Each pack passes domain acceptance while sharing canonical ontology and authority.",
    "required_builtin_packs": ["focusa.core.cognition@1", "focusa.general@1", "focusa.software@1", "focusa.legal@1", "focusa.markets@1", "focusa.research@1", "focusa.custom@1"],
    "packs": PACKS,
    "composition_rule": "Composite projects activate multiple compatible packs. A route, UI, connector, or model cannot invent a new canonical type on demand.",
    "shared_authority_law": "All packs share canonical ontology registry and Workpoint authority. Domain-pack activation cannot change permission, workspace, role, evidence profile, or Workpoint authority implicitly.",
    "schema_path": "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.domain_pack_manifest.v1.json",
}


def main() -> None:
    SCHEMA_DIR.mkdir(parents=True, exist_ok=True)
    MANIFEST_SCHEMA.write_text(json.dumps(MANIFEST_SCHEMA_DEF, indent=2) + "\n")
    CONTRACT_PATH.write_text(json.dumps(contract, indent=2) + "\n")
    print(f"Spec 135F-3 domain packs generated: {len(PACKS)} packs")
    print(f"  built-in: {sum(1 for p in PACKS if p['built_in'])}, custom: {sum(1 for p in PACKS if p['custom'])}")
    print(f"  all share canonical ontology: {all(any('focusa.core.cognition@1' in e for e in p['extends']) for p in PACKS if p['pack_id'] != 'focusa.core.cognition')}")


if __name__ == "__main__":
    main()