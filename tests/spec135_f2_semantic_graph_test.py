#!/usr/bin/env python3
"""Spec 135F-2 semantic graph and provenance proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md").read_text()
CONTRACT = json.loads(
    (ROOT / "docs/contracts/spec135-semantic-graph-provenance.v1.json").read_text()
)
CANDIDATE = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.semantic_candidate.v2.json").read_text()
)
CANONICAL = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.canonical_semantic_record.v2.json").read_text()
)

assert CANDIDATE["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert CANDIDATE["title"] == "Focusa Semantic Candidate v2"
for field in ("candidate_id", "semantic_kind", "source_ref", "provenance_refs", "evidence_refs", "confidence", "freshness", "scope", "status"):
    assert field in CANDIDATE["required"], field
assert "verified_candidate" in CANDIDATE["properties"]["status"]["enum"]

assert CANONICAL["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert CANONICAL["title"] == "Focusa Canonical Semantic Record v2"
for field in ("semantic_id", "definition_id", "revision", "promotion_record_ref", "verification_record_refs", "provenance_refs", "scope"):
    assert field in CANONICAL["required"], field
# Canonical record requires at least one verification record
assert CANONICAL["properties"]["verification_record_refs"]["minItems"] == 1

# Structural separation: separate schema IDs (not just a status field)
assert CANDIDATE["properties"]["schema"]["const"] != CANONICAL["properties"]["schema"]["const"]
assert CONTRACT["structural_separation"] == "Candidate and canonical records use separate stores and indexes. A status field alone is insufficient separation."

# Provenance invariants
assert CONTRACT["acceptance_criteria"] == "Graph queries return evidence-backed results and preserve exact source/session/project identity."
for inv in ("Every candidate and canonical record carries source_ref and provenance_refs",
            "Graph queries preserve exact source/session/project identity",
            "Contradiction records link contradicting claims with evidence",
            "Confidence is 0.0-1.0 per candidate",
            "Freshness is explicit and may expire",
            "Provenance and identity are preserved through projection, never guessed"):
    assert inv in CONTRACT["provenance_invariants"], inv

# Traversal bounds
bounds = CONTRACT["traversal_bounds"]
for b in ("depth", "node", "edge", "time", "token"):
    assert bounds[b] is True, f"missing bound: {b}"

# Contradiction
assert CONTRACT["contradiction"]["requires_evidence"] is True
assert CONTRACT["contradiction"]["preserves_both_claims"] is True

for spec_text in (
    "Candidate and canonical semantic graphs",
    "semantic_candidate.v2",
    "canonical_semantic_record.v2",
    "provenance_refs",
    "Structural separation",
    "Graph traversal has depth, node, edge, time, and token bounds",
):
    assert spec_text in SPEC, spec_text

print("Spec 135 F2 semantic graph and provenance: PASS")