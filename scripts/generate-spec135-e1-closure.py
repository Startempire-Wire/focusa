#!/usr/bin/env python3
"""Generate Spec 135E-1 migration/amendment/rollback closure proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
CROOT=ROOT/"docs/contracts"
OUT=CROOT/"spec135-cross-spec-closure.v1.json"
refs=["spec135-e1-migration-inventory.v1.yaml","spec135-e2-e4-migration-plan.v1.yaml","spec135-e5-rollout-closure.v1.yaml"]
contracts=[]
for ref in refs:
 p=json.loads((CROOT/ref).read_text()); contracts.append({"ref":f"docs/contracts/{ref}","schema":p["schema"],"acceptance":p.get("acceptance",{}),"passed":bool(p.get("acceptance")) and all(p["acceptance"].values())})
C={"schema":"focusa.spec135.cross_spec_closure.v1","acceptance_criteria":"All amendments and migrations pass; no authority fork or legacy Mission Canvas substitute remains.","migration_contracts":contracts,"authority":{"canonical_operation_registry":"docs/contracts/spec135/generated-contract-v1/operation-registry.json","canonical_workpoint":"Focusa Workpoint reducer","presentation_clients_own_state":False,"parallel_runtime":False},"legacy_removals":[{"surface":"Pi Work Rail","removed":"setWidget(\"focusa\") legacy duplicate","canonical":"focusa-mission-canvas-work-rail"},{"surface":"generated UI","removed":"second Svelte A2UI renderer","canonical":"@a2ui/lit/v0_9"},{"surface":"semantic registry","removed":"route-local competing registries","canonical":"focusa.ontology_registry.v2"}],"rollback":{"exact_snapshot":True,"append_only_history":True,"original_state_preserved_on_failure":True,"silent_empty_initialization":False},"proof_refs":["tests/spec135_e5_rollout_closure_lint.py","tests/spec135_machine_readable_delivery_graph_test.py","tests/spec135_j5_runtime_ownership_test.py","tests/spec135_m2_pi_work_rail_test.py"]}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print("Spec 135E-1 cross-spec closure generated")
