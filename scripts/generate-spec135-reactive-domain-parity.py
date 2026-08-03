#!/usr/bin/env python3
"""Generate Spec 135F-4 reactive context and domain parity contract."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/contracts/spec135-reactive-context-domain-parity.v1.json"
PACKS = ["General", "Software", "Legal", "Markets", "Research", "Custom"]
CONTRACT = {
    "schema": "focusa.spec135.reactive_context_domain_parity.v1",
    "acceptance_criteria": "Reactive updates meet budgets and domain parity tests show no semantic or authority drift.",
    "event_binding": {
        "ui_invalidation_schema": "focusa.workspace_event.v1",
        "semantic_delta_schema": "focusa.semantic_delta.v2",
        "separately_versioned": True,
        "authority_rule": "UI invalidation triggers bounded refetch only; it cannot promote semantic candidates or mint Workpoint authority.",
    },
    "incremental_pipeline": [
        "canonical graph revision committed",
        "semantic delta emitted with exact project/workstream scope",
        "affected generalized slice keys invalidated",
        "visible/subscribed bounded slices refetched",
        "Workpoint candidate marked stale when its semantic revision is superseded",
        "active Workpoint remains continuation authority",
    ],
    "budgets": {
        "max_delta_batch": 100,
        "max_traversal_depth": 8,
        "max_nodes": 200,
        "max_edges": 400,
        "max_slice_tokens": 4000,
        "max_reaction_ms": 250,
        "hidden_projection_rerender": False,
    },
    "domain_parity": [
        {
            "profile": profile,
            "canonical_semantic_state_ref": "focusa:canonical-semantic-state:shared",
            "workpoint_authority_ref": "focusa:workpoint-authority:shared",
            "permission_escalation": False,
            "projection_only": True,
        }
        for profile in PACKS
    ],
    "parity_laws": [
        "All profiles project the same canonical semantic revision",
        "Visual profile changes never mutate semantic state",
        "Domain-pack activation never escalates permission or Workpoint authority",
        "General, Software, Legal, Markets, Research, and Custom differ only in registered projection semantics",
        "Unknown semantic deltas fail closed and remain preserved for replay",
    ],
    "evidence_refs": [
        "docs/contracts/spec135-domain-packs-complete.v1.json",
        "docs/contracts/spec135-semantic-graph-provenance.v1.json",
        "docs/contracts/spec135-workspace-invalidation-live-refresh.v1.json",
    ],
}
OUT.write_text(json.dumps(CONTRACT, indent=2) + "\n")
print(f"Spec 135F-4 reactive domain parity generated: {len(PACKS)} projections")
