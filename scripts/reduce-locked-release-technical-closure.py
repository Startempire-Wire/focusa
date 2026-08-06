#!/usr/bin/env python3
"""Fail-closed technical closure reducer for the admitted locked release."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof" / "audit"
LEDGER_PATH = AUDIT / "next-locked-release-governance-reconciliation.json"
OUTPUT = AUDIT / "next-locked-release-technical-closure-gate.json"
SCHEMA = "focusa.locked_release_technical_closure_gate.v1"
RECEIPT_SCHEMA = "focusa.locked_release_technical_closure_receipt.v1"


def digest_value(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(payload).hexdigest()


def load_ledger(path: Path = LEDGER_PATH) -> dict:
    ledger = json.loads(path.read_text())
    if ledger.get("schema") != "focusa.locked_release_governance_reconciliation.v1":
        raise SystemExit("unexpected governance reconciliation schema")
    return ledger


def technical_readiness(ledger: dict) -> tuple[dict[str, bool], dict[str, list[str]]]:
    mappings = {row["bead_id"]: row for row in ledger["mappings"]}
    memo: dict[str, bool] = {}
    bases: dict[str, list[str]] = {}

    def ready(bead_id: str, visiting: set[str]) -> bool:
        if bead_id in memo:
            return memo[bead_id]
        if bead_id in visiting:
            memo[bead_id] = False
            bases[bead_id] = ["dependency_cycle"]
            return False
        row = mappings[bead_id]
        has_direct_evidence = bool(
            row.get("implementation_commit_refs")
            or row.get("runtime_or_acceptance_evidence_refs")
        )
        direct = has_direct_evidence and (
            row.get("provider_state") == "closed"
            or row.get("technical_acceptance_claim") is True
        )
        if direct:
            memo[bead_id] = not row.get("active_blocker_refs")
            bases[bead_id] = [
                "direct_evidence"
                if row.get("provider_state") == "closed"
                else "explicit_verified_acceptance_claim"
            ]
            return memo[bead_id]

        duplicate_of = (row.get("closure_receipt") or {}).get("exact_duplicate_of")
        if duplicate_of in mappings:
            target_ready = ready(duplicate_of, visiting | {bead_id})
            memo[bead_id] = target_ready and not row.get("active_blocker_refs")
            bases[bead_id] = [f"exact_duplicate:{duplicate_of}"]
            return memo[bead_id]

        prefix = bead_id + "."
        descendants = sorted(
            candidate_id for candidate_id in mappings if candidate_id.startswith(prefix)
        )
        if descendants:
            descendants_ready = all(
                ready(candidate_id, visiting | {bead_id})
                for candidate_id in descendants
            )
            memo[bead_id] = descendants_ready and not row.get("active_blocker_refs")
            bases[bead_id] = [f"all_admitted_descendants:{len(descendants)}"]
            return memo[bead_id]

        memo[bead_id] = False
        bases[bead_id] = ["technical_evidence_missing"]
        return False

    for bead_id in mappings:
        ready(bead_id, set())
    return memo, bases


def build_gate(ledger: dict) -> dict:
    readiness, bases = technical_readiness(ledger)
    mappings = {row["bead_id"]: row for row in ledger["mappings"]}
    invalid_closed = sorted(
        bead_id
        for bead_id, row in mappings.items()
        if row["provider_state"] == "closed" and not readiness[bead_id]
    )
    technically_pending = sorted(
        bead_id for bead_id, accepted in readiness.items() if not accepted
    )
    result = {
        "schema": SCHEMA,
        "status": "verified" if not invalid_closed else "blocked",
        "workset_id": ledger["workset_id"],
        "reconciliation_digest": ledger["reconciliation_digest"],
        "mapping_count": len(mappings),
        "technically_accepted_count": sum(readiness.values()),
        "technically_pending_count": len(technically_pending),
        "technically_pending_ids": technically_pending,
        "invalid_closed_count": len(invalid_closed),
        "invalid_closed_ids": invalid_closed,
        "policy": {
            "provider_status_is_not_proof": True,
            "allowed_acceptance_bases": [
                "direct_evidence",
                "explicit_verified_acceptance_claim",
                "exact_duplicate_of_technically_accepted_target",
                "all_admitted_descendants_technically_accepted",
            ],
            "active_blockers_fail_closed": True,
            "unknown_beads_fail_closed": True,
            "reopen_is_idempotent": True,
        },
        "acceptance_basis": {bead_id: bases[bead_id] for bead_id in sorted(bases)},
    }
    result["gate_digest"] = digest_value(result)
    return result


def evaluate(ledger: dict, bead_id: str, requested_state: str) -> tuple[dict, int]:
    mappings = {row["bead_id"]: row for row in ledger["mappings"]}
    readiness, bases = technical_readiness(ledger)
    row = mappings.get(bead_id)
    if row is None:
        allowed = False
        reason = "unknown_or_unadmitted_bead"
        current_state = None
    elif requested_state == "open":
        allowed = True
        reason = "reopen_is_replay_safe"
        current_state = row["provider_state"]
    else:
        allowed = readiness[bead_id]
        reason = (
            "technical_acceptance_satisfied"
            if allowed
            else "technical_acceptance_missing"
        )
        current_state = row["provider_state"]
    request = {
        "bead_id": bead_id,
        "requested_state": requested_state,
        "reconciliation_digest": ledger["reconciliation_digest"],
    }
    receipt = {
        "schema": RECEIPT_SCHEMA,
        "decision": "allow" if allowed else "block",
        "reason": reason,
        "request": request,
        "request_digest": digest_value(request),
        "current_provider_state": current_state,
        "technical_acceptance_basis": bases.get(bead_id, ["unadmitted"]),
    }
    receipt["receipt_digest"] = digest_value(receipt)
    return receipt, 0 if allowed else 2


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ledger", type=Path, default=LEDGER_PATH)
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--bead-id")
    parser.add_argument("--request-state", choices=("closed", "open"))
    args = parser.parse_args()
    ledger = load_ledger(args.ledger)

    if args.bead_id or args.request_state:
        if not args.bead_id or not args.request_state:
            parser.error("--bead-id and --request-state are required together")
        receipt, status = evaluate(ledger, args.bead_id, args.request_state)
        print(json.dumps(receipt, indent=2, sort_keys=True))
        return status

    gate = build_gate(ledger)
    rendered = json.dumps(gate, indent=2, sort_keys=True) + "\n"
    if args.write:
        OUTPUT.write_text(rendered)
        print(OUTPUT.relative_to(ROOT))
    elif args.check:
        if not OUTPUT.exists() or OUTPUT.read_text() != rendered:
            print(
                f"technical closure gate drift: regenerate {OUTPUT.relative_to(ROOT)}"
            )
            return 1
        print("locked-release technical closure reducer: PASS")
    else:
        print(rendered, end="")
    return 0 if gate["status"] == "verified" else 2


if __name__ == "__main__":
    raise SystemExit(main())
