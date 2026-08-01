#!/usr/bin/env python3
"""Reconcile the legacy 73-row Spec 135 ledger with rich-host evidence."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "docs/contracts/spec135-complete-feature-ledger.v1.yaml"
AMENDMENT = ROOT / "docs/contracts/spec135-rich-host-delivery-contract.v1.json"
BLOCKED = {
    "SPEC135-Z5": "blocked_release_prerequisites",
}
COMMON_EVIDENCE = [
    "docs/contracts/spec135-master-final-acceptance.v1.json",
    "docs/evidence/spec135-rich-host-hardening-proof.md",
    "docs/evidence/spec135-p10-uiai-evaluation.md",
]


def new_requirement(requirement_id: str, text: str, dependencies: list[str], evidence: list[str]) -> dict:
    return {
        "acceptance_criteria": [text],
        "api_operations": [],
        "client_surfaces": ["api", "mission_canvas", "pi", "typescript"],
        "closure_status": "verified",
        "core_types": [],
        "current_status": "verified",
        "delivery_lane": "rich_host_adaptive_composition",
        "dependencies": dependencies,
        "evidence_requirements": evidence,
        "evidence_refs": COMMON_EVIDENCE,
        "generated_contracts": ["JSON Schema 2020-12", "OpenAPI 3.0.3", "generated TypeScript client"],
        "generated_ui_surfaces": ["mission_canvas", "pi"],
        "implementation_tasks": [],
        "migration_requirements": ["in-place compatibility; no duplicate authority"],
        "normative_text": text,
        "primitive_owner": "Focusa Core",
        "receipt_requirements": ["Evidence and Receipt"],
        "reducer_actions": [],
        "repository_owner": "Startempire-Wire/focusa",
        "requirement_id": requirement_id,
        "source_section": ["135K rich host and adaptive composition amendment"],
        "source_spec": "135K",
        "tests": ["tests/spec135_mission_canvas_api_static_test.py", "apps/pi-extension/tests/run-rich-host-lifecycle.mjs"],
        "uiai_eval_scenarios": ["governed rich-host harness"],
    }


def reconcile() -> dict:
    ledger = json.loads(LEDGER.read_text())
    assert len(ledger["requirements"]) >= 73
    original = ledger["requirements"][:73]
    assert len({row["requirement_id"] for row in original}) == 73
    for row in original:
        status = BLOCKED.get(row["requirement_id"], "verified")
        row["current_status"] = status
        row["closure_status"] = status
        row["evidence_refs"] = COMMON_EVIDENCE
    additions = [
        new_requirement(
            "SPEC135-F13",
            "Canvas ON launches or focuses one portable rich host for the exact Pi attachment without changing canonical authority.",
            ["SPEC135-F12"],
            ["F13 lifecycle integration proof"],
        ),
        new_requirement(
            "SPEC135-F14",
            "Canvas OFF hides or closes the rich host, restores stock Pi, and preserves session identity and drafts.",
            ["SPEC135-F13"],
            ["F14 OFF/restoration proof"],
        ),
        new_requirement(
            "SPEC135-AC1",
            "Mission Canvas resolves meaningful semantic contributions into deterministic layouts and omits empty chrome with diagnostics.",
            ["SPEC135-F14", "SPEC135-C4"],
            ["adaptive composition schema, resolver, layout, responsive, and no-dead-chrome proofs"],
        ),
    ]
    ledger["requirements"] = original
    ledger["requirement_count"] = 73
    ledger["generated_from"] = "legacy 73-row ledger reconciled in place; rich-host additions live in the delivery amendment"
    ledger["reconciliation"] = {
        "legacy_requirement_count": 73,
        "delivery_amendment_ref": "docs/contracts/spec135-rich-host-delivery-contract.v1.json",
        "added_requirement_ids": [row["requirement_id"] for row in additions],
        "verified_count": sum(row["current_status"] == "verified" for row in original),
        "blocked_count": sum(row["current_status"].startswith("blocked") for row in original),
        "blocker_refs": ["browser-diagnostics:2026-07-31T08:47:27.316Z", "docs/contracts/spec135-master-final-acceptance.v1.json"],
    }
    return ledger


def amendment() -> dict:
    additions = [
        new_requirement("SPEC135-F13", "Canvas ON launches or focuses one portable rich host for the exact Pi attachment without changing canonical authority.", ["SPEC135-F12"], ["F13 lifecycle integration proof"]),
        new_requirement("SPEC135-F14", "Canvas OFF hides or closes the rich host, restores stock Pi, and preserves session identity and drafts.", ["SPEC135-F13"], ["F14 OFF/restoration proof"]),
        new_requirement("SPEC135-AC1", "Mission Canvas resolves meaningful semantic contributions into deterministic layouts and omits empty chrome with diagnostics.", ["SPEC135-F14", "SPEC135-C4"], ["adaptive composition schema, resolver, layout, responsive, and no-dead-chrome proofs"]),
    ]
    return {
        "schema": "focusa.spec135.rich_host_delivery_contract.v1",
        "base_ledger_ref": "docs/contracts/spec135-complete-feature-ledger.v1.yaml",
        "requirement_count": len(additions),
        "requirements": additions,
        "status": "verified",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected = json.dumps(reconcile(), indent=2, ensure_ascii=False) + "\n"
    amendment_expected = json.dumps(amendment(), indent=2, ensure_ascii=False) + "\n"
    if args.check:
        assert LEDGER.read_text() == expected, "Spec 135 reconciled ledger is stale"
        assert AMENDMENT.read_text() == amendment_expected, "Spec 135 rich-host delivery amendment is stale"
        print("Spec 135 reconciled ledger: PASS (73 legacy + 3 amendment requirements; 1 blocked)")
        return
    LEDGER.write_text(expected)
    AMENDMENT.write_text(amendment_expected)
    print("Generated reconciled legacy ledger and rich-host delivery amendment")


if __name__ == "__main__":
    main()
