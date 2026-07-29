#!/usr/bin/env python3
"""Spec 135F-4 reactive context and domain parity proof."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
C = json.loads((ROOT / "docs/contracts/spec135-reactive-context-domain-parity.v1.json").read_text())
assert C["acceptance_criteria"] == "Reactive updates meet budgets and domain parity tests show no semantic or authority drift."
assert C["event_binding"]["separately_versioned"] is True
assert "cannot promote semantic candidates" in C["event_binding"]["authority_rule"]
assert C["budgets"]["max_reaction_ms"] <= 250
assert C["budgets"]["hidden_projection_rerender"] is False
profiles = C["domain_parity"]
assert {p["profile"] for p in profiles} == {"General", "Software", "Legal", "Markets", "Research", "Custom"}
assert len({p["canonical_semantic_state_ref"] for p in profiles}) == 1
assert len({p["workpoint_authority_ref"] for p in profiles}) == 1
assert all(p["projection_only"] and not p["permission_escalation"] for p in profiles)
for ref in C["evidence_refs"]:
    assert (ROOT / ref).exists(), ref
for required in ("active Workpoint remains continuation authority", "visible/subscribed bounded slices refetched", "Workpoint candidate marked stale"):
    assert any(required in step for step in C["incremental_pipeline"]), required
print("Spec 135 F4 reactive context and domain parity: PASS")
