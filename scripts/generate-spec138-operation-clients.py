#!/usr/bin/env python3
"""Generate the canonical Spec 138/138A operation contract and client tables."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec138-generated-operation-contracts.v1.json"
RUST = ROOT / "crates/focusa-core/src/spec138_operations.rs"
PI = ROOT / "apps/pi-extension/src/generated/spec138-operations.ts"
MENUBAR = ROOT / "apps/menubar/src/lib/generated/spec138-operations.ts"
OPERATION_REGISTRY = ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json"

# id, method, canonical path, label, accepted typed authority-event kinds
OPERATIONS = [
    ("prediction.question.create", "POST", "/v1/prediction-questions", "Create prediction question", ["question"]),
    ("prediction.information_set.commit", "POST", "/v1/information-sets", "Commit information set", ["epistemic_primitive"]),
    ("prediction.commit", "POST", "/v1/predictions/commit", "Commit prediction", ["commitment", "action_commitment"]),
    ("prediction.supersede", "POST", "/v1/predictions/{id}/supersede", "Supersede prediction", ["commitment", "memory_lifecycle"]),
    ("prediction.get", "GET", "/v1/predictions/{id}", "Get prediction", []),
    ("prediction.list", "GET", "/v1/predictions/recent", "List recent predictions", []),
    ("outcome.claim", "POST", "/v1/outcomes/claim", "Claim outcome", ["outcome_claim", "outcome_authority"]),
    ("outcome.dispute", "POST", "/v1/outcomes/{id}/dispute", "Dispute outcome", ["outcome_authority"]),
    ("outcome.resolve", "POST", "/v1/outcomes/resolve", "Resolve outcome", ["outcome_resolution", "outcome_authority", "action_outcome"]),
    ("outcome.correct", "POST", "/v1/outcomes/{id}/correct", "Correct outcome", ["outcome_authority"]),
    ("prediction.evaluate", "POST", "/v1/evaluations/predictions", "Evaluate prediction", ["evaluation"]),
    ("calibration.report", "GET", "/v1/calibration/reports", "Read calibration report", []),
    ("metacognition.signal.capture", "POST", "/v1/metacognition/signals", "Capture metacognitive signal", ["epistemic_primitive", "reflection_claim"]),
    ("metacognition.reflect", "POST", "/v1/metacognition/reflections", "Record reflection", ["reflection_claim"]),
    ("metacognition.adjustment.propose", "POST", "/v1/metacognition/adjustments", "Propose adjustment", ["promotion_assessment", "learning_candidate"]),
    ("metacognition.adjustment.evaluate", "POST", "/v1/metacognition/evaluations", "Evaluate adjustment", ["promotion_assessment", "learning_settlement"]),
    ("learning.candidate.decide", "POST", "/v1/learning/candidates/{id}/decide", "Decide learning candidate", ["promotion_decision", "learning_settlement"]),
    ("learning.apply", "POST", "/v1/learning/{id}/apply", "Apply learning", ["learning_record", "learning_settlement"]),
    ("learning.transfer.resolve", "POST", "/v1/learning/transfers/resolve", "Resolve learning transfer", ["transfer_evaluation", "transfer_outcome"]),
    ("learning.retrieve", "GET", "/v1/learning/retrieve", "Retrieve learning", []),
    ("learning.conflicts", "GET", "/v1/learning/conflicts", "Read learning conflicts", []),
    ("learning.expire", "POST", "/v1/learning/{id}/expire", "Expire learning", ["memory_lifecycle"]),
    ("learning.supersede", "POST", "/v1/learning/{id}/supersede", "Supersede learning", ["memory_lifecycle", "learning_record"]),
    ("learning.revoke", "POST", "/v1/learning/{id}/revoke", "Revoke learning", ["memory_lifecycle"]),
    ("learning.rollback", "POST", "/v1/learning/{id}/rollback", "Roll back learning", ["memory_lifecycle", "learning_settlement"]),
    ("learning.consolidate", "POST", "/v1/learning/consolidate", "Consolidate learning", ["learning_record", "learning_settlement"]),
    ("self_model.get", "GET", "/v1/self-model", "Read self model", []),
]


def rows():
    return [
        {
            "operation_id": op_id,
            "method": method,
            "path": path,
            "label": label,
            "mode": "read" if method == "GET" else "canonical_mutation",
            "authority": "durable_scoped_prediction_authority",
            "scope": ["project_root", "continuity_id"],
            "request_contract": "focusa.spec138_operation_read.v1" if method == "GET" else "focusa.spec138_operation_mutation.v1",
            "response_contract": "focusa.spec138_operation_result.v1",
            "accepted_event_kinds": kinds,
            "client_authority": False,
        }
        for op_id, method, path, label, kinds in OPERATIONS
    ]


def contract_text() -> str:
    value = {
        "schema": "focusa.spec138_generated_operation_contracts.v1",
        "source": "Spec138 section 23 plus Spec138A client parity matrix",
        "operation_count": len(OPERATIONS),
        "operations": rows(),
    }
    return json.dumps(value, indent=2) + "\n"


def rust_text() -> str:
    entries = []
    for row in rows():
        kinds = ", ".join(json.dumps(value) for value in row["accepted_event_kinds"])
        values = tuple(json.dumps(row[key]) for key in ("operation_id", "method", "path", "label")) + (kinds,)
        entries.append(
            "    Spec138OperationDescriptor {\n"
            "        operation_id: %s,\n        method: %s,\n        path: %s,\n"
            "        label: %s,\n        accepted_event_kinds: &[%s],\n    },"
            % values
        )
    return """// @generated by scripts/generate-spec138-operation-clients.py; do not edit.\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct Spec138OperationDescriptor {\n    pub operation_id: &'static str,\n    pub method: &'static str,\n    pub path: &'static str,\n    pub label: &'static str,\n    pub accepted_event_kinds: &'static [&'static str],\n}\n\npub const SPEC138_OPERATIONS: &[Spec138OperationDescriptor] = &[\n%s\n];\n\npub fn spec138_operation(operation_id: &str) -> Option<&'static Spec138OperationDescriptor> {\n    SPEC138_OPERATIONS\n        .iter()\n        .find(|row| row.operation_id == operation_id)\n}\n""" % "\n".join(entries)


