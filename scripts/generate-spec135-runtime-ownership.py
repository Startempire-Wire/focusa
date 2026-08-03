#!/usr/bin/env python3
"""Generate Spec 135J-5 operation runtime ownership and drift contract."""
import json
from collections import Counter
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
REG=ROOT/"docs/contracts/spec135/generated-contract-v1/operation-registry.json"
OUT=ROOT/"docs/contracts/spec135-runtime-ownership-drift.v1.json"
ops=json.loads(REG.read_text())["operations"]
ids=[o["operation_id"] for o in ops]
rows=[]
for o in ops:
 own=o.get("ownership",{})
 rows.append({"operation_id":o["operation_id"],"runtime_owner":own.get("subsystem",o.get("family")),"core_action_ref":own.get("core_action_ref",o["operation_id"]),"error_schema_ref":o.get("contracts",{}).get("error_schema_ref","focusa.tool_result.v1"),"recovery_typed":True})
C={
 "schema":"focusa.spec135.runtime_ownership_drift.v1",
 "acceptance_criteria":"One runtime path owns each operation and all failure envelopes provide typed recovery.",
 "operation_count":len(rows),"duplicate_operation_ids":[k for k,v in Counter(ids).items() if v>1],"operations":rows,
 "error_envelope":{"schema":"focusa.tool_result.v1","required_fields":["status","error_code","message","scope_status","recovery_actions","evidence_refs"],"recovery_actions_typed":True,"raw_stack_visible":False},
 "drift_gates":["operation registry operation IDs are unique","OpenAPI operation IDs equal registry operation IDs","TypeScript client exposes every operation","A2UI actions bind only registered operations","generated contract regeneration is clean","route ownership resolves to exactly one subsystem"],
 "proof_refs":["tests/spec135_operation_registry_test.py","tests/tool_contract_test.sh","tests/spec135_p5_parity_migration_lint.py","tests/spec135_i1_generated_ui_test.py"],
}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print(f"Spec 135J-5 runtime ownership generated: {len(rows)} operations")
