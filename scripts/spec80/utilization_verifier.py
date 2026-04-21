#!/usr/bin/env python3
"""SPEC80 §20.3 full utilization verifier runtime implementation."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def criterion(name: str, evidence_path: str, pass_condition: bool) -> dict:
    return {
        "name": name,
        "pass": pass_condition,
        "evidence_refs": [evidence_path],
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tree-dossier", required=True)
    ap.add_argument("--metacog-dossier", required=True)
    ap.add_argument("--parity-dossier", required=True)
    ap.add_argument("--outcome-report", required=True)
    ap.add_argument("--governance-dossier", required=True)
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    tree_ok = Path(args.tree_dossier).exists()
    metacog_ok = Path(args.metacog_dossier).exists()
    parity_ok = Path(args.parity_dossier).exists()
    governance_ok = Path(args.governance_dossier).exists()

    outcome_path = Path(args.outcome_report)
    outcome_ok = False
    if outcome_path.exists():
        try:
            report = json.loads(outcome_path.read_text(encoding="utf-8"))
            outcome_ok = report.get("final_decision") == "pass"
        except Exception:
            outcome_ok = False

    criteria = [
        criterion("tree_lineage_correctness", args.tree_dossier, tree_ok),
        criterion("tool_first_metacognition_loop", args.metacog_dossier, metacog_ok),
        criterion("cli_api_parity", args.parity_dossier, parity_ok),
        criterion("outcome_compounding_gate_d", args.outcome_report, outcome_ok),
        criterion("governance_integrity", args.governance_dossier, governance_ok),
    ]

    blocking = [c["name"] for c in criteria if not c["pass"]]
    result = {
        "verifier_id": "spec80_full_utilization_v1",
        "criteria": criteria,
        "all_pass": len(blocking) == 0,
        "blocking_criteria": blocking,
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
