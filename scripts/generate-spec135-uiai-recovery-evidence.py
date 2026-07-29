#!/usr/bin/env python3
"""Generate Spec 135C-4 UIAI evaluation and failure-recovery evidence."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "docs/contracts/spec135/generated-contract-v1"
OUT = ROOT / "docs/contracts/spec135-uiai-evaluation-recovery.v1.json"
RESULTS = [
    "uiai-eval.alpha0-context-commit.result.json",
    "uiai-eval.alpha0-generated-ui.result.json",
    "uiai-eval.c1-context-ingestion.result.json",
    "uiai-eval.c2-context-retrieval.result.json",
    "uiai-eval.c3-context-claims.result.json",
    "uiai-eval.u1-workspace-artifact.result.json",
    "uiai-eval.u2-workspace-live-refresh.result.json",
]
receipts=[]
for name in RESULTS:
    payload=json.loads((BASE/name).read_text())
    receipts.append({
        "result_ref": f"docs/contracts/spec135/generated-contract-v1/{name}",
        "scenario_id": payload["scenario_id"],
        "status": payload["status"],
        "browser_session_refs": payload.get("browser_session_refs", []),
        "browser_context_refs": payload.get("browser_context_refs", []),
        "has_diagnostics": "diagnostics" in payload,
        "has_recovery": "recovery" in payload or "recovery_steps" in payload or payload.get("status") == "passed",
    })
contract={
    "schema":"focusa.spec135.uiai_evaluation_recovery.v1",
    "acceptance_criteria":"UIAI evaluations pass production workflows and every failure produces bounded diagnostics and recovery.",
    "evaluation_receipts":receipts,
    "diagnostic_categories":["visual","console","exceptions","network","scope","failed_requests"],
    "failure_envelope":{
        "bounded":True,
        "required_fields":["failure_class","summary","session_origin","browser_context_ref","diagnostics_ref","recovery_steps"],
        "secret_safe":True,
        "raw_page_dump":False,
    },
    "recovery_outcomes":["capture_pending","scope_mismatch","origin_mismatch","blocked","retry_from_cursor","snapshot_fallback"],
    "laws":[
        "A Focusa link failure must not destroy the UIAI artifact",
        "Diagnostics preserve exact browser session and context origin",
        "Visual failure evidence includes screenshot or stable artifact ref",
        "Console and network evidence remain bounded and redacted",
        "Every non-passed scenario requires explicit recovery steps",
    ],
}
OUT.write_text(json.dumps(contract,indent=2)+"\n")
print(f"Spec 135C-4 UIAI recovery evidence generated: {len(receipts)} scenarios")
