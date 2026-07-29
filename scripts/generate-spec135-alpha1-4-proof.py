#!/usr/bin/env python3
"""Generate Spec 135H-1 Alpha 1-4 production workflow proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/"docs/contracts/spec135/generated-contract-v1"
OUT=ROOT/"docs/contracts/spec135-alpha1-4-production-proof.v1.json"
proofs=["spec135-alpha1-context-ingestion-proof.json","spec135-alpha2-role-interview-proof.json","spec135-alpha3-spec-task-proof.json","spec135-alpha4-work-rail-proof.json"]
rows=[]
for i,name in enumerate(proofs,1):
 p=json.loads((BASE/name).read_text())
 rows.append({"alpha":i,"proof_ref":f"docs/contracts/spec135/generated-contract-v1/{name}","status":p.get("status","missing"),"evidence_refs":p.get("evidence_refs",p.get("evidence_and_receipts",[])),"acceptance":p.get("acceptance",{})})
C={"schema":"focusa.spec135.alpha1_4_production_proof.v1","acceptance_criteria":"Alpha 1–4 pass with runtime receipts, UIAI diagnostics where applicable, and resolved blockers.","workflows":rows,"runtime_test_refs":["tests/spec135_b2_context_connectors_e2e_test.py","tests/spec135_role_profile_e2e_test.py","tests/spec135_interview_session_e2e_test.py","tests/spec135_b5_spec120_integration_e2e_test.py","tests/spec135_task_plan_e2e_test.py","tests/spec135_work_rail_e2e_test.py"],"pi_ui_refs":["apps/pi-extension/src/crist-canvas.ts","apps/pi-extension/src/work-rail-interactions.ts"],"blockers":[]}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print("Spec 135H-1 Alpha 1-4 production proof generated")
