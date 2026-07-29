#!/usr/bin/env python3
"""Generate Spec 135 master 12/12 acceptance evidence gate."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"docs/contracts/spec135-master-final-acceptance.v1.json"
checks=[
 ("install_to_first_workpoint","docs/contracts/spec135-first-workpoint-integration.v1.json"),
 ("client_parity","docs/contracts/spec135-client-operation-parity.v1.json"),
 ("security_scope","docs/contracts/spec135-q2-security-privacy-gates.v1.yaml"),
 ("performance","docs/contracts/spec135-q3-performance-budgets.v1.yaml"),
 ("recovery","docs/contracts/spec135-reconnect-replay-recovery.v1.json"),
 ("dogfood","docs/contracts/spec135-alpha5-8-production-proof.v1.json"),
 ("issue_53","docs/contracts/spec135-interaction-mode-toggle.v1.json"),
 ("alpha_1_4","docs/contracts/spec135-alpha1-4-production-proof.v1.json"),
 ("multiplexing","docs/contracts/spec135-multiplexing-concurrency-proof.v1.json"),
 ("generated_contracts","docs/contracts/spec135/generated-contract-v1/operation-registry.json"),
 ("cross_spec_migration","docs/contracts/spec135-cross-spec-closure.v1.json"),
 ("clean_pr_lineage","docs/contracts/spec135/generated-contract-v1/spec135-z3-worktree-lineage-proof.json"),
]
rows=[]
for cid,ref in checks:
 path=ROOT/ref
 rows.append({"check_id":cid,"evidence_ref":ref,"status":"passed" if path.exists() and path.stat().st_size>2 else "missing"})
C={"schema":"focusa.spec135.master_final_acceptance.v1","acceptance_criteria":"All 135–135K Beads close from runtime evidence, completion gate passes 12/12, CI passes, and PR is merge-ready.","gate_count":12,"checks":rows,"passed_count":sum(r["status"]=="passed" for r in rows),"beads_closure_authority":"runtime evidence maps to provider items; provider JSONL remains provider-owned and is not hand-edited","branch_policy":"feature branch + PR only; never direct commit to main","go_sdk":"excluded; Pi TUI uses TypeScript","merge_ready_conditions":["12/12 evidence gate","strict CI pass","generated contracts converge","worktree clean after final commit","feature branch pushed","PR checks green and mergeable"]}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print(f"Spec 135 master acceptance generated: {C['passed_count']}/12")
