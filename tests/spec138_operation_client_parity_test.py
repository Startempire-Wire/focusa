#!/usr/bin/env python3
"""Behavioral parity gate for the 27 generated Spec138/138A operations."""
from __future__ import annotations

import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec138-generated-operation-contracts.v1.json"

EXPECTED = [
    ("prediction.question.create", "POST", "/v1/prediction-questions"),
    ("prediction.information_set.commit", "POST", "/v1/information-sets"),
    ("prediction.commit", "POST", "/v1/predictions/commit"),
    ("prediction.supersede", "POST", "/v1/predictions/{id}/supersede"),
    ("prediction.get", "GET", "/v1/predictions/{id}"),
    ("prediction.list", "GET", "/v1/predictions/recent"),
    ("outcome.claim", "POST", "/v1/outcomes/claim"),
    ("outcome.dispute", "POST", "/v1/outcomes/{id}/dispute"),
    ("outcome.resolve", "POST", "/v1/outcomes/resolve"),
    ("outcome.correct", "POST", "/v1/outcomes/{id}/correct"),
    ("prediction.evaluate", "POST", "/v1/evaluations/predictions"),
    ("calibration.report", "GET", "/v1/calibration/reports"),
    ("metacognition.signal.capture", "POST", "/v1/metacognition/signals"),
    ("metacognition.reflect", "POST", "/v1/metacognition/reflections"),
    ("metacognition.adjustment.propose", "POST", "/v1/metacognition/adjustments"),
    ("metacognition.adjustment.evaluate", "POST", "/v1/metacognition/evaluations"),
    ("learning.candidate.decide", "POST", "/v1/learning/candidates/{id}/decide"),
    ("learning.apply", "POST", "/v1/learning/{id}/apply"),
    ("learning.transfer.resolve", "POST", "/v1/learning/transfers/resolve"),
    ("learning.retrieve", "GET", "/v1/learning/retrieve"),
    ("learning.conflicts", "GET", "/v1/learning/conflicts"),
    ("learning.expire", "POST", "/v1/learning/{id}/expire"),
    ("learning.supersede", "POST", "/v1/learning/{id}/supersede"),
    ("learning.revoke", "POST", "/v1/learning/{id}/revoke"),
    ("learning.rollback", "POST", "/v1/learning/{id}/rollback"),
    ("learning.consolidate", "POST", "/v1/learning/consolidate"),
    ("self_model.get", "GET", "/v1/self-model"),
]


def load_json(path: str):
    return json.loads((ROOT / path).read_text())


def tuples(rows):
    return [(row["operation_id"], row["method"], row["path"]) for row in rows]


def generated_ts_rows(path: str):
    text = (ROOT / path).read_text()
    match = re.search(r"SPEC138_OPERATIONS[^=]*= (\[.*\]);\n", text)
    assert match, f"{path}: generated operation array missing"
    return json.loads(match.group(1))


def test_canonical_contract_matches_matrix_and_api_vocabulary() -> None:
    contract = load_json("docs/contracts/spec138-generated-operation-contracts.v1.json")
    matrix = yaml.safe_load((ROOT / "docs/contracts/spec138-operation-client-parity-matrix.v1.yaml").read_text())
    assert contract["operation_count"] == 27
    assert tuples(contract["operations"]) == EXPECTED
    assert [row["operation"] for row in matrix["rows"]] == [row[0] for row in EXPECTED]
    assert all(row["authority"] == "durable_scoped_prediction_authority" for row in contract["operations"])
    assert all(row["client_authority"] is False for row in contract["operations"])
    assert all(row["accepted_event_kinds"] for row in contract["operations"] if row["method"] == "POST")
    registry = load_json("docs/contracts/spec135/generated-contract-v1/operation-registry.json")
    assert registry["spec138_operation_contract_schema"] == contract["schema"]
    assert tuples(registry["spec138_operations"]) == EXPECTED
    descriptors = {row["operation_id"]: row for row in registry["operations"]}
    for operation_id, method, path in EXPECTED:
        descriptor = descriptors[f"focusa.{operation_id}"]
        assert (descriptor["method"], descriptor["path"], descriptor["canonical"]) == (method, path, True)
    openapi = load_json("docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json")
    for operation_id, method, path in EXPECTED:
        assert openapi["paths"][path][method.lower()]["operationId"] == f"focusa.{operation_id}"


