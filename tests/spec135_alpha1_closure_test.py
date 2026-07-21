#!/usr/bin/env python3
"""Cross-functional SPEC135-ALPHA1 merge-gate proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
alpha = json.loads((BUNDLE / "spec135-alpha1-context-ingestion-proof.json").read_text())

assert alpha["schema"] == "focusa.spec135_alpha1_context_ingestion_proof.v1"
assert alpha["status"] == "passed"
assert alpha["merge_gate"]["all_feeders_closed"] is True
assert alpha["merge_gate"]["required_feeders"] == [
    "SPEC135-F12",
    "SPEC135-C3",
    "SPEC135-U2",
]
assert [
    "SPEC135-F12",
    "SPEC135-C1",
    "SPEC135-C2",
    "SPEC135-C3",
    "SPEC135-ALPHA1",
] in alpha["merge_gate"]["critical_paths"]
assert [
    "SPEC135-F12",
    "SPEC135-U1",
    "SPEC135-U2",
    "SPEC135-ALPHA1",
] in alpha["merge_gate"]["critical_paths"]

proofs = {}
for mapping in alpha["acceptance_mapping"].values():
    proof = json.loads((BUNDLE / mapping["proof_ref"]).read_text())
    assert proof["status"] in {"passed", "verified"}, mapping["proof_ref"]
    proofs[mapping["proof_ref"]] = proof

c1 = proofs["spec135-c1-context-ingestion-proof.json"]
assert c1["runtime_proof"]["canonical_source_count"] == 3
assert c1["runtime_proof"]["pdf_extraction_status"] == "success"
assert c1["runtime_proof"]["restart_source_count"] == 3

c2 = proofs["spec135-c2-context-retrieval-proof.json"]
assert "source_locator" in c2["runtime_proof"]["citation_fields"]
assert "line_start" in c2["runtime_proof"]["citation_fields"]
assert c2["runtime_proof"]["restart_result"] == "same ordered chunk ids"
assert (
    c2["runtime_proof"]["vector_absence_behavior"] == "deterministic lexical fallback"
)

c3 = proofs["spec135-c3-context-claim-graph-proof.json"]
assert c3["runtime_proof"]["restart"] == "exact_projection_resumed"
assert c3["runtime_proof"]["resolved_contradictions"] == 1
assert c3["runtime_proof"]["unresolved_after_resolution"] == 0

u1 = proofs["spec135-u1-workspace-artifact-proof.json"]
assert u1["runtime_proof"]["external_artifact_authority"] is True
assert u1["runtime_proof"]["restart"] == "revision_2_descriptor_resumed"

u2 = proofs["spec135-u2-workspace-live-refresh-proof.json"]
assert u2["runtime_proof"]["workspace_event_schema"] == "focusa.workspace_event.v1"
assert u2["generated_ui_proof"]["surface_a_renders"] == 2
assert u2["generated_ui_proof"]["surface_b_renders"] == 0

for eval_ref in alpha["generated_ui_eval_refs"]:
    result = json.loads((BUNDLE / eval_ref).read_text())
    assert result["status"] == "passed", eval_ref

print(
    "Spec 135 Alpha 1 closure: PASS (real ingest, cited retrieval, governed claims, rich rendering, restart-safe targeted refresh)"
)
