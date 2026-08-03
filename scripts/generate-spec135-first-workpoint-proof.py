#!/usr/bin/env python3
"""Generate Spec 135I-4 first proven Workpoint integration proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/"docs/contracts/spec135/generated-contract-v1"
Z2=json.loads((BASE/"spec135-z2-permanent-integration-evidence.json").read_text())
OUT=ROOT/"docs/contracts/spec135-first-workpoint-integration.v1.json"
C={
 "schema":"focusa.spec135.first_workpoint_integration.v1",
 "acceptance_criteria":"End-to-end production journey reaches first verified Workpoint and survives renderer/API degradation.",
 "journey":["guided onboarding","source-linked context","approved role","closed interview","reviewed spec","materialized task","first Workpoint","execution evidence","durable receipt","live refresh"],
 "permanent_chain":Z2["chain"],
 "real_execution_refs":[f"docs/contracts/spec135/generated-contract-v1/{ref}" for ref in Z2["real_execution_refs"]],
 "continuity_proof":Z2["continuity_proof"],
 "first_workpoint":{"no_fork":True,"verified_evidence_required":True,"receipt_required":True,"provider_task_materialized":True},
 "fallbacks":{"renderer":"bounded text artifact card + Open action","api_disconnect":"stale label + cursor replay + snapshot fallback","unsupported_action":"disabled with plain-language recovery","degraded_scope":"read-only; canonical mutation blocked"},
 "evidence_refs":["tests/spec135_b7_genesis_resume_e2e_test.py","tests/spec135_i3_nontechnical_onboarding_test.py","tests/spec135_j4_reconnect_recovery_test.py","tests/spec135_c1_rich_artifact_test.py"],
}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print("Spec 135I-4 first proven Workpoint integration generated")
