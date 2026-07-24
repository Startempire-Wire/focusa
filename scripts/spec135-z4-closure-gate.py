#!/usr/bin/env python3
"""Clean-checkout, strict-static Spec 135 Z4 closure gate."""

import json
import subprocess
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[1]
TESTS = [
    "tests/spec135_generated_clients_test.py",
    "tests/spec135_protocol_handshake_test.py",
    "tests/spec135_ready_frontier_contracts_lint.py",
    "tests/spec135_security_migration_contracts_lint.py",
    "tests/spec135_p2_connector_framework_lint.py",
    "tests/spec135_p3_connector_auth_lint.py",
    "tests/spec135_c4_google_drive_connector_lint.py",
    "tests/spec135_p4_ag_ui_adapter_lint.py",
    "tests/spec135_p5_parity_migration_lint.py",
    "tests/spec135_q5_supply_chain_lint.py",
    "tests/spec135_q6_quality_gate_lint.py",
    "tests/spec135_e5_rollout_closure_lint.py",
    "tests/spec135_v4_software_domain_lint.py",
    "tests/spec135_u4_u5_usability_friction_test.py",
    "tests/spec135_u6_adaptive_ui_test.py",
    "tests/spec135_alpha7_domain_parity_lint.py",
    "tests/spec135_alpha8_nontechnical_dogfood_lint.py",
    "tests/spec135_z1_closure_matrix_lint.py",
    "tests/spec135_z2_permanent_integration_lint.py",
    "tests/spec135_z3_lineage_lint.py",
]

status = subprocess.check_output(["git", "status", "--porcelain=v1"], cwd=R, text=True).strip()
checks = [{"gate": "clean_checkout", "ok": not status, "detail": "clean" if not status else "dirty"}]
for test in TESTS:
    run = subprocess.run(["python3", str(R / test)], cwd=R, capture_output=True, text=True)
    checks.append({
        "gate": test,
        "ok": run.returncode == 0,
        "detail": (run.stdout or run.stderr).strip()[-160:],
    })

bundle = R / "docs/contracts/spec135/generated-contract-v1"
for result_path in sorted(bundle.glob("uiai-eval.*.result.json")):
    result = json.loads(result_path.read_text())
    checks.append({"gate": result_path.name, "ok": result.get("status") == "passed", "detail": result.get("status", "missing")})

openapi = json.loads((bundle / "openapi-3.0.3.json").read_text())
operation_ids = [
    operation["operationId"]
    for path in openapi["paths"].values()
    for method, operation in path.items()
    if method in {"get", "post", "put", "patch", "delete"}
]
checks.append({
    "gate": "schemathesis_compatible_openapi_contract",
    "ok": openapi.get("openapi") == "3.0.3" and len(operation_ids) == len(set(operation_ids)) and len(operation_ids) >= 80,
    "detail": f"operations={len(operation_ids)}",
})

failed = [row for row in checks if not row["ok"]]
result = {
    "schema": "focusa.spec135.z4_closure_gate_result.v1",
    "requirement_id": "SPEC135-Z4",
    "status": "blocked" if failed else "passed",
    "check_count": len(checks),
    "failed_count": len(failed),
    "checks": checks[:64],
    "evidence_ref": "evidence:spec135-z4:strict-clean-checkout",
    "recovery": [f"repair:{row['gate']}" for row in failed[:16]],
}
print(json.dumps(result, indent=2, sort_keys=True))
sys.exit(1 if failed else 0)