def test_generated_rust_pi_and_menubar_tables_are_exact() -> None:
    contract = load_json("docs/contracts/spec138-generated-operation-contracts.v1.json")
    assert tuples(generated_ts_rows("apps/pi-extension/src/generated/spec138-operations.ts")) == EXPECTED
    assert tuples(generated_ts_rows("apps/menubar/src/lib/generated/spec138-operations.ts")) == EXPECTED
    rust = (ROOT / "crates/focusa-core/src/spec138_operations.rs").read_text()
    actual = re.findall(
        r'Spec138OperationDescriptor \{\s*operation_id: "([^"]+)",\s*method: "([^"]+)",\s*path: "([^"]+)"', rust
    )
    assert actual == EXPECTED
    assert len({row[0] for row in actual}) == len(actual)
    assert len({(row[1], row[2]) for row in actual}) == len(actual)
    assert len(contract["operations"]) == len(actual)


def test_api_cli_and_pi_invoke_generated_descriptors() -> None:
    api = (ROOT / "crates/focusa-api/src/routes/prediction_authority_canonical.rs").read_text()
    for _, method, path in EXPECTED:
        route = rf'\.route\(\s*"{re.escape(path)}"\s*,\s*{method.lower()}\('
        assert re.search(route, api), f"canonical API route missing: {method} {path}"
    assert "route does not accept this ScopedAuthorityEvent variant" in api
    assert "path id is not referenced by the authority event" in api
    assert "append_batch(vec![body.event.clone()])" in api
    assert "PersistentPredictionAuthorityLedger::for_scope" in api
    cli = (ROOT / "crates/focusa-cli/src/commands/predict.rs").read_text()
    assert "spec138_operation(&operation)" in cli and "descriptor.method == \"GET\"" in cli
    pi = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
    assert 'name: "focusa_epistemic_operation"' in pi
    assert "spec138Operation(String(p.operation_id" in pi and "bindSpec138OperationPath" in pi
    projection = load_json("docs/contracts/spec141/generated-capability-v2/pi-tools.json")
    tool = next(row for row in projection["tools"] if row["name"] == "focusa_epistemic_operation")
    ids = [row["const"] for row in tool["parameters"]["properties"]["operation_id"]["anyOf"]]
    assert ids == [row[0] for row in EXPECTED]
    capabilities = load_json("docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json")
    descriptor = next(
        row for row in capabilities["descriptors"]
        if row["tool_names"]["pi"] == "focusa_epistemic_operation"
    )
    assert [(row["method"], row["path"]) for row in descriptor["tool_names"]["rest"]] == [
        (method, path) for _, method, path in EXPECTED
    ]
    assert descriptor["tool_names"]["cli"] == ["focusa predict operation --operation <operation-id>"]


def test_ui_affordances_are_exact_and_non_authoritative() -> None:
    state = (ROOT / "apps/pi-extension/src/state.ts").read_text()
    canvas = (ROOT / "apps/pi-extension/src/mission-canvas-model.ts").read_text()
    menubar_api = (ROOT / "apps/menubar/src/lib/api.ts").read_text()
    menubar_ui = (ROOT / "apps/menubar/src/lib/components/EpistemicAuthorityPeek.svelte").read_text()
    tui = (ROOT / "crates/focusa-tui/src/mission_control.rs").read_text()
    assert "spec138FocusSliceAffordances" in state and "client_authority: false" in state
    assert "spec138MissionCanvasAffordances" in canvas and "client_authority: false" in canvas
    assert "requestSpec138Operation" in menubar_api and "if (!event)" in menubar_api
    assert "SPEC138_OPERATIONS.length" in menubar_ui
    assert "spec138-generated-operation-contracts.v1.json" in tui and "daemon_authority=true" in tui


if __name__ == "__main__":
    tests = sorted((name, value) for name, value in globals().items() if name.startswith("test_") and callable(value))
    assert tests, "no Spec138 operation parity tests discovered"
    for name, test in tests:
        test()
        print(f"PASS {name}")
    print(f"Spec138 operation client parity: PASS ({len(tests)} tests, {len(EXPECTED)} operations)")
