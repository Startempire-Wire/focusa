#!/usr/bin/env python3
"""Spec 135I-4 first proven Workpoint/fallback integration proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-first-workpoint-integration.v1.json").read_text())
assert C["acceptance_criteria"] == "End-to-end production journey reaches first verified Workpoint and survives renderer/API degradation."
assert C["journey"] == ["guided onboarding","source-linked context","approved role","closed interview","reviewed spec","materialized task","first Workpoint","execution evidence","durable receipt","live refresh"]
assert C["permanent_chain"][-3:] == ["artifact","mission_canvas"] or C["permanent_chain"][-1] == "mission_canvas"
assert all(C["continuity_proof"].values())
assert all(C["first_workpoint"].values())
for ref in C["real_execution_refs"]+C["evidence_refs"]:
    assert (ROOT/ref).exists(), ref
for fallback in ("renderer","api_disconnect","unsupported_action","degraded_scope"):
    assert C["fallbacks"][fallback], fallback
assert "snapshot fallback" in C["fallbacks"]["api_disconnect"]
assert "canonical mutation blocked" in C["fallbacks"]["degraded_scope"]
print("Spec 135 I4 first proven Workpoint and fallback integration: PASS")
