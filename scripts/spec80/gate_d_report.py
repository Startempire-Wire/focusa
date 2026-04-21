#!/usr/bin/env python3
"""SPEC80 Gate D report generator runtime implementation.

Input JSON contract:
{
  "report_id": "...",
  "forms": {"valid": 50, "novel_context": 20},
  "contracts": [
    {"contract_id":"...","status":"pass|fail|insufficient_data","relative_delta":0.0,"sample_size_ok":true}
  ]
}
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", required=True)
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    payload = json.loads(Path(args.input).read_text(encoding="utf-8"))
    forms = payload.get("forms", {})
    contracts = payload.get("contracts", [])

    forms_ok = int(forms.get("valid", 0)) >= 50 and int(forms.get("novel_context", 0)) >= 20
    if len(contracts) < 6:
        decision = "insufficient_data"
        pass_count = 0
    else:
        pass_count = sum(1 for c in contracts if c.get("status") == "pass")
        failed_turn = next((c for c in contracts if c.get("contract_id") == "failed_turn_ratio"), None)
        critical_regression = bool(failed_turn and float(failed_turn.get("relative_delta", 0.0)) > 0.05)

        sample_ok = all(bool(c.get("sample_size_ok", False)) for c in contracts)
        if not forms_ok or not sample_ok:
            decision = "insufficient_data"
        elif critical_regression:
            decision = "fail"
        elif pass_count >= 4:
            decision = "pass"
        else:
            decision = "fail"

    result: dict[str, Any] = {
        "gate_id": "Gate D",
        "report_id": payload.get("report_id", "gate-d-runtime"),
        "pass_count": pass_count,
        "total_contracts": len(contracts),
        "forms": {
            "valid": int(forms.get("valid", 0)),
            "novel_context": int(forms.get("novel_context", 0)),
            "meets_floor": forms_ok,
        },
        "contracts": contracts,
        "final_decision": decision,
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
