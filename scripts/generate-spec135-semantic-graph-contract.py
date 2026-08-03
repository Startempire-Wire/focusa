#!/usr/bin/env python3
"""Spec 135F-2: generate the durable semantic graph and provenance contract.

Scoped semantic graph nodes/edges, source provenance, contradiction,
confidence, freshness, bounded traversal, and bounded projection.
Candidate and canonical records use separate stores (structural separation).
Graph queries return evidence-backed results and preserve exact
source/session/project identity."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"
CONTRACT_PATH = ROOT / "docs/contracts/spec135-semantic-graph-provenance.v1.json"
CANDIDATE_SCHEMA = SCHEMA_DIR / "focusa.semantic_candidate.v2.json"
CANONICAL_SCHEMA = SCHEMA_DIR / "focusa.canonical_semantic_record.v2.json"

common_scope = {
    "type": "object",
    "required": ["project_root", "continuity_id"],
    "properties": {
        "project_root": {"type": "string", "minLength": 1},
        "continuity_id": {"type": "string", "minLength": 1},
        "workpoint_id": {"type": "string"},
    },
}

candidate_schema = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.semantic_candidate.v2.json",
    "title": "Focusa Semantic Candidate v2",
    "description": "Candidate semantic record. Separate store from canonical; status alone is insufficient separation.",
    "type": "object",
    "required": [
        "schema", "candidate_id", "semantic_kind", "semantic_id",
        "definition_id", "domain_pack_id", "source_ref", "provenance_refs",
        "evidence_refs", "confidence", "freshness", "scope", "status",
        "created_at", "updated_at",
    ],
    "properties": {
        "schema": {"const": "focusa.semantic_candidate.v2"},
        "candidate_id": {"type": "string", "minLength": 1},
        "semantic_kind": {"type": "string", "enum": ["object", "link", "status_change", "membership", "action_result", "identity_resolution"]},
        "semantic_id": {"type": "string", "minLength": 1},
        "definition_id": {"type": "string", "minLength": 1},
        "domain_pack_id": {"type": "string", "minLength": 1},
        "payload": {},
        "source_ref": {"type": "string", "minLength": 1},
        "provenance_refs": {"type": "array", "items": {"type": "string", "minLength": 1}},
        "evidence_refs": {"type": "array", "items": {"type": "string", "minLength": 1}},
        "confidence": {"type": "number", "minimum": 0, "maximum": 1},
        "freshness": {"type": "string", "minLength": 1},
        "expires_at": {"type": "string"},
        "scope": common_scope,
        "status": {"type": "string", "enum": ["proposed", "pending_verification", "verified_candidate", "rejected", "expired", "superseded"]},
        "created_at": {"type": "string", "minLength": 1},
        "updated_at": {"type": "string", "minLength": 1},
    },
    "additionalProperties": False,
}

canonical_schema = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.canonical_semantic_record.v2.json",
    "title": "Focusa Canonical Semantic Record v2",
    "description": "Canonical semantic record. Separate store from candidate; promotion requires verification record.",
    "type": "object",
    "required": [
        "schema", "semantic_id", "definition_id", "domain_pack_id",
        "revision", "status", "promotion_record_ref", "verification_record_refs",
        "provenance_refs", "scope", "created_at", "updated_at",
    ],
    "properties": {
        "schema": {"const": "focusa.canonical_semantic_record.v2"},
        "semantic_id": {"type": "string", "minLength": 1},
        "definition_id": {"type": "string", "minLength": 1},
        "domain_pack_id": {"type": "string", "minLength": 1},
        "revision": {"type": "integer", "minimum": 1},
        "payload": {},
        "status": {"type": "string"},
        "identity_resolution_ref": {"type": "string"},
        "promotion_record_ref": {"type": "string", "minLength": 1},
        "verification_record_refs": {"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}},
        "provenance_refs": {"type": "array", "items": {"type": "string", "minLength": 1}},
        "scope": common_scope,
        "created_at": {"type": "string", "minLength": 1},
        "updated_at": {"type": "string", "minLength": 1},
    },
    "additionalProperties": False,
}

contract = {
    "schema": "focusa.spec135.semantic_graph_provenance.v1",
    "spec_ref": "docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md",
    "acceptance_criteria": "Graph queries return evidence-backed results and preserve exact source/session/project identity.",
    "structural_separation": "Candidate and canonical records use separate stores and indexes. A status field alone is insufficient separation.",
    "semantic_kinds": ["object", "link", "status_change", "membership", "action_result", "identity_resolution"],
    "provenance_invariants": [
        "Every candidate and canonical record carries source_ref and provenance_refs",
        "Graph queries preserve exact source/session/project identity",
        "Contradiction records link contradicting claims with evidence",
        "Confidence is 0.0-1.0 per candidate",
        "Freshness is explicit and may expire",
        "Provenance and identity are preserved through projection, never guessed",
    ],
    "traversal_bounds": {
        "depth": True,
        "node": True,
        "edge": True,
        "time": True,
        "token": True,
    },
    "contradiction": {
        "link_type": "focusa.core/contradicts@1",
        "requires_evidence": True,
        "preserves_both_claims": True,
    },
    "candidate_schema_ref": "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.semantic_candidate.v2.json",
    "canonical_schema_ref": "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.canonical_semantic_record.v2.json",
}


def main() -> None:
    SCHEMA_DIR.mkdir(parents=True, exist_ok=True)
    CANDIDATE_SCHEMA.write_text(json.dumps(candidate_schema, indent=2) + "\n")
    CANONICAL_SCHEMA.write_text(json.dumps(canonical_schema, indent=2) + "\n")
    CONTRACT_PATH.write_text(json.dumps(contract, indent=2) + "\n")
    print("Spec 135F-2 semantic graph + provenance contract generated")
    print(f"  candidate schema: focusa.semantic_candidate.v2.json")
    print(f"  canonical schema: focusa.canonical_semantic_record.v2.json")


if __name__ == "__main__":
    main()