def ts_text() -> str:
    return """// @generated by scripts/generate-spec138-operation-clients.py; do not edit.\nexport interface Spec138OperationDescriptor {\n  operation_id: string; method: \"GET\" | \"POST\"; path: string; label: string;\n  mode: \"read\" | \"canonical_mutation\"; authority: string; scope: string[];\n  request_contract: string; response_contract: string; accepted_event_kinds: string[];\n  client_authority: false;\n}\nexport const SPEC138_OPERATIONS: readonly Spec138OperationDescriptor[] = %s;\nexport const SPEC138_OPERATION_IDS = SPEC138_OPERATIONS.map((row) => row.operation_id);\nexport function spec138Operation(operationId: string) {\n  return SPEC138_OPERATIONS.find((row) => row.operation_id === operationId);\n}\nexport function bindSpec138OperationPath(path: string, id?: string): string {\n  if (!path.includes(\"{id}\")) return path;\n  if (!id?.trim()) throw new Error(\"operation path requires id\");\n  return path.replace(\"{id}\", encodeURIComponent(id.trim()));\n}\n""" % json.dumps(rows(), separators=(",", ":"))


def write_or_check(path: Path, expected: str, check: bool) -> bool:
    if check:
        return path.exists() and path.read_text() == expected
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(expected)
    return True


def sync_operation_registry(check: bool) -> bool:
    registry = json.loads(OPERATION_REGISTRY.read_text())
    expected = rows()
    if check:
        return (
            registry.get("spec138_operation_contract_schema")
            == "focusa.spec138_generated_operation_contracts.v1"
            and registry.get("spec138_operations") == expected
        )
    registry["spec138_operation_contract_schema"] = "focusa.spec138_generated_operation_contracts.v1"
    registry["spec138_operations"] = expected
    OPERATION_REGISTRY.write_text(json.dumps(registry, indent=2) + "\n")
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    expected = {CONTRACT: contract_text(), RUST: rust_text(), PI: ts_text(), MENUBAR: ts_text()}
    ok = all(write_or_check(path, text, args.check) for path, text in expected.items())
    ok = sync_operation_registry(args.check) and ok
    print(json.dumps({"status": "passed" if ok else "blocked", "operation_count": len(OPERATIONS), "mode": "check" if args.check else "write"}))
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